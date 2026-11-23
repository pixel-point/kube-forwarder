//! Window content: toolbar, navigation between pages and overlay layers.

pub mod autocomplete;
pub mod cluster_add;
pub mod cluster_form;
pub mod clusters;
pub mod service_form;

use gpui::{
    AnyView, App, AppContext, Context, CursorStyle, Entity, EventEmitter, InteractiveElement,
    IntoElement, MouseButton, ParentElement, Render, ScrollHandle, SharedString,
    StatefulInteractiveElement, Styled, Subscription, Window, div, point, px,
};
use gpui_component::notification::Notification;
use gpui_component::scroll::ScrollableElement;
use gpui_component::{InteractiveElementExt, Root, WindowExt};
use kf_core::export::ImportedCluster;

use crate::dialogs;
use crate::store::{AppStore, StartupNotice, StoreEvent};
use crate::theme::{self, FONT_BASE, TOOLBAR_HEIGHT};
use crate::ui::ClickHandler;

use cluster_add::ClusterAddPage;
use cluster_form::{ClusterFormMode, ClusterFormPage};
use clusters::ClustersPage;
use service_form::{ServiceFormMode, ServiceFormPage};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Pages of the application (the routes of the original router).
#[derive(Clone, Debug)]
pub enum Route {
    Clusters,
    ClusterAdd,
    ClusterNew {
        back: Box<Route>,
    },
    ClusterEdit {
        id: String,
    },
    ClusterImport {
        imported: Box<ImportedCluster>,
    },
    ServiceNew {
        cluster_id: Option<String>,
        cluster_created: bool,
    },
    ServiceEdit {
        id: String,
    },
    ServiceClone {
        id: String,
    },
}

/// Emitted by pages to switch to another page.
pub struct Navigate(pub Route);

/// Builds a click handler that emits [`Navigate`] from a page.
pub fn navigate_on_click<T: EventEmitter<Navigate>>(cx: &Context<T>, route: Route) -> ClickHandler {
    let page = cx.weak_entity();
    std::rc::Rc::new(move |_, _, cx: &mut App| {
        let route = route.clone();
        page.update(cx, |_, cx| cx.emit(Navigate(route))).ok();
    })
}

enum Page {
    Clusters(Entity<ClustersPage>),
    ClusterAdd(Entity<ClusterAddPage>),
    ClusterForm(Entity<ClusterFormPage>),
    ServiceForm(Entity<ServiceFormPage>),
}

impl Page {
    fn view(&self) -> AnyView {
        match self {
            Page::Clusters(page) => page.clone().into(),
            Page::ClusterAdd(page) => page.clone().into(),
            Page::ClusterForm(page) => page.clone().into(),
            Page::ServiceForm(page) => page.clone().into(),
        }
    }
}

pub struct MainView {
    store: Entity<AppStore>,
    page: Page,
    scroll_handle: ScrollHandle,
    should_move_window: bool,
    _page_subscription: Subscription,
    _store_subscription: Subscription,
}

impl MainView {
    pub fn new(store: Entity<AppStore>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let (page, page_subscription) = Self::build_page(Route::Clusters, &store, window, cx);
        let store_subscription =
            cx.subscribe_in(&store, window, |_, _, event, window, cx| match event {
                StoreEvent::Error(error) => dialogs::show_error(window, cx, error),
            });

        let notice = store.update(cx, |store, _| store.startup_notice.take());
        if let Some(notice) = notice {
            cx.defer_in(window, move |_, window, cx| match notice {
                StartupNotice::Imported { from, clusters } => {
                    window.push_notification(
                        Notification::success(format!(
                            "Imported {clusters} cluster{} from the previous version of Kube Forwarder ({}).",
                            if clusters == 1 { "" } else { "s" },
                            from.display()
                        ))
                        .autohide(false),
                        cx,
                    );
                }
                StartupNotice::Warning(warning) => {
                    dialogs::show_message(window, cx, "Kube Forwarder", Some(&warning));
                }
            });
        }

        let this = Self {
            store,
            page,
            scroll_handle: ScrollHandle::new(),
            should_move_window: false,
            _page_subscription: page_subscription,
            _store_subscription: store_subscription,
        };
        #[cfg(debug_assertions)]
        this.apply_debug_env(window, cx);
        this
    }

    /// Development aid (debug builds only) used to take screenshots of every page:
    /// `KUBE_FORWARDER_DEBUG_ROUTE` opens a page (`add`, `new`, `edit:<cluster id>`,
    /// `resource-new:<cluster id>`, `resource-edit:<id>`, `resource-clone:<id>`) and
    /// `KUBE_FORWARDER_DEBUG_START` starts resources (comma separated ids).
    #[cfg(debug_assertions)]
    fn apply_debug_env(&self, window: &mut Window, cx: &mut Context<Self>) {
        if let Ok(ids) = std::env::var("KUBE_FORWARDER_DEBUG_START") {
            for id in ids.split(',').map(str::trim).filter(|id| !id.is_empty()) {
                let task = self
                    .store
                    .update(cx, |store, cx| store.start_service(id, cx));
                cx.spawn_in(window, async move |_, cx| {
                    if let Err(error) = task.await {
                        cx.update(|window, cx| dialogs::show_error(window, cx, &error))
                            .ok();
                    }
                })
                .detach();
            }
        }
        let Ok(route) = std::env::var("KUBE_FORWARDER_DEBUG_ROUTE") else {
            return;
        };
        let (name, arg) = route.split_once(':').unwrap_or((route.as_str(), ""));
        let arg = arg.to_string();
        let route = match name {
            "add" => Route::ClusterAdd,
            "new" => Route::ClusterNew {
                back: Box::new(Route::ClusterAdd),
            },
            "edit" => Route::ClusterEdit { id: arg },
            "resource-new" => Route::ServiceNew {
                cluster_id: Some(arg),
                cluster_created: true,
            },
            "resource-edit" => Route::ServiceEdit { id: arg },
            "resource-clone" => Route::ServiceClone { id: arg },
            _ => return,
        };
        // Navigate once the first page has loaded its images (GPUI notifies only the view
        // that requested an asset first).
        cx.spawn_in(window, async move |this, cx| {
            cx.background_executor()
                .timer(std::time::Duration::from_millis(300))
                .await;
            this.update_in(cx, |this, window, cx| this.navigate(route, window, cx))
                .ok();
        })
        .detach();
    }

    fn build_page(
        route: Route,
        store: &Entity<AppStore>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> (Page, Subscription) {
        let store = store.clone();
        match route {
            Route::Clusters => {
                let page = cx.new(|cx| ClustersPage::new(store, window, cx));
                let subscription = cx.subscribe_in(&page, window, Self::on_navigate);
                (Page::Clusters(page), subscription)
            }
            Route::ClusterAdd => {
                let page = cx.new(|cx| ClusterAddPage::new(store, window, cx));
                let subscription = cx.subscribe_in(&page, window, Self::on_navigate);
                (Page::ClusterAdd(page), subscription)
            }
            Route::ClusterNew { back } => {
                let page = cx.new(|cx| {
                    ClusterFormPage::new(store, ClusterFormMode::New { back: *back }, window, cx)
                });
                let subscription = cx.subscribe_in(&page, window, Self::on_navigate);
                (Page::ClusterForm(page), subscription)
            }
            Route::ClusterEdit { id } => {
                let page = cx.new(|cx| {
                    ClusterFormPage::new(store, ClusterFormMode::Edit { id }, window, cx)
                });
                let subscription = cx.subscribe_in(&page, window, Self::on_navigate);
                (Page::ClusterForm(page), subscription)
            }
            Route::ClusterImport { imported } => {
                let page = cx.new(|cx| {
                    ClusterFormPage::new(
                        store,
                        ClusterFormMode::Import {
                            imported: *imported,
                        },
                        window,
                        cx,
                    )
                });
                let subscription = cx.subscribe_in(&page, window, Self::on_navigate);
                (Page::ClusterForm(page), subscription)
            }
            Route::ServiceNew {
                cluster_id,
                cluster_created,
            } => {
                let page = cx.new(|cx| {
                    ServiceFormPage::new(
                        store,
                        ServiceFormMode::New {
                            cluster_id,
                            cluster_created,
                        },
                        window,
                        cx,
                    )
                });
                let subscription = cx.subscribe_in(&page, window, Self::on_navigate);
                (Page::ServiceForm(page), subscription)
            }
            Route::ServiceEdit { id } => {
                let page = cx.new(|cx| {
                    ServiceFormPage::new(store, ServiceFormMode::Edit { id }, window, cx)
                });
                let subscription = cx.subscribe_in(&page, window, Self::on_navigate);
                (Page::ServiceForm(page), subscription)
            }
            Route::ServiceClone { id } => {
                let page = cx.new(|cx| {
                    ServiceFormPage::new(store, ServiceFormMode::Clone { id }, window, cx)
                });
                let subscription = cx.subscribe_in(&page, window, Self::on_navigate);
                (Page::ServiceForm(page), subscription)
            }
        }
    }

    fn on_navigate<P>(
        &mut self,
        _: &Entity<P>,
        event: &Navigate,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.navigate(event.0.clone(), window, cx);
    }

    pub fn navigate(&mut self, route: Route, window: &mut Window, cx: &mut Context<Self>) {
        tracing::debug!(?route, "navigate");
        let (page, subscription) = Self::build_page(route, &self.store, window, cx);
        self.page = page;
        self._page_subscription = subscription;
        self.scroll_handle.set_offset(point(px(0.), px(0.)));
        cx.notify();
    }

    #[cfg(test)]
    pub fn page(&self) -> AnyView {
        self.page.view()
    }

    fn render_toolbar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("toolbar")
            .flex()
            .flex_none()
            .items_center()
            .h(px(TOOLBAR_HEIGHT))
            .px(px(20.))
            .border_b_1()
            .border_color(theme::border())
            .bg(theme::white())
            .on_double_click(|_, window, _| window.titlebar_double_click())
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _, _, _| this.should_move_window = true),
            )
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, _, _, _| this.should_move_window = false),
            )
            .on_mouse_move(cx.listener(|this, _, window, _| {
                if this.should_move_window {
                    this.should_move_window = false;
                    window.start_window_move();
                }
            }))
            .child(div().flex_1())
            .child(
                div()
                    .id("version")
                    .flex_none()
                    .cursor(CursorStyle::PointingHand)
                    .text_color(theme::text_quaternary())
                    .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                    .on_click(|_, window, cx| {
                        let detail = format!(
                            "Version: {VERSION}\nPlatform: {} {}\nState: {}",
                            std::env::consts::OS,
                            std::env::consts::ARCH,
                            kf_core::storage::default_state_path().display()
                        );
                        dialogs::show_message(
                            window,
                            cx,
                            &format!("Kube Forwarder v{VERSION}"),
                            Some(&detail),
                        );
                    })
                    .child(SharedString::from(format!("v{VERSION}"))),
            )
    }
}

impl Render for MainView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("main")
            .size_full()
            .flex()
            .flex_col()
            .bg(theme::white())
            .text_color(theme::text())
            .text_size(px(FONT_BASE))
            .child(self.render_toolbar(cx))
            .child(
                div()
                    .id("page-scroll")
                    .relative()
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .track_scroll(&self.scroll_handle)
                    .child(self.page.view())
                    .vertical_scrollbar(&self.scroll_handle),
            )
            .children(Root::render_dialog_layer(window, cx))
            .children(Root::render_notification_layer(window, cx))
    }
}

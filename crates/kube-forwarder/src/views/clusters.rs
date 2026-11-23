//! The list of clusters and their resources (`Clusters.vue`, `ClusterItem.vue`,
//! `ServiceItem.vue`).

use std::path::PathBuf;
use std::rc::Rc;

use gpui::prelude::FluentBuilder;
use gpui::{
    AnyElement, AppContext, BoxShadow, Context, Corner, CursorStyle, Entity, EventEmitter,
    FontWeight, InteractiveElement, IntoElement, ParentElement, Render, SharedString,
    StatefulInteractiveElement, Styled, Subscription, Window, div, img, point, px,
};
use gpui_component::WindowExt;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::input::{InputEvent, InputState};
use gpui_component::menu::{DropdownMenu, PopupMenuItem};
use gpui_component::notification::Notification;
use gpui_component::tooltip::Tooltip;
use kf_core::export;
use kf_core::model::{Cluster, Service};

use crate::dialogs;
use crate::store::{AppStore, ServiceStatus};
use crate::theme::{self, FONT_BIG, RADIUS};
use crate::ui::{self, KfButton};
use crate::views::{Navigate, Route, navigate_on_click};

pub struct ClustersPage {
    store: Entity<AppStore>,
    search: Entity<InputState>,
    query: String,
    search_focused: bool,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<Navigate> for ClustersPage {}

type PageAction = Rc<dyn Fn(&mut ClustersPage, &mut Window, &mut Context<ClustersPage>)>;

/// An entry of a "⋮" menu.
struct MenuEntry {
    label: &'static str,
    danger: bool,
    dimmed: bool,
    action: PageAction,
}

impl MenuEntry {
    fn new(
        label: &'static str,
        action: impl Fn(&mut ClustersPage, &mut Window, &mut Context<ClustersPage>) + 'static,
    ) -> Self {
        Self {
            label,
            danger: false,
            dimmed: false,
            action: Rc::new(action),
        }
    }

    fn danger(mut self) -> Self {
        self.danger = true;
        self
    }

    fn dimmed(mut self, dimmed: bool) -> Self {
        self.dimmed = dimmed;
        self
    }
}

/// What a resource row needs from the store (computed before rendering).
struct ServiceRow {
    service: Service,
    status: ServiceStatus,
    http_ports: Vec<u16>,
    issue: Option<(u16, kf_core::KfError)>,
}

impl ServiceRow {
    fn new(service: &Service, store: &AppStore) -> Self {
        Self {
            service: service.clone(),
            status: store.status(service),
            http_ports: service
                .forwards
                .iter()
                .map(|forward| forward.local_port)
                .filter(|port| store.is_http(service, *port))
                .collect(),
            issue: store.issue(&service.id).cloned(),
        }
    }
}

impl ClustersPage {
    pub fn new(store: Entity<AppStore>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let search = cx.new(|cx| InputState::new(window, cx).placeholder("Search..."));
        let subscriptions = vec![
            cx.observe(&store, |_, _, cx| cx.notify()),
            cx.subscribe_in(
                &search,
                window,
                |this, search, event: &InputEvent, _, cx| {
                    match event {
                        InputEvent::Change => this.query = search.read(cx).value().to_string(),
                        InputEvent::Focus => this.search_focused = true,
                        InputEvent::Blur => this.search_focused = false,
                        InputEvent::PressEnter { .. } => {}
                    }
                    cx.notify();
                },
            ),
        ];
        Self {
            store,
            search,
            query: String::new(),
            search_focused: false,
            _subscriptions: subscriptions,
        }
    }

    fn go(&mut self, route: Route, cx: &mut Context<Self>) {
        cx.emit(Navigate(route));
    }

    /// A "⋮" button that opens a menu.
    fn more_menu(
        &self,
        id: impl Into<gpui::ElementId>,
        outline: bool,
        entries: Vec<MenuEntry>,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let page = cx.weak_entity();
        let entries = Rc::new(entries);
        Button::new(id)
            .when(outline, |button| button.outline())
            .when(!outline, |button| button.ghost())
            .child(ui::icon("kf-dots", 4., 18., theme::text_quaternary()))
            .w(px(if outline { 26. } else { 24. }))
            .h(px(if outline { 36. } else { 24. }))
            .dropdown_menu_with_anchor(Corner::TopRight, move |mut menu, _, _| {
                for entry in entries.iter() {
                    let page = page.clone();
                    let action = entry.action.clone();
                    let label = entry.label;
                    let color = if entry.danger {
                        theme::danger()
                    } else if entry.dimmed {
                        theme::text_tertiary()
                    } else {
                        theme::text()
                    };
                    menu = menu.item(
                        PopupMenuItem::element(move |_, _| {
                            div()
                                .min_w(px(130.))
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(color)
                                .child(label)
                        })
                        .on_click(move |_, window, cx| {
                            let action = action.clone();
                            page.update(cx, |page, cx| action(page, window, cx)).ok();
                        }),
                    );
                }
                menu.min_w(px(150.))
            })
    }

    // ---------------------------------------------------------------------------------
    // Actions
    // ---------------------------------------------------------------------------------

    fn toggle_service(&mut self, service_id: String, window: &mut Window, cx: &mut Context<Self>) {
        let (status, blocking) = {
            let store = self.store.read(cx);
            let Some(service) = store.state.service(&service_id) else {
                return;
            };
            (store.status(service), store.blocking_ports(service))
        };
        match status {
            ServiceStatus::Ready => {
                let task = self
                    .store
                    .update(cx, |store, cx| store.start_service(&service_id, cx));
                cx.spawn_in(window, async move |_, cx| {
                    if let Err(error) = task.await {
                        cx.update(|window, cx| dialogs::show_error(window, cx, &error))
                            .ok();
                    }
                })
                .detach();
            }
            ServiceStatus::Connected | ServiceStatus::Connecting => {
                self.store
                    .update(cx, |store, cx| store.stop_service(&service_id, cx));
            }
            ServiceStatus::Blocked => {
                let details = blocking
                    .into_iter()
                    .map(|(port, name)| format!("{port} used by {name}"))
                    .collect::<Vec<_>>()
                    .join(";\n");
                dialogs::show_message(
                    window,
                    cx,
                    "Service can't be forwarded. Some ports already in use.",
                    Some(&details),
                );
            }
        }
    }

    fn edit_service(&mut self, service_id: String, window: &mut Window, cx: &mut Context<Self>) {
        let can_modify = {
            let store = self.store.read(cx);
            let Some(service) = store.state.service(&service_id) else {
                return;
            };
            store.status(service).can_modify()
        };
        if can_modify {
            self.go(Route::ServiceEdit { id: service_id }, cx);
        } else {
            dialogs::show_message(
                window,
                cx,
                "You must stop forwarding before editing the resource.",
                None,
            );
        }
    }

    fn delete_service(&mut self, service_id: String, window: &mut Window, cx: &mut Context<Self>) {
        let (can_modify, label) = {
            let store = self.store.read(cx);
            let Some(service) = store.state.service(&service_id) else {
                return;
            };
            (
                store.status(service).can_modify(),
                service.label().to_string(),
            )
        };
        if !can_modify {
            dialogs::show_message(
                window,
                cx,
                "You must stop forwarding before deleting the resource.",
                None,
            );
            return;
        }
        let confirmed = dialogs::confirm(
            window,
            cx,
            &format!("Are you sure you want to delete \"{label}\"?"),
            None,
            "Delete",
        );
        let store = self.store.clone();
        cx.spawn(async move |_, cx| {
            if confirmed.await {
                store
                    .update(cx, |store, cx| store.delete_service(&service_id, cx))
                    .ok();
            }
        })
        .detach();
    }

    fn delete_cluster(&mut self, cluster_id: String, window: &mut Window, cx: &mut Context<Self>) {
        let (name, count) = {
            let store = self.store.read(cx);
            let Some(cluster) = store.state.cluster(&cluster_id) else {
                return;
            };
            (
                cluster.name.clone(),
                store.state.services_of(&cluster_id).count(),
            )
        };
        let detail = (count > 0).then(|| {
            format!(
                "Its {count} resource{} will be deleted too.",
                if count == 1 { "" } else { "s" }
            )
        });
        let confirmed = dialogs::confirm(
            window,
            cx,
            &format!("Are you sure you want to delete the \"{name}\" cluster?"),
            detail.as_deref(),
            "Delete",
        );
        let store = self.store.clone();
        cx.spawn(async move |_, cx| {
            if confirmed.await {
                store
                    .update(cx, |store, cx| store.delete_cluster(&cluster_id, cx))
                    .ok();
            }
        })
        .detach();
    }

    fn set_folded(&mut self, ids: Vec<String>, folded: bool, cx: &mut Context<Self>) {
        self.store
            .update(cx, |store, cx| store.set_folded(&ids, folded, cx));
    }

    fn export_cluster(&mut self, cluster_id: String, window: &mut Window, cx: &mut Context<Self>) {
        let Some(cluster) = self.store.read(cx).state.cluster(&cluster_id).cloned() else {
            return;
        };
        let choice = dialogs::choose(
            window,
            cx,
            &format!("Export cluster: {}", cluster.name),
            Some(
                "The export contains the resources of the cluster. Kube configs usually contain \
                 credentials: include the config only if the file stays with people you trust.",
            ),
            &["Export", "Export with Config"],
        );
        let store = self.store.clone();
        cx.spawn_in(window, async move |_, cx| {
            let include_config = match choice.await {
                Some(0) => false,
                Some(1) => true,
                _ => return,
            };
            let Ok(file) = cx.update(|_, cx| {
                export::export_cluster(&store.read(cx).state, &cluster_id, include_config)
            }) else {
                return;
            };
            let file = match file {
                Ok(file) => file,
                Err(error) => {
                    cx.update(|window, cx| dialogs::show_error(window, cx, &error))
                        .ok();
                    return;
                }
            };

            let directory = dirs::document_dir()
                .or_else(dirs::home_dir)
                .unwrap_or_else(|| PathBuf::from("."));
            let file_name = export::default_file_name(&cluster.name);
            let Ok(path) = cx.update(|_, cx| cx.prompt_for_new_path(&directory, Some(&file_name)))
            else {
                return;
            };
            let Ok(Ok(Some(path))) = path.await else {
                return;
            };
            let result = std::fs::write(&path, export::to_json(&file));
            cx.update(|window, cx| match result {
                Ok(()) => window.push_notification(
                    Notification::success(format!("Exported to {}", path.display())),
                    cx,
                ),
                Err(error) => dialogs::show_error(
                    window,
                    cx,
                    &kf_core::KfError::new("Sorry, an error occurred while saving the file.")
                        .with_details(error.to_string()),
                ),
            })
            .ok();
        })
        .detach();
    }

    // ---------------------------------------------------------------------------------
    // Rendering
    // ---------------------------------------------------------------------------------

    /// Clusters sorted by name with the resources matching the search query. Clusters
    /// without matching resources are hidden while searching.
    fn visible_items(&self, store: &AppStore) -> Vec<(Cluster, Vec<ServiceRow>)> {
        let query = self.query.trim();
        store
            .state
            .sorted_clusters()
            .into_iter()
            .filter_map(|cluster| {
                let services: Vec<ServiceRow> = store
                    .state
                    .services_of(&cluster.id)
                    .filter(|service| service.matches_query(query))
                    .map(|service| ServiceRow::new(service, store))
                    .collect();
                (query.is_empty() || !services.is_empty()).then(|| (cluster.clone(), services))
            })
            .collect()
    }

    fn render_empty(&self, cx: &mut Context<Self>) -> AnyElement {
        div()
            .relative()
            .flex()
            .flex_col()
            .items_center()
            .size_full()
            .min_h(px(480.))
            .child(
                img("images/pattern.png")
                    .absolute()
                    .left_0()
                    .right_0()
                    .bottom_0()
                    .w_full()
                    .h(px(180.))
                    .opacity(0.75),
            )
            .child(
                div()
                    .mt(px(83.))
                    .mb(px(40.))
                    .flex()
                    .flex_col()
                    .items_center()
                    .text_size(px(30.))
                    .line_height(px(36.))
                    .child(
                        div()
                            .flex()
                            .child("Add your\u{a0}")
                            .child(div().text_color(theme::primary()).child("first cluster"))
                            .child("\u{a0}before"),
                    )
                    .child("you start forward services"),
            )
            .child({
                let go = navigate_on_click(cx, Route::ClusterAdd);
                KfButton::new("empty-add-cluster")
                    .label("Add a cluster")
                    .primary()
                    .large()
                    .width(220.)
                    .on_click(move |event, window, cx| go(event, window, cx))
            })
            .into_any_element()
    }

    fn render_header(
        &self,
        all_folded: bool,
        visible_ids: Vec<String>,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let fold_entry = if all_folded {
            MenuEntry::new("Show All Clusters", move |page, _, cx| {
                page.set_folded(visible_ids.clone(), false, cx)
            })
        } else {
            MenuEntry::new("Collapse All Clusters", move |page, _, cx| {
                page.set_folded(visible_ids.clone(), true, cx)
            })
        };
        let add = navigate_on_click(cx, Route::ClusterAdd);
        ui::header(
            None,
            Rc::new(|_, _, _| {}),
            div()
                .flex()
                .items_center()
                .w_full()
                .child(
                    div()
                        .w(px(if self.search_focused { 320. } else { 191. }))
                        .child(
                            ui::text_input(&self.search, 36.)
                                .prefix(ui::icon(
                                    "kf-magnifier",
                                    17.,
                                    16.,
                                    if self.search_focused {
                                        theme::text_secondary()
                                    } else {
                                        theme::text_quaternary()
                                    },
                                ))
                                .cleanable(true),
                        ),
                )
                .child(div().flex_1())
                .child(
                    KfButton::new("add-cluster")
                        .label("Add a cluster")
                        .primary()
                        .outline()
                        .on_click(move |event, window, cx| add(event, window, cx)),
                )
                .child(div().w(px(13.)))
                .child(self.more_menu("clusters-more", true, vec![fold_entry], cx)),
        )
    }

    fn render_cluster(
        &self,
        cluster: &Cluster,
        services: &[ServiceRow],
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let id = cluster.id.clone();
        let folded = cluster.folded;

        let header = div()
            .id(SharedString::from(format!("cluster-header-{id}")))
            .debug_selector(|| format!("cluster-header-{id}"))
            .flex()
            .items_center()
            .h(px(if folded { 34. } else { 35. }))
            .pl(px(15.))
            .pr(px(if folded { 10. } else { 6. }))
            .bg(theme::bg_secondary())
            .text_size(px(FONT_BIG))
            .font_weight(FontWeight::MEDIUM)
            .text_color(theme::text_tertiary())
            .when(!folded, |this| {
                this.border_b_1().border_color(theme::border())
            })
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .truncate()
                    .child(cluster.name.clone()),
            )
            .map(|this| {
                if folded {
                    let id = id.clone();
                    this.cursor(CursorStyle::PointingHand)
                        .on_click(cx.listener(move |page, _, _, cx| {
                            page.set_folded(vec![id.clone()], false, cx)
                        }))
                        .child(ui::icon(
                            "kf-arrow-dropdown",
                            15.,
                            8.,
                            theme::text_quaternary(),
                        ))
                } else {
                    let (add_id, edit_id, export_id, fold_id, delete_id) =
                        (id.clone(), id.clone(), id.clone(), id.clone(), id.clone());
                    this.child(self.more_menu(
                        SharedString::from(format!("cluster-more-{id}")),
                        false,
                        vec![
                            MenuEntry::new("Add a Resource", move |page, _, cx| {
                                page.go(
                                    Route::ServiceNew {
                                        cluster_id: Some(add_id.clone()),
                                        cluster_created: false,
                                    },
                                    cx,
                                )
                            }),
                            MenuEntry::new("Edit", move |page, _, cx| {
                                page.go(Route::ClusterEdit { id: edit_id.clone() }, cx)
                            }),
                            MenuEntry::new("Export", move |page, window, cx| {
                                page.export_cluster(export_id.clone(), window, cx)
                            }),
                            MenuEntry::new("Collapse", move |page, _, cx| {
                                page.set_folded(vec![fold_id.clone()], true, cx)
                            }),
                            MenuEntry::new("Delete", move |page, window, cx| {
                                page.delete_cluster(delete_id.clone(), window, cx)
                            })
                            .danger(),
                        ],
                        cx,
                    ))
                }
            });

        let body = (!folded).then(|| {
            if services.is_empty() {
                let add = navigate_on_click(
                    cx,
                    Route::ServiceNew {
                        cluster_id: Some(id.clone()),
                        cluster_created: false,
                    },
                );
                div()
                    .flex()
                    .justify_center()
                    .p(px(21.))
                    .child(
                        KfButton::new(SharedString::from(format!("cluster-add-resource-{id}")))
                            .label("Add a resource")
                            .primary()
                            .on_click(move |event, window, cx| add(event, window, cx)),
                    )
                    .into_any_element()
            } else {
                div()
                    .flex()
                    .flex_col()
                    .children(
                        services
                            .iter()
                            .enumerate()
                            .map(|(ix, row)| self.render_service(row, ix, cx)),
                    )
                    .into_any_element()
            }
        });

        div()
            .id(SharedString::from(format!("cluster-{id}")))
            .w_full()
            .mb(px(20.))
            .border_1()
            .border_color(theme::border())
            .rounded(px(RADIUS))
            .bg(theme::white())
            .when(!folded, |this| {
                this.shadow(vec![BoxShadow {
                    color: theme::border(),
                    offset: point(px(0.), px(5.)),
                    blur_radius: px(10.),
                    spread_radius: px(0.),
                }])
            })
            .child(header)
            .children(body)
    }

    fn render_service(&self, row: &ServiceRow, ix: usize, cx: &mut Context<Self>) -> AnyElement {
        let service = &row.service;
        let status = row.status;
        let running = status == ServiceStatus::Connected;
        let can_modify = status.can_modify();
        let id = service.id.clone();
        let address = service.bind_address().to_string();

        let mut ports = div().flex().items_center();
        for (port_ix, forward) in service.forwards.iter().enumerate() {
            if port_ix > 0 {
                ports = ports.child(
                    div()
                        .mx(px(5.))
                        .size(px(3.))
                        .rounded(px(1.5))
                        .bg(theme::text_quaternary()),
                );
            }
            let port = forward.local_port;
            if row.http_ports.contains(&port) {
                let url = format!("http://{}:{port}", url_host(&address));
                ports = ports.child(
                    div()
                        .id(SharedString::from(format!("port-{id}-{port}")))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(theme::primary())
                        .underline()
                        .cursor(CursorStyle::PointingHand)
                        .tooltip({
                            let url = url.clone();
                            move |window, cx| Tooltip::new(format!("Open {url}")).build(window, cx)
                        })
                        .on_click(move |_, _, cx| cx.open_url(&url))
                        .child(port.to_string()),
                );
            } else {
                ports = ports.child(
                    div()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(theme::text())
                        .child(port.to_string()),
                );
            }
        }

        let description = div()
            .flex()
            .flex_wrap()
            .items_center()
            .gap_x(px(4.))
            .text_color(theme::text_tertiary())
            .child("From")
            .child(
                div()
                    .text_color(theme::text())
                    .child(service.namespace.clone()),
            )
            .child("namespace exposed to")
            .when_some(service.local_address.clone(), |this, address| {
                this.child(
                    div()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(theme::text())
                        .child(address),
                )
            })
            .child(if service.forwards.len() > 1 {
                "ports"
            } else {
                "port"
            })
            .child(ports);

        let issue_line = row.issue.clone().map(|(port, error)| {
            let details = error.full_text();
            div()
                .id(SharedString::from(format!("issue-{id}")))
                .mt(px(5.))
                .flex()
                .items_center()
                .gap(px(5.))
                .text_size(px(theme::FONT_SMALL))
                .text_color(theme::warning())
                .tooltip(move |window, cx| Tooltip::new(details.clone()).build(window, cx))
                .child(ui::icon("kf-warning", 13., 13., theme::warning()))
                .child(
                    div()
                        .min_w_0()
                        .truncate()
                        .child(format!("Port {port}: {}", error.message)),
                )
        });

        let toggle_id = id.clone();
        let toggle = KfButton::new(SharedString::from(format!("toggle-{id}")))
            .primary()
            .small()
            .width(30.)
            .variant(if running {
                ui::Variant::Outline
            } else {
                ui::Variant::Filled
            })
            .icon(
                if running { "kf-pause" } else { "kf-play" },
                if running { 10. } else { 11. },
                if running { 12. } else { 14. },
            )
            .loading(status == ServiceStatus::Connecting)
            .cancelable(true)
            .dimmed(status == ServiceStatus::Blocked)
            .tooltip(match status {
                ServiceStatus::Ready => "Start forwarding",
                ServiceStatus::Connecting => "Connecting… Click to cancel",
                ServiceStatus::Connected => "Stop forwarding",
                ServiceStatus::Blocked => "Ports are used by another resource",
            })
            .on_click(cx.listener(move |page, _, window, cx| {
                page.toggle_service(toggle_id.clone(), window, cx)
            }));

        let (edit_id, clone_id, delete_id) = (id.clone(), id.clone(), id.clone());
        let menu = self.more_menu(
            SharedString::from(format!("service-more-{id}")),
            false,
            vec![
                MenuEntry::new("Edit", move |page, window, cx| {
                    page.edit_service(edit_id.clone(), window, cx)
                })
                .dimmed(!can_modify),
                MenuEntry::new("Clone", move |page, _, cx| {
                    page.go(
                        Route::ServiceClone {
                            id: clone_id.clone(),
                        },
                        cx,
                    )
                }),
                MenuEntry::new("Delete", move |page, window, cx| {
                    page.delete_service(delete_id.clone(), window, cx)
                })
                .danger()
                .dimmed(!can_modify),
            ],
            cx,
        );

        div()
            .id(SharedString::from(format!("service-{id}")))
            .debug_selector(|| format!("service-{id}"))
            .flex()
            .py(px(10.))
            .pl(px(15.))
            .pr(px(6.))
            .when(ix > 0, |this| {
                this.border_t_1().border_color(theme::border())
            })
            .child(ui::status_dot(running))
            .child(
                div()
                    .mx(px(17.))
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .child(
                        div()
                            .text_size(px(FONT_BIG))
                            .font_weight(FontWeight::MEDIUM)
                            .mb(px(5.))
                            .child(service.label().to_string()),
                    )
                    .child(description)
                    .children(issue_line),
            )
            .child(
                div()
                    .flex()
                    .flex_none()
                    .items_center()
                    .gap(px(14.))
                    .child(toggle)
                    .child(menu),
            )
            .into_any_element()
    }
}

/// Host part of a URL for a listen address (`[::1]` for IPv6 literals).
fn url_host(address: &str) -> String {
    if address.contains(':') && !address.starts_with('[') {
        format!("[{address}]")
    } else {
        address.to_string()
    }
}

impl Render for ClustersPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let store = self.store.read(cx);
        if store.state.clusters.is_empty() {
            return div().size_full().child(self.render_empty(cx));
        }

        let items = self.visible_items(store);
        let all_folded = !items.is_empty() && items.iter().all(|(cluster, _)| cluster.folded);
        let visible_ids: Vec<String> = items
            .iter()
            .map(|(cluster, _)| cluster.id.clone())
            .collect();

        let clusters: Vec<AnyElement> = items
            .iter()
            .map(|(cluster, services)| {
                self.render_cluster(cluster, services, cx)
                    .into_any_element()
            })
            .collect();

        div()
            .size_full()
            .p(px(20.))
            .flex()
            .flex_col()
            .child(self.render_header(all_folded, visible_ids, cx))
            .children(clusters)
            .when(items.is_empty(), |this| {
                this.child(
                    div()
                        .flex()
                        .flex_1()
                        .min_h(px(300.))
                        .items_center()
                        .justify_center()
                        .text_size(px(16.))
                        .text_color(theme::text_tertiary())
                        .child("Nothing found..."),
                )
            })
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Read, Write};

    use gpui::TestAppContext;
    use kf_core::model::{AppState, Cluster, ClusterConfig, Forward, Service, WorkloadType};

    use super::*;
    use crate::test_support::{click, is_rendered, setup, type_into, wait_until};

    fn service(id: &str, cluster_id: &str, alias: Option<&str>, name: &str, port: u16) -> Service {
        Service {
            id: id.into(),
            cluster_id: cluster_id.into(),
            alias: alias.map(str::to_string),
            namespace: "kf-e2e".into(),
            workload_type: WorkloadType::Deployment,
            workload_name: name.into(),
            forwards: vec![Forward::new(port, 8080)],
            local_address: None,
        }
    }

    fn cluster(id: &str, name: &str, folded: bool, config: ClusterConfig) -> Cluster {
        Cluster {
            id: id.into(),
            name: name.into(),
            folded,
            config,
        }
    }

    fn content() -> ClusterConfig {
        ClusterConfig::from_content("apiVersion: v1\n", None)
    }

    #[gpui::test]
    fn search_filters_resources(cx: &mut TestAppContext) {
        let state = AppState {
            clusters: vec![
                cluster("c1", "alpha", false, content()),
                cluster("c2", "beta", false, content()),
            ],
            services: vec![
                service("s1", "c1", Some("frontend"), "web", 18001),
                service("s2", "c1", None, "postgres-0", 18002),
                service("s3", "c2", None, "redis", 18003),
            ],
            ..AppState::default()
        };
        let (harness, cx) = setup(cx, state);
        let page = harness.page::<ClustersPage>(cx);
        for id in ["s1", "s2", "s3"] {
            assert!(is_rendered(cx, &format!("service-{id}")), "{id}");
        }

        // GPUI keeps the debug bounds of elements that are not rendered anymore, so the
        // filtered list is read from the page.
        let visible = |cx: &mut gpui::VisualTestContext| {
            let store = harness.store.clone();
            page.read_with(cx, |page, cx| {
                page.visible_items(store.read(cx))
                    .into_iter()
                    .map(|(cluster, rows)| {
                        (
                            cluster.id,
                            rows.into_iter()
                                .map(|row| row.service.id)
                                .collect::<Vec<_>>(),
                        )
                    })
                    .collect::<Vec<_>>()
            })
        };
        let search = page.read_with(cx, |page, _| page.search.clone());
        type_into(cx, &search, "POST");
        assert_eq!(
            visible(cx),
            vec![("c1".to_string(), vec!["s2".to_string()])],
            "search is case insensitive and hides clusters without matches"
        );

        cx.simulate_keystrokes("cmd-a");
        cx.simulate_input("end");
        cx.run_until_parked();
        assert_eq!(
            visible(cx),
            vec![("c1".to_string(), vec!["s1".to_string()])],
            "aliases are searched"
        );

        cx.simulate_keystrokes("cmd-a backspace");
        cx.run_until_parked();
        assert_eq!(visible(cx).len(), 2);
    }

    #[gpui::test]
    fn folded_clusters_open_on_click(cx: &mut TestAppContext) {
        let state = AppState {
            clusters: vec![cluster("c1", "alpha", true, content())],
            services: vec![service("s1", "c1", None, "web", 18001)],
            ..AppState::default()
        };
        let (harness, cx) = setup(cx, state);
        assert!(!is_rendered(cx, "service-s1"));
        click(cx, "cluster-header-c1");
        assert!(is_rendered(cx, "service-s1"));
        assert!(
            !harness.state(cx).clusters[0].folded,
            "the folded state is saved"
        );
    }

    fn http_get(port: u16) -> std::io::Result<String> {
        let mut stream = std::net::TcpStream::connect(("127.0.0.1", port))?;
        stream.set_read_timeout(Some(std::time::Duration::from_secs(20)))?;
        stream.write_all(b"GET / HTTP/1.0\r\nHost: localhost\r\n\r\n")?;
        let mut response = String::new();
        stream.read_to_string(&mut response)?;
        Ok(response)
    }

    /// Starts and stops forwarding with the play/pause button against a real cluster
    /// (see `crates/kf-core/tests/e2e.rs`).
    #[gpui::test]
    #[ignore = "needs a cluster: set KF_E2E_KUBECONFIG"]
    fn starts_and_stops_forwarding(cx: &mut TestAppContext) {
        let path = std::env::var("KF_E2E_KUBECONFIG").expect("KF_E2E_KUBECONFIG");
        let context = kf_core::kubeconfig::read_file(std::path::Path::new(&path))
            .unwrap()
            .current_context
            .unwrap();
        let port = std::net::TcpListener::bind(("127.0.0.1", 0))
            .unwrap()
            .local_addr()
            .unwrap()
            .port();
        let state = AppState {
            clusters: vec![cluster(
                "c1",
                "k3d",
                false,
                ClusterConfig::from_path(path, context),
            )],
            services: vec![service("s-web", "c1", None, "web", port)],
            ..AppState::default()
        };
        let (harness, cx) = setup(cx, state);
        let status = |cx: &mut gpui::VisualTestContext| {
            harness.store.read_with(cx, |store, _| {
                store.status(store.state.service("s-web").unwrap())
            })
        };

        click(cx, "button-toggle-s-web");
        wait_until(cx, |cx| status(cx) == ServiceStatus::Connected);
        let response = http_get(port).unwrap();
        assert!(response.contains("pod=web-"), "{response}");
        wait_until(cx, |cx| {
            harness.store.read_with(cx, |store, _| {
                store.is_http(store.state.service("s-web").unwrap(), port)
            })
        });

        click(cx, "button-toggle-s-web");
        assert_eq!(status(cx), ServiceStatus::Ready);
        wait_until(cx, |_| http_get(port).is_err());
    }
}

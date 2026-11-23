//! "Add a cluster" page (`ClusterAdd.vue`): contexts of `~/.kube/config` or of opened
//! files, pasting a config and restoring an export.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use gpui::prelude::FluentBuilder;
use gpui::{
    Context, Entity, EventEmitter, FontWeight, IntoElement, ParentElement, PathPromptOptions,
    Render, SharedString, Styled, Subscription, Window, div, px,
};
use kf_core::kubeconfig::{self, ContextInfo};
use kf_core::model::{Cluster, ClusterConfig, StoringMethod};
use kf_core::{KfError, export, k8s};

use crate::dialogs;
use crate::store::{AppStore, run_in_tokio};
use crate::theme;
use crate::ui::{self, KfButton};
use crate::views::{Navigate, Route, navigate_on_click};

struct ConfigFile {
    path: PathBuf,
    error: Option<KfError>,
    contexts: Vec<ContextInfo>,
    checked: BTreeSet<String>,
    /// Contexts of this file that are already added (same path and context).
    added: BTreeSet<String>,
}

pub struct ClusterAddPage {
    store: Entity<AppStore>,
    files: Vec<ConfigFile>,
    default_detected: bool,
    files_opened: bool,
    saving: bool,
    _subscription: Subscription,
}

impl EventEmitter<Navigate> for ClusterAddPage {}

fn same_path(a: &Path, b: &Path) -> bool {
    match (a.canonicalize(), b.canonicalize()) {
        (Ok(a), Ok(b)) => a == b,
        _ => a == b,
    }
}

impl ClusterAddPage {
    pub fn new(store: Entity<AppStore>, _window: &mut Window, cx: &mut Context<Self>) -> Self {
        let subscription = cx.observe(&store, |_, _, cx| cx.notify());
        let mut page = Self {
            store,
            files: Vec::new(),
            default_detected: false,
            files_opened: false,
            saving: false,
            _subscription: subscription,
        };
        if let Some(path) = kubeconfig::default_kubeconfig_path() {
            let file = page.load_file(path, cx);
            page.default_detected = file.error.is_none() && !file.contexts.is_empty();
            page.files.push(file);
        }
        page
    }

    fn load_file(&self, path: PathBuf, cx: &Context<Self>) -> ConfigFile {
        let (contexts, error) = match kubeconfig::read_file(&path) {
            Ok(parsed) => (parsed.contexts, None),
            Err(error) => (Vec::new(), Some(error)),
        };
        let state = &self.store.read(cx).state;
        let added = contexts
            .iter()
            .filter(|context| {
                state.clusters.iter().any(|cluster| {
                    cluster.config.storing_method == StoringMethod::Path
                        && cluster.config.current_context.as_deref() == Some(context.name.as_str())
                        && cluster.config.path.as_deref().is_some_and(|stored| {
                            same_path(&kubeconfig::expand_tilde(stored), &path)
                        })
                })
            })
            .map(|context| context.name.clone())
            .collect();
        ConfigFile {
            path,
            error,
            contexts,
            checked: BTreeSet::new(),
            added,
        }
    }

    fn open_files(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let paths = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: true,
            prompt: Some("Open".into()),
        });
        cx.spawn_in(window, async move |this, cx| {
            let Ok(Ok(Some(paths))) = paths.await else {
                return;
            };
            this.update(cx, |this, cx| {
                this.files_opened = true;
                this.files = paths
                    .into_iter()
                    .map(|path| this.load_file(path, cx))
                    .collect();
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn restore_from_json(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let paths = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: false,
            prompt: Some("Restore".into()),
        });
        cx.spawn_in(window, async move |this, cx| {
            let Ok(Ok(Some(paths))) = paths.await else {
                return;
            };
            let Some(path) = paths.into_iter().next() else {
                return;
            };
            let imported =
                kubeconfig::read_text_file(&path).and_then(|text| export::parse_import(&text));
            match imported {
                Ok(imported) => {
                    this.update(cx, |_, cx| {
                        cx.emit(Navigate(Route::ClusterImport {
                            imported: Box::new(imported),
                        }))
                    })
                    .ok();
                }
                Err(error) => {
                    cx.update(|window, cx| dialogs::show_error(window, cx, &error))
                        .ok();
                }
            }
        })
        .detach();
    }

    fn toggle(&mut self, file_ix: usize, context: String, cx: &mut Context<Self>) {
        if let Some(file) = self.files.get_mut(file_ix)
            && !file.checked.remove(&context)
        {
            file.checked.insert(context);
        }
        cx.notify();
    }

    fn selected(&self) -> Vec<Cluster> {
        let mut clusters = Vec::new();
        for file in &self.files {
            for context in &file.contexts {
                if file.checked.contains(&context.name) {
                    clusters.push(Cluster {
                        id: String::new(),
                        name: context.name.clone(),
                        folded: false,
                        config: ClusterConfig::from_path(
                            file.path.to_string_lossy(),
                            &context.name,
                        ),
                    });
                }
            }
        }
        clusters
    }

    fn save(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let clusters = self.selected();
        if self.saving || clusters.is_empty() {
            return;
        }
        self.saving = true;
        cx.notify();

        let configs: Vec<(String, ClusterConfig)> = clusters
            .iter()
            .map(|cluster| (cluster.name.clone(), cluster.config.clone()))
            .collect();
        let checks = run_in_tokio(cx, async move {
            let checks = configs
                .into_iter()
                .map(|(name, config)| async move { (name, k8s::check_connection(&config).await) });
            Ok(futures::future::join_all(checks).await)
        });

        cx.spawn_in(window, async move |this, cx| {
            let errors: Vec<String> = checks
                .await
                .unwrap_or_default()
                .into_iter()
                .filter_map(|(name, result)| {
                    result
                        .err()
                        .map(|error| format!("Failed to connect to {name}: {}", error.full_text()))
                })
                .collect();

            let proceed = if errors.is_empty() {
                true
            } else {
                let Ok(confirmed) = cx.update(|window, cx| {
                    dialogs::confirm(
                        window,
                        cx,
                        "Do you want to continue saving?",
                        Some(&errors.join("\n\n")),
                        "Continue",
                    )
                }) else {
                    return;
                };
                confirmed.await
            };

            this.update(cx, |this, cx| {
                this.saving = false;
                if proceed {
                    this.store
                        .update(cx, |store, cx| store.add_clusters(clusters, cx));
                    cx.emit(Navigate(Route::Clusters));
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn render_detected(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let has_configs = self
            .files
            .iter()
            .any(|file| file.error.is_some() || !file.contexts.is_empty());
        let any_selected = self.files.iter().any(|file| !file.checked.is_empty());

        if !has_configs {
            return ui::block()
                .mb(px(20.))
                .flex()
                .flex_col()
                .gap(px(20.))
                .child("Please, open a file first.")
                .child(
                    div().flex().child(
                        KfButton::new("open-files-empty")
                            .label("Open file(s)")
                            .primary()
                            .outline()
                            .on_click(
                                cx.listener(|this, _, window, cx| this.open_files(window, cx)),
                            ),
                    ),
                );
        }

        let intro = if self.files_opened {
            let source = if self.files.len() > 1 {
                div().child("submitted configs")
            } else {
                div()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(self.files[0].path.display().to_string())
            };
            div()
                .flex()
                .flex_wrap()
                .gap_x(px(4.))
                .child("We have detected the following clusters in the")
                .child(source)
                .child(". Please choose clusters you want to add:")
        } else {
            div()
                .flex()
                .flex_wrap()
                .gap_x(px(4.))
                .child("We have detected existing")
                .child(
                    div()
                        .font_weight(FontWeight::SEMIBOLD)
                        .child("~/.kube/config"),
                )
                .child("file with the following clusters:")
        };

        let multiple = self.files.len() > 1;
        let mut list = div().flex().flex_col().my(px(30.));
        for (file_ix, file) in self.files.iter().enumerate() {
            if multiple {
                list = list.child(
                    div()
                        .mt(px(if file_ix == 0 { 0. } else { 30. }))
                        .mb(px(10.))
                        .text_color(theme::text_secondary())
                        .child(file.path.display().to_string()),
                );
            }
            if let Some(error) = &file.error {
                list = list.child(
                    div()
                        .font_weight(FontWeight::SEMIBOLD)
                        .child("Sorry, an error occurred while opening the file.")
                        .child(
                            div()
                                .font_weight(FontWeight::NORMAL)
                                .text_color(theme::danger())
                                .child(error.full_text()),
                        ),
                );
            }
            for (context_ix, context) in file.contexts.iter().enumerate() {
                let added = file.added.contains(&context.name);
                let checked = file.checked.contains(&context.name);
                let name = context.name.clone();
                let details = match &context.user {
                    Some(user) => format!("cluster: {}, user: {user}", context.cluster),
                    None => format!("cluster: {}", context.cluster),
                };
                let label = div()
                    .flex()
                    .flex_wrap()
                    .gap_x(px(6.))
                    .child(
                        div()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(context.name.clone()),
                    )
                    .child(
                        div()
                            .text_color(theme::text_tertiary())
                            .child(SharedString::from(format!("({details})"))),
                    )
                    .when(added, |this| {
                        this.child(div().text_color(theme::success()).child("already added"))
                    });
                list = list.child(
                    div()
                        .when(context_ix > 0, |this| this.mt(px(10.)))
                        .when(added, |this| this.opacity(0.6))
                        .child(ui::checkbox(
                            SharedString::from(format!("context-{file_ix}-{context_ix}")),
                            label,
                            checked,
                            cx.listener(move |this, _, _, cx| {
                                this.toggle(file_ix, name.clone(), cx)
                            }),
                        )),
                );
            }
        }

        ui::block().mb(px(20.)).child(intro).child(list).child(
            div().flex().child(
                KfButton::new("add-selected")
                    .label("Add selected clusters")
                    .primary()
                    .disabled(!any_selected)
                    .loading(self.saving)
                    .on_click(cx.listener(|this, _, window, cx| this.save(window, cx))),
            ),
        )
    }
}

impl Render for ClusterAddPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let back = navigate_on_click(cx, Route::Clusters);
        let logo = back.clone();
        let paste = navigate_on_click(
            cx,
            Route::ClusterNew {
                back: Box::new(Route::ClusterAdd),
            },
        );

        div()
            .p(px(20.))
            .flex()
            .flex_col()
            .child(ui::header(Some(back), logo, div()))
            .when(self.files_opened || self.default_detected, |this| {
                this.child(self.render_detected(cx))
            })
            .child(
                div()
                    .flex()
                    .gap(px(20.))
                    .child(
                        ui::block()
                            .flex_1()
                            .child("Add Kubernetes config manually")
                            .child(
                                div()
                                    .mt(px(30.))
                                    .flex()
                                    .gap(px(10.))
                                    .child(
                                        KfButton::new("open-files")
                                            .label("Open file(s)")
                                            .primary()
                                            .outline()
                                            .on_click(cx.listener(|this, _, window, cx| {
                                                this.open_files(window, cx)
                                            })),
                                    )
                                    .child(
                                        KfButton::new("paste-config")
                                            .label("Paste config")
                                            .primary()
                                            .outline()
                                            .on_click(move |event, window, cx| {
                                                paste(event, window, cx)
                                            }),
                                    ),
                            ),
                    )
                    .child(
                        ui::block()
                            .flex_1()
                            .child(
                                div().flex().gap_x(px(4.)).child("Restore from").child(
                                    div()
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .child("Kube Forwarder JSON"),
                                ),
                            )
                            .child(
                                div().mt(px(30.)).flex().child(
                                    KfButton::new("restore-json")
                                        .label("Restore from JSON")
                                        .primary()
                                        .outline()
                                        .on_click(cx.listener(|this, _, window, cx| {
                                            this.restore_from_json(window, cx)
                                        })),
                                ),
                            ),
                    ),
            )
    }
}

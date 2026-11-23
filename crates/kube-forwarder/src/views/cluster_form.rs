//! Cluster form (`ClusterForm.vue`) used to create, edit and import clusters.

use std::collections::HashSet;
use std::path::PathBuf;
use std::time::Duration;

use gpui::prelude::FluentBuilder;
use gpui::{
    AppContext, Context, Entity, EventEmitter, IntoElement, ParentElement, PathPromptOptions,
    Render, SharedString, Styled, Subscription, Task, Window, div, px,
};
use gpui_component::input::{Input, InputEvent, InputState};
use kf_core::export::{ExportedService, ImportedCluster};
use kf_core::k8s::{self, ConnectionCheck};
use kf_core::kubeconfig;
use kf_core::model::{Cluster, ClusterConfig, StoringMethod};
use kf_core::validate::{self, ValidationError};

use crate::dialogs;
use crate::store::{AppStore, run_in_tokio};
use crate::theme::{self, FONT_SMALL};
use crate::ui::{self, KfButton};
use crate::views::autocomplete::{Autocomplete, AutocompleteEvent};
use crate::views::{Navigate, Route, navigate_on_click};

pub enum ClusterFormMode {
    New { back: Route },
    Edit { id: String },
    Import { imported: ImportedCluster },
}

pub struct ClusterFormPage {
    store: Entity<AppStore>,
    mode: ClusterFormMode,
    name: Entity<InputState>,
    storing_method: StoringMethod,
    content: Entity<InputState>,
    path: Entity<InputState>,
    context: Entity<Autocomplete>,
    /// The path the context suggestions were read from.
    contexts_path: Option<String>,
    contexts_task: Option<Task<()>>,
    checking: bool,
    touched: HashSet<&'static str>,
    error: Option<SharedString>,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<Navigate> for ClusterFormPage {}

impl ClusterFormPage {
    pub fn new(
        store: Entity<AppStore>,
        mode: ClusterFormMode,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let (name, config) = match &mode {
            ClusterFormMode::New { .. } => (String::new(), ClusterConfig::default()),
            ClusterFormMode::Edit { id } => store
                .read(cx)
                .state
                .cluster(id)
                .map(|cluster| (cluster.name.clone(), cluster.config.clone()))
                .unwrap_or_default(),
            ClusterFormMode::Import { imported } => (
                imported.name.clone(),
                imported.config.clone().unwrap_or_default(),
            ),
        };

        let name = cx.new(|cx| InputState::new(window, cx).default_value(name));
        let content = cx.new(|cx| {
            InputState::new(window, cx)
                .multi_line(true)
                // GPUI lays placeholders out as a single line: no line breaks here.
                .placeholder("Paste the content of a kube config (YAML)")
                .default_value(config.content.clone().unwrap_or_default())
        });
        let path = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("~/.kube/config")
                .default_value(config.path.clone().unwrap_or_default())
        });
        let context = cx.new(|cx| {
            Autocomplete::new(
                "Context name",
                config.current_context.as_deref().unwrap_or_default(),
                window,
                cx,
            )
        });

        let mut subscriptions = Vec::new();
        for (field, input) in [("name", &name), ("content", &content)] {
            subscriptions.push(cx.subscribe(input, move |this, _, event: &InputEvent, cx| {
                if matches!(event, InputEvent::Change | InputEvent::Blur) {
                    this.touched.insert(field);
                    this.error = None;
                    cx.notify();
                }
            }));
        }
        subscriptions.push(cx.subscribe_in(
            &path,
            window,
            |this, _, event: &InputEvent, window, cx| {
                if matches!(event, InputEvent::Change | InputEvent::Blur) {
                    this.touched.insert("path");
                    this.error = None;
                    this.schedule_contexts(window, cx);
                    cx.notify();
                }
            },
        ));
        subscriptions.push(
            cx.subscribe(&context, |this, _, event: &AutocompleteEvent, cx| {
                if let AutocompleteEvent::Change = event {
                    this.touched.insert("currentContext");
                    this.error = None;
                    cx.notify();
                }
            }),
        );

        let mut page = Self {
            store,
            mode,
            name,
            storing_method: config.storing_method,
            content,
            path,
            context,
            contexts_path: None,
            contexts_task: None,
            checking: false,
            touched: HashSet::new(),
            error: None,
            _subscriptions: subscriptions,
        };
        page.load_contexts(window, cx);
        page
    }

    fn back_route(&self) -> Route {
        match &self.mode {
            ClusterFormMode::New { back } => back.clone(),
            _ => Route::Clusters,
        }
    }

    fn import_services(&self) -> &[ExportedService] {
        match &self.mode {
            ClusterFormMode::Import { imported } => &imported.services,
            _ => &[],
        }
    }

    /// The cluster as currently entered.
    fn cluster(&self, cx: &Context<Self>) -> Cluster {
        let (id, folded) = match &self.mode {
            ClusterFormMode::Edit { id } => (
                id.clone(),
                self.store
                    .read(cx)
                    .state
                    .cluster(id)
                    .is_some_and(|cluster| cluster.folded),
            ),
            _ => (String::new(), false),
        };
        let current_context = self.context.read(cx).value(cx);
        Cluster {
            id,
            name: self.name.read(cx).value().trim().to_string(),
            folded,
            config: ClusterConfig {
                storing_method: self.storing_method,
                path: Some(self.path.read(cx).value().trim().to_string()),
                content: Some(self.content.read(cx).value().to_string()),
                current_context: Some(current_context),
            }
            .normalized(),
        }
    }

    fn errors(&self, cx: &Context<Self>) -> Vec<ValidationError> {
        validate::validate_cluster(&self.cluster(cx))
    }

    fn field_error(&self, errors: &[ValidationError], field: &'static str) -> Option<SharedString> {
        if !self.touched.contains(field) {
            return None;
        }
        validate::error_for(errors, field).map(|message| SharedString::from(message.to_string()))
    }

    /// Reads the contexts of the file at `path` after typing stopped for a moment.
    fn schedule_contexts(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.contexts_task = Some(cx.spawn_in(window, async move |this, cx| {
            cx.background_executor()
                .timer(Duration::from_millis(300))
                .await;
            this.update_in(cx, |this, window, cx| this.load_contexts(window, cx))
                .ok();
        }));
    }

    fn load_contexts(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let path = self.path.read(cx).value().trim().to_string();
        if self.contexts_path.as_deref() == Some(path.as_str()) {
            return;
        }
        self.contexts_path = Some(path.clone());
        let parsed = (!path.is_empty())
            .then(|| kubeconfig::read_file(&kubeconfig::expand_tilde(&path)).ok())
            .flatten();
        let (options, current) = match parsed {
            Some(parsed) => (parsed.context_names(), parsed.current_context),
            None => (Vec::new(), None),
        };
        self.context.update(cx, |context, cx| {
            if context.value(cx).is_empty()
                && let Some(current) = &current
            {
                context.set_value(current, window, cx);
            }
            context.set_options(options, cx);
        });
    }

    fn select_file(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let paths = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: false,
            prompt: Some("Select".into()),
        });
        cx.spawn_in(window, async move |this, cx| {
            let Ok(Ok(Some(paths))) = paths.await else {
                return;
            };
            let Some(path) = paths.into_iter().next() else {
                return;
            };
            let parsed = kubeconfig::read_file(&path);
            if let Err(error) = &parsed {
                let Ok(confirmed) = cx.update(|window, cx| {
                    dialogs::confirm(
                        window,
                        cx,
                        "The file contains an invalid config. Do you want to continue?",
                        Some(&error.full_text()),
                        "Continue",
                    )
                }) else {
                    return;
                };
                if !confirmed.await {
                    return;
                }
            }
            this.update_in(cx, |this, window, cx| {
                let path_text = path.display().to_string();
                this.path.update(cx, |input, cx| {
                    input.set_value(path_text.clone(), window, cx)
                });
                this.touched.insert("path");
                this.contexts_path = Some(path_text);
                let (options, current) = match parsed {
                    Ok(parsed) => (parsed.context_names(), parsed.current_context),
                    Err(_) => (Vec::new(), None),
                };
                this.context.update(cx, |context, cx| {
                    context.set_value(current.as_deref().unwrap_or_default(), window, cx);
                    context.set_options(options, cx);
                });
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn copy_from_file(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let paths = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: false,
            prompt: Some("Copy".into()),
        });
        cx.spawn_in(window, async move |this, cx| {
            let Ok(Ok(Some(paths))) = paths.await else {
                return;
            };
            let Some(path): Option<PathBuf> = paths.into_iter().next() else {
                return;
            };
            match kubeconfig::read_text_file(&path) {
                Ok(text) => {
                    this.update_in(cx, |this, window, cx| {
                        this.content
                            .update(cx, |input, cx| input.set_value(text, window, cx));
                        this.touched.insert("content");
                        cx.notify();
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

    fn check_connection(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.checking {
            return;
        }
        let config = self.cluster(cx).config;
        if let Err(error) = kubeconfig::load(&config) {
            dialogs::show_message(window, cx, "Config is invalid", Some(&error.full_text()));
            return;
        }
        self.checking = true;
        cx.notify();
        let check = run_in_tokio(cx, async move { k8s::check_connection(&config).await });
        cx.spawn_in(window, async move |this, cx| {
            let result = check.await;
            this.update_in(cx, |this, window, cx| {
                this.checking = false;
                match result {
                    Ok(ConnectionCheck::Ok) => {
                        dialogs::show_message(window, cx, "Connection successful", None)
                    }
                    Ok(ConnectionCheck::Restricted(message)) => dialogs::show_message(
                        window,
                        cx,
                        "Connection successful",
                        Some(&format!(
                            "The cluster is reachable, but this user can't list namespaces, \
                             so namespaces won't be suggested.\n\n{message}"
                        )),
                    ),
                    Err(error) => dialogs::show_message(
                        window,
                        cx,
                        "Connection failed",
                        Some(&error.full_text()),
                    ),
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn submit(&mut self, cx: &mut Context<Self>) {
        let cluster = self.cluster(cx);
        let services = self.import_services().to_vec();
        let result = self.store.update(cx, |store, cx| match &self.mode {
            ClusterFormMode::New { .. } => {
                store
                    .create_cluster(cluster, cx)
                    .map(|cluster| Route::ServiceNew {
                        cluster_id: Some(cluster.id),
                        cluster_created: true,
                    })
            }
            ClusterFormMode::Edit { .. } => {
                store.update_cluster(cluster, cx).map(|_| Route::Clusters)
            }
            ClusterFormMode::Import { .. } => store
                .import_cluster(cluster, &services, cx)
                .map(|_| Route::Clusters),
        });
        match result {
            Ok(route) => cx.emit(Navigate(route)),
            Err(errors) => {
                self.touched
                    .extend(["name", "path", "content", "currentContext"]);
                self.error = errors
                    .first()
                    .map(|error| SharedString::from(error.message.clone()));
                cx.notify();
            }
        }
    }
}

impl Render for ClusterFormPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let errors = self.errors(cx);
        let valid = errors.is_empty();
        self.context.update(cx, |context, cx| {
            let invalid = self.touched.contains("currentContext")
                && validate::error_for(&errors, "currentContext").is_some();
            context.set_invalid(invalid, cx);
        });

        let back = navigate_on_click(cx, self.back_route());
        let logo = navigate_on_click(cx, Route::Clusters);
        let cancel = back.clone();
        let is_new = !matches!(self.mode, ClusterFormMode::Edit { .. });
        let message = match &self.mode {
            ClusterFormMode::Import { imported } if !imported.services.is_empty() => Some(format!(
                "The cluster will be imported with {} resource{}",
                imported.services.len(),
                if imported.services.len() == 1 {
                    ""
                } else {
                    "s"
                }
            )),
            _ => None,
        };

        let method = self.storing_method;
        let radios = div()
            .flex()
            .gap(px(40.))
            .mt(px(10.))
            .children(StoringMethod::ALL.map(|option| {
                ui::radio(
                    SharedString::from(format!("storing-{}", option.label())),
                    option.label(),
                    option == method,
                    cx.listener(move |this, _, _, cx| {
                        this.storing_method = option;
                        this.error = None;
                        cx.notify();
                    }),
                )
            }));

        let config_fields = match method {
            StoringMethod::Content => div().mt(px(30.)).child(ui::control_group(
                "Config file",
                Some("Get this from ~/.kube/config or your cloud provider"),
                self.field_error(&errors, "content"),
                false,
                div()
                    .flex()
                    .flex_col()
                    .child(Input::new(&self.content).h(px(193.)))
                    .child(
                        div().mt(px(10.)).flex().child(
                            KfButton::new("copy-from-file")
                                .label("Copy from a file")
                                .primary()
                                .outline()
                                .small()
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.copy_from_file(window, cx)
                                })),
                        ),
                    ),
            )),
            StoringMethod::Path => div()
                .child(
                    div().mt(px(30.)).child(ui::control_group(
                        "Path",
                        None,
                        self.field_error(&errors, "path"),
                        false,
                        div()
                            .flex()
                            .items_center()
                            .gap(px(10.))
                            .child(
                                KfButton::new("select-file")
                                    .label("Select a file")
                                    .primary()
                                    .outline()
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.select_file(window, cx)
                                    })),
                            )
                            .child(div().flex_1().child(ui::text_input(&self.path, 40.))),
                    )),
                )
                .child(div().mt(px(30.)).child(ui::control_group(
                    "Current context",
                    None,
                    self.field_error(&errors, "currentContext"),
                    false,
                    self.context.clone(),
                ))),
        };

        div()
            .p(px(20.))
            .flex()
            .flex_col()
            .child(ui::header(Some(back), logo, div()))
            .child(
                ui::block()
                    .flex()
                    .flex_col()
                    .child(ui::control_group(
                        "Cluster name",
                        None,
                        self.field_error(&errors, "name"),
                        false,
                        ui::text_input(&self.name, 40.),
                    ))
                    .child(div().mt(px(30.)).child(ui::control_group(
                        "Set destination to your kube config or paste it as a text",
                        None,
                        None,
                        false,
                        radios,
                    )))
                    .child(config_fields)
                    .child(
                        div()
                            .mt(px(30.))
                            .flex()
                            .items_center()
                            .gap(px(10.))
                            .child(
                                KfButton::new("cancel")
                                    .label("Cancel")
                                    .danger()
                                    .outline()
                                    .on_click(move |event, window, cx| cancel(event, window, cx)),
                            )
                            .child(div().flex_1())
                            .map(|this| match (&self.error, message) {
                                (Some(error), _) => this.child(
                                    div()
                                        .px(px(10.))
                                        .text_size(px(FONT_SMALL))
                                        .text_color(theme::danger())
                                        .child(error.clone()),
                                ),
                                (None, Some(message)) => this.child(
                                    div()
                                        .px(px(10.))
                                        .text_size(px(FONT_SMALL))
                                        .text_color(theme::text_secondary())
                                        .child(message),
                                ),
                                (None, None) => this,
                            })
                            .child(
                                KfButton::new("check-connection")
                                    .label("Check connection")
                                    .primary()
                                    .outline()
                                    .loading(self.checking)
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.check_connection(window, cx)
                                    })),
                            )
                            .child(
                                KfButton::new("submit-cluster")
                                    .label(if is_new { "Add a cluster" } else { "Save" })
                                    .primary()
                                    .disabled(!valid)
                                    .on_click(cx.listener(|this, _, _, cx| this.submit(cx))),
                            ),
                    ),
            )
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use gpui::TestAppContext;
    use kf_core::model::AppState;

    use super::*;
    use crate::test_support::{click, setup, type_into, value};
    use crate::views::service_form::ServiceFormPage;

    #[gpui::test]
    fn creates_a_cluster_from_a_pasted_config(cx: &mut TestAppContext) {
        let (harness, cx) = setup(cx, AppState::default());
        harness.navigate(
            cx,
            Route::ClusterNew {
                back: Box::new(Route::ClusterAdd),
            },
        );
        let page = harness.page::<ClusterFormPage>(cx);

        // The form is empty: the submit button is disabled.
        click(cx, "button-submit-cluster");
        assert!(harness.state(cx).clusters.is_empty());

        let (name, content) =
            page.read_with(cx, |page, _| (page.name.clone(), page.content.clone()));
        type_into(cx, &name, "local");
        type_into(cx, &content, "apiVersion: v1\nkind: Config\n");
        click(cx, "button-submit-cluster");

        let state = harness.state(cx);
        assert_eq!(state.clusters.len(), 1);
        assert_eq!(state.clusters[0].name, "local");
        assert_eq!(
            state.clusters[0].config.storing_method,
            StoringMethod::Content
        );
        assert_eq!(
            state.clusters[0].config.content.as_deref(),
            Some("apiVersion: v1\nkind: Config\n")
        );

        // Continues with the form of the first resource of the new cluster.
        let next = harness.page::<ServiceFormPage>(cx);
        assert!(next.read_with(cx, |page, _| page.cluster_created));
    }

    #[gpui::test]
    fn suggests_the_contexts_of_a_kube_config_file(cx: &mut TestAppContext) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config");
        std::fs::write(
            &path,
            "apiVersion: v1\nkind: Config\ncurrent-context: dev\ncontexts:\n\
             - name: dev\n  context:\n    cluster: a\n- name: prod\n  context:\n    cluster: b\n",
        )
        .unwrap();

        let (harness, cx) = setup(cx, AppState::default());
        harness.navigate(
            cx,
            Route::ClusterNew {
                back: Box::new(Route::Clusters),
            },
        );
        let page = harness.page::<ClusterFormPage>(cx);
        click(cx, "choice-storing-Set a path");
        let (name, path_input, context) = page.read_with(cx, |page, _| {
            (page.name.clone(), page.path.clone(), page.context.clone())
        });

        type_into(cx, &name, "from file");
        type_into(cx, &path_input, path.to_str().unwrap());
        cx.executor().advance_clock(Duration::from_millis(400));
        cx.run_until_parked();
        let input = context.read_with(cx, |context, _| context.input.clone());
        assert_eq!(
            value(cx, &input),
            "dev",
            "the current context of the file is preselected"
        );

        click(cx, "button-submit-cluster");
        let cluster = harness.state(cx).clusters[0].clone();
        assert_eq!(cluster.config.storing_method, StoringMethod::Path);
        assert_eq!(cluster.config.path.as_deref(), path.to_str());
        assert_eq!(cluster.config.current_context.as_deref(), Some("dev"));
    }
}

//! Resource form (`ServiceForm.vue`, `ForwardsTable.vue`) used to create, edit and clone
//! resources.

use std::collections::HashSet;
use std::time::Duration;

use gpui::prelude::FluentBuilder;
use gpui::{
    App, AppContext, Context, CursorStyle, Entity, EventEmitter, InteractiveElement, IntoElement,
    ParentElement, Render, SharedString, StatefulInteractiveElement, Styled, Subscription, Task,
    Window, div, px,
};
use gpui_component::IndexPath;
use gpui_component::input::{InputEvent, InputState};
use gpui_component::select::{Select, SelectEvent, SelectItem, SelectState};
use kf_core::model::{Forward, Service, WorkloadType, new_id};
use kf_core::validate::{self, ValidationError, parse_port};
use kf_core::{k8s, kubeconfig};

use crate::store::{AppStore, run_in_tokio};
use crate::theme::{self, FONT_SMALL, RADIUS};
use crate::ui::{self, KfButton};
use crate::views::autocomplete::{Autocomplete, AutocompleteEvent};
use crate::views::{Navigate, Route, navigate_on_click};

pub enum ServiceFormMode {
    New {
        cluster_id: Option<String>,
        cluster_created: bool,
    },
    Edit {
        id: String,
    },
    Clone {
        id: String,
    },
}

#[derive(Clone)]
pub struct Choice {
    value: String,
    title: SharedString,
}

impl SelectItem for Choice {
    type Value = String;

    fn title(&self) -> SharedString {
        self.title.clone()
    }

    fn value(&self) -> &String {
        &self.value
    }
}

type ChoiceSelect = SelectState<Vec<Choice>>;

struct PortRow {
    id: String,
    local: Entity<InputState>,
    remote: Entity<InputState>,
}

pub struct ServiceFormPage {
    store: Entity<AppStore>,
    mode: ServiceFormMode,
    pub(crate) cluster_created: bool,
    cluster_select: Entity<ChoiceSelect>,
    kind_select: Entity<ChoiceSelect>,
    cluster_id: Option<String>,
    kind: Option<WorkloadType>,
    namespace: Entity<Autocomplete>,
    name: Entity<Autocomplete>,
    alias: Entity<InputState>,
    rows: Vec<PortRow>,
    new_local: Entity<InputState>,
    new_remote: Entity<InputState>,
    /// The new row mirrors the typed port into the other field until it's edited.
    new_local_edited: bool,
    new_remote_edited: bool,
    /// Values set programmatically. GPUI delivers change events after the handler that
    /// changed a value returns, so they are recognized by their value.
    expected_local: Option<String>,
    expected_remote: Option<String>,
    use_local_address: bool,
    local_address: Entity<InputState>,
    client: Option<(String, kube::Client)>,
    namespaces_key: Option<String>,
    names_key: Option<(String, String, WorkloadType)>,
    touched: HashSet<&'static str>,
    error: Option<SharedString>,
    create_row_task: Option<Task<()>>,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<Navigate> for ServiceFormPage {}

fn port_input(value: Option<u16>, window: &mut Window, cx: &mut App) -> Entity<InputState> {
    cx.new(|cx| {
        InputState::new(window, cx)
            .placeholder("")
            .default_value(value.map(|port| port.to_string()).unwrap_or_default())
    })
}

impl ServiceFormPage {
    pub fn new(
        store: Entity<AppStore>,
        mode: ServiceFormMode,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let (initial, cluster_created) = {
            let state = &store.read(cx).state;
            match &mode {
                ServiceFormMode::New {
                    cluster_id,
                    cluster_created,
                } => (
                    Service {
                        id: String::new(),
                        cluster_id: cluster_id.clone().unwrap_or_default(),
                        alias: None,
                        namespace: String::new(),
                        workload_type: WorkloadType::Pod,
                        workload_name: String::new(),
                        forwards: Vec::new(),
                        local_address: None,
                    },
                    *cluster_created,
                ),
                ServiceFormMode::Edit { id } | ServiceFormMode::Clone { id } => (
                    state.service(id).cloned().unwrap_or_else(|| Service {
                        id: String::new(),
                        cluster_id: String::new(),
                        alias: None,
                        namespace: String::new(),
                        workload_type: WorkloadType::Pod,
                        workload_name: String::new(),
                        forwards: Vec::new(),
                        local_address: None,
                    }),
                    false,
                ),
            }
        };
        let is_new = matches!(mode, ServiceFormMode::New { .. });

        let clusters: Vec<Choice> = store
            .read(cx)
            .state
            .sorted_clusters()
            .into_iter()
            .map(|cluster| Choice {
                value: cluster.id.clone(),
                title: cluster.name.clone().into(),
            })
            .collect();
        let cluster_ix = clusters
            .iter()
            .position(|choice| choice.value == initial.cluster_id);
        let cluster_id = cluster_ix.map(|ix| clusters[ix].value.clone());
        let cluster_select =
            cx.new(|cx| SelectState::new(clusters, cluster_ix.map(IndexPath::new), window, cx));

        let kinds: Vec<Choice> = WorkloadType::ALL
            .iter()
            .map(|kind| Choice {
                value: kind_value(*kind).to_string(),
                title: kind.label().into(),
            })
            .collect();
        let kind = (!is_new).then_some(initial.workload_type);
        let kind_ix = kind.and_then(|kind| WorkloadType::ALL.iter().position(|k| *k == kind));
        let kind_select =
            cx.new(|cx| SelectState::new(kinds, kind_ix.map(IndexPath::new), window, cx));

        let namespace = cx.new(|cx| Autocomplete::new("", &initial.namespace, window, cx));
        let name = cx.new(|cx| Autocomplete::new("", &initial.workload_name, window, cx));
        let alias = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Optional...")
                .default_value(initial.alias.clone().unwrap_or_default())
        });
        let local_address = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("localhost")
                .default_value(initial.local_address.clone().unwrap_or_default())
        });

        let rows: Vec<PortRow> = initial
            .forwards
            .iter()
            .map(|forward| PortRow {
                id: forward.id.clone(),
                local: port_input(Some(forward.local_port), window, cx),
                remote: port_input(Some(forward.remote_port), window, cx),
            })
            .collect();
        let new_local = port_input(None, window, cx);
        let new_remote = port_input(None, window, cx);

        let mut subscriptions = vec![
            cx.subscribe_in(
                &cluster_select,
                window,
                |this, _, event: &SelectEvent<Vec<Choice>>, _, cx| {
                    let SelectEvent::Confirm(value) = event;
                    this.cluster_id = value.clone();
                    this.touched.insert("clusterId");
                    this.error = None;
                    cx.notify();
                },
            ),
            cx.subscribe_in(
                &kind_select,
                window,
                |this, _, event: &SelectEvent<Vec<Choice>>, _, cx| {
                    let SelectEvent::Confirm(value) = event;
                    this.kind = value.as_deref().and_then(kind_from_value);
                    this.touched.insert("workloadType");
                    this.error = None;
                    cx.notify();
                },
            ),
            cx.subscribe_in(
                &namespace,
                window,
                |this, _, event: &AutocompleteEvent, window, cx| {
                    match event {
                        AutocompleteEvent::Focus => this.load_namespaces(window, cx),
                        AutocompleteEvent::Change => {
                            this.touched.insert("namespace");
                            this.error = None;
                        }
                    }
                    cx.notify();
                },
            ),
            cx.subscribe_in(
                &name,
                window,
                |this, _, event: &AutocompleteEvent, window, cx| {
                    match event {
                        AutocompleteEvent::Focus => this.load_names(window, cx),
                        AutocompleteEvent::Change => {
                            this.touched.insert("workloadName");
                            this.error = None;
                        }
                    }
                    cx.notify();
                },
            ),
            cx.subscribe(&alias, |this, _, _: &InputEvent, cx| {
                this.error = None;
                cx.notify();
            }),
            cx.subscribe(&local_address, |this, _, event: &InputEvent, cx| {
                if matches!(event, InputEvent::Change) {
                    this.touched.insert("localAddress");
                    this.error = None;
                    cx.notify();
                }
            }),
        ];
        for row in &rows {
            subscriptions.extend(Self::watch_row(row, window, cx));
        }
        subscriptions.extend(Self::watch_new_row(&new_local, true, window, cx));
        subscriptions.extend(Self::watch_new_row(&new_remote, false, window, cx));

        Self {
            store,
            mode,
            cluster_created,
            cluster_select,
            kind_select,
            cluster_id,
            kind,
            namespace,
            name,
            alias,
            rows,
            new_local,
            new_remote,
            new_local_edited: false,
            new_remote_edited: false,
            expected_local: None,
            expected_remote: None,
            use_local_address: initial.local_address.is_some(),
            local_address,
            client: None,
            namespaces_key: None,
            names_key: None,
            touched: HashSet::new(),
            error: None,
            create_row_task: None,
            _subscriptions: subscriptions,
        }
    }

    fn watch_row(row: &PortRow, window: &mut Window, cx: &mut Context<Self>) -> Vec<Subscription> {
        [&row.local, &row.remote]
            .into_iter()
            .map(|input| {
                cx.subscribe_in(input, window, |this, _, event: &InputEvent, _, cx| {
                    if matches!(event, InputEvent::Change) {
                        this.touched.insert("forwards");
                        this.error = None;
                        cx.notify();
                    }
                })
            })
            .collect()
    }

    /// The last row of the table creates a port forward when it loses focus (like the
    /// original table) and copies the typed port into the other column.
    fn watch_new_row(
        input: &Entity<InputState>,
        is_local: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Vec<Subscription> {
        vec![cx.subscribe_in(
            input,
            window,
            move |this, input, event: &InputEvent, window, cx| {
                match event {
                    InputEvent::Change => {
                        let value = input.read(cx).value().to_string();
                        let expected = if is_local {
                            &mut this.expected_local
                        } else {
                            &mut this.expected_remote
                        };
                        if expected.as_deref() == Some(value.as_str()) {
                            *expected = None;
                            return;
                        }
                        if is_local {
                            this.new_local_edited = !value.is_empty();
                            if !this.new_remote_edited {
                                this.expected_remote = Some(value.clone());
                                let target = this.new_remote.clone();
                                target.update(cx, |input, cx| input.set_value(value, window, cx));
                            }
                        } else {
                            this.new_remote_edited = !value.is_empty();
                            if !this.new_local_edited {
                                this.expected_local = Some(value.clone());
                                let target = this.new_local.clone();
                                target.update(cx, |input, cx| input.set_value(value, window, cx));
                            }
                        }
                    }
                    InputEvent::Focus => this.create_row_task = None,
                    InputEvent::Blur => {
                        // Wait for the focus to settle: moving between the two inputs of the new
                        // row must not create the forward yet.
                        this.create_row_task = Some(cx.spawn_in(window, async move |this, cx| {
                            cx.background_executor()
                                .timer(Duration::from_millis(100))
                                .await;
                            this.update_in(cx, |this, window, cx| this.create_row(window, cx))
                                .ok();
                        }));
                    }
                    InputEvent::PressEnter { .. } => this.create_row(window, cx),
                }
            },
        )]
    }

    fn create_row(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let local = self.new_local.read(cx).value().trim().to_string();
        let remote = self.new_remote.read(cx).value().trim().to_string();
        if local.is_empty() && remote.is_empty() {
            return;
        }
        let row = PortRow {
            id: new_id(),
            local: port_input(None, window, cx),
            remote: port_input(None, window, cx),
        };
        row.local
            .update(cx, |input, cx| input.set_value(local, window, cx));
        row.remote
            .update(cx, |input, cx| input.set_value(remote, window, cx));
        self._subscriptions
            .extend(Self::watch_row(&row, window, cx));
        self.rows.push(row);

        self.expected_local = Some(String::new());
        self.expected_remote = Some(String::new());
        self.new_local
            .update(cx, |input, cx| input.set_value("", window, cx));
        self.new_remote
            .update(cx, |input, cx| input.set_value("", window, cx));
        self.new_local_edited = false;
        self.new_remote_edited = false;
        self.touched.insert("forwards");
        self.error = None;
        cx.notify();
    }

    fn remove_row(&mut self, id: &str, cx: &mut Context<Self>) {
        self.rows.retain(|row| row.id != id);
        self.touched.insert("forwards");
        cx.notify();
    }

    fn cluster_config(&self, cx: &App) -> Option<(String, kf_core::model::ClusterConfig)> {
        let cluster_id = self.cluster_id.clone()?;
        let cluster = self.store.read(cx).state.cluster(&cluster_id)?;
        Some((cluster_id, cluster.config.clone()))
    }

    /// A client per selected cluster (exec credential plugins run only once).
    fn client(
        &self,
        cx: &App,
    ) -> Option<impl Future<Output = Result<kube::Client, kf_core::KfError>> + use<>> {
        let (cluster_id, config) = self.cluster_config(cx)?;
        let cached = self
            .client
            .as_ref()
            .filter(|(id, _)| *id == cluster_id)
            .map(|(_, client)| client.clone());
        let build = run_in_tokio(cx, async move {
            match cached {
                Some(client) => Ok(client),
                None => kubeconfig::client_for(&config).await,
            }
        });
        Some(build)
    }

    fn load_namespaces(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(cluster_id) = self.cluster_id.clone() else {
            return;
        };
        if self.namespaces_key.as_deref() == Some(cluster_id.as_str()) {
            return;
        }
        let Some(client) = self.client(cx) else {
            return;
        };
        self.namespaces_key = Some(cluster_id.clone());
        self.namespace
            .update(cx, |input, cx| input.set_loading(true, cx));
        let tokio = cx.global::<crate::store::Tokio>().0.clone();
        cx.spawn_in(window, async move |this, cx| {
            let result = async {
                let client = client.await?;
                let list_client = client.clone();
                let names = tokio
                    .spawn(async move { k8s::list_namespaces(&list_client).await })
                    .await
                    .map_err(|error| kf_core::KfError::new(error.to_string()))??;
                Ok::<_, kf_core::KfError>((client, names))
            }
            .await;
            this.update(cx, |this, cx| {
                match result {
                    Ok((client, names)) => {
                        this.client = Some((cluster_id, client));
                        this.namespace
                            .update(cx, |input, cx| input.set_options(names, cx));
                    }
                    Err(error) => {
                        tracing::warn!(error = %error.full_text(), "failed to list namespaces");
                        this.namespaces_key = None;
                        this.namespace
                            .update(cx, |input, cx| input.set_options(Vec::new(), cx));
                    }
                }
                this.namespace
                    .update(cx, |input, cx| input.set_loading(false, cx));
            })
            .ok();
        })
        .detach();
    }

    fn load_names(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let (Some(cluster_id), Some(kind)) = (self.cluster_id.clone(), self.kind) else {
            return;
        };
        let namespace = self.namespace.read(cx).value(cx);
        if namespace.is_empty() {
            return;
        }
        let key = (cluster_id.clone(), namespace.clone(), kind);
        if self.names_key.as_ref() == Some(&key) {
            return;
        }
        let Some(client) = self.client(cx) else {
            return;
        };
        self.names_key = Some(key);
        self.name
            .update(cx, |input, cx| input.set_loading(true, cx));
        let tokio = cx.global::<crate::store::Tokio>().0.clone();
        cx.spawn_in(window, async move |this, cx| {
            let result = async {
                let client = client.await?;
                let list_client = client.clone();
                let names =
                    tokio
                        .spawn(async move {
                            k8s::list_workload_names(&list_client, &namespace, kind).await
                        })
                        .await
                        .map_err(|error| kf_core::KfError::new(error.to_string()))??;
                Ok::<_, kf_core::KfError>((client, names))
            }
            .await;
            this.update(cx, |this, cx| {
                match result {
                    Ok((client, names)) => {
                        this.client = Some((cluster_id, client));
                        this.name
                            .update(cx, |input, cx| input.set_options(names, cx));
                    }
                    Err(error) => {
                        tracing::warn!(error = %error.full_text(), "failed to list resources");
                        this.names_key = None;
                        this.name
                            .update(cx, |input, cx| input.set_options(Vec::new(), cx));
                    }
                }
                this.name
                    .update(cx, |input, cx| input.set_loading(false, cx));
            })
            .ok();
        })
        .detach();
    }

    /// Ports of the table. Invalid ports are reported per field.
    fn forwards(&self, cx: &App) -> (Vec<Forward>, Vec<(String, bool, bool)>) {
        let mut forwards = Vec::new();
        let mut validity = Vec::new();
        for row in &self.rows {
            let local = parse_port(&row.local.read(cx).value());
            let remote = parse_port(&row.remote.read(cx).value());
            validity.push((row.id.clone(), local.is_err(), remote.is_err()));
            if let (Ok(local_port), Ok(remote_port)) = (local, remote) {
                forwards.push(Forward {
                    id: row.id.clone(),
                    local_port,
                    remote_port,
                });
            }
        }
        (forwards, validity)
    }

    fn service(&self, cx: &App) -> (Service, Vec<ValidationError>) {
        let (forwards, validity) = self.forwards(cx);
        let id = match &self.mode {
            ServiceFormMode::Edit { id } => id.clone(),
            _ => String::new(),
        };
        let service = Service {
            id,
            cluster_id: self.cluster_id.clone().unwrap_or_default(),
            alias: Some(self.alias.read(cx).value().to_string()),
            namespace: self.namespace.read(cx).value(cx),
            workload_type: self.kind.unwrap_or(WorkloadType::Pod),
            workload_name: self.name.read(cx).value(cx),
            forwards,
            local_address: self
                .use_local_address
                .then(|| self.local_address.read(cx).value().to_string()),
        }
        .normalized();

        let mut errors = validate::validate_service(&service, &self.store.read(cx).state);
        if self.kind.is_none() {
            errors.push(ValidationError {
                field: "workloadType",
                message: validate::REQUIRED.into(),
            });
        }
        if validity.iter().any(|(_, local, remote)| *local || *remote) {
            errors.push(ValidationError {
                field: "forwards",
                message: "Ports must be integers between 1 and 65535.".into(),
            });
        }
        (service, errors)
    }

    fn field_error(&self, errors: &[ValidationError], field: &'static str) -> Option<SharedString> {
        if !self.touched.contains(field) {
            return None;
        }
        validate::error_for(errors, field).map(|message| SharedString::from(message.to_string()))
    }

    fn submit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        // A port typed into the last row counts as well.
        self.create_row(window, cx);
        let (service, errors) = self.service(cx);
        if !errors.is_empty() {
            self.touched.extend([
                "clusterId",
                "namespace",
                "workloadType",
                "workloadName",
                "forwards",
                "localAddress",
            ]);
            self.error = errors
                .first()
                .map(|error| SharedString::from(error.message.clone()));
            cx.notify();
            return;
        }
        let editing = matches!(self.mode, ServiceFormMode::Edit { .. });
        let result = self.store.update(cx, |store, cx| {
            if editing {
                store.update_service(service, cx)
            } else {
                store.create_service(service, cx).map(|_| ())
            }
        });
        match result {
            Ok(()) => cx.emit(Navigate(Route::Clusters)),
            Err(errors) => {
                self.error = errors
                    .first()
                    .map(|error| SharedString::from(error.message.clone()));
                cx.notify();
            }
        }
    }

    fn render_ports(
        &self,
        row_validity: &[(String, bool, bool)],
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let cell = || {
            div()
                .flex()
                .items_center()
                .h(px(36.))
                .border_l_1()
                .border_t_1()
                .border_color(theme::table_border())
        };
        let port_cell = |input: &Entity<InputState>, invalid: bool| {
            cell()
                .flex_1()
                .when(invalid, |this| this.bg(theme::bg_danger()))
                .child(ui::text_input(input, 34.).appearance(false))
        };

        let mut table = div()
            .flex()
            .flex_col()
            .w_full()
            .rounded(px(RADIUS))
            .bg(theme::white())
            .border_r_1()
            .border_b_1()
            .border_color(theme::table_border())
            .child(
                div()
                    .flex()
                    .text_color(theme::text_tertiary())
                    .child(
                        cell()
                            .relative()
                            .flex_1()
                            .justify_center()
                            .bg(theme::bg_secondary())
                            .child("Local Port")
                            .child(div().absolute().right(px(-5.)).top(px(10.)).child(ui::icon(
                                "kf-arrow-thin-right",
                                9.,
                                15.,
                                theme::text_quaternary(),
                            ))),
                    )
                    .child(
                        cell()
                            .flex_1()
                            .justify_center()
                            .bg(theme::bg_secondary())
                            .child("Resource Port"),
                    )
                    .child(cell().w(px(40.)).bg(theme::bg_secondary())),
            );

        for row in &self.rows {
            let (local_invalid, remote_invalid) = row_validity
                .iter()
                .find(|(id, _, _)| *id == row.id)
                .map(|(_, local, remote)| (*local, *remote))
                .unwrap_or_default();
            let id = row.id.clone();
            table = table.child(
                div()
                    .flex()
                    .child(port_cell(
                        &row.local,
                        local_invalid && self.touched.contains("forwards"),
                    ))
                    .child(port_cell(
                        &row.remote,
                        remote_invalid && self.touched.contains("forwards"),
                    ))
                    .child(
                        cell().w(px(40.)).justify_center().child(
                            div()
                                .id(SharedString::from(format!("remove-{id}")))
                                .flex()
                                .items_center()
                                .justify_center()
                                .size(px(30.))
                                .rounded(px(RADIUS))
                                .cursor(CursorStyle::PointingHand)
                                .hover(|style| style.bg(theme::with_alpha(theme::danger(), 0.08)))
                                .on_click(
                                    cx.listener(move |this, _, _, cx| this.remove_row(&id, cx)),
                                )
                                .child(ui::icon("kf-cross", 11., 11., theme::danger())),
                        ),
                    ),
            );
        }

        table.child(
            div()
                .flex()
                .child(port_cell(&self.new_local, false))
                .child(port_cell(&self.new_remote, false))
                .child(cell().w(px(40.))),
        )
    }
}

fn kind_value(kind: WorkloadType) -> &'static str {
    match kind {
        WorkloadType::Pod => "pod",
        WorkloadType::Deployment => "deployment",
        WorkloadType::Service => "service",
    }
}

fn kind_from_value(value: &str) -> Option<WorkloadType> {
    WorkloadType::ALL
        .into_iter()
        .find(|kind| kind_value(*kind) == value)
}

impl Render for ServiceFormPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (_, errors) = self.service(cx);
        let (_, row_validity) = self.forwards(cx);
        let valid = errors.is_empty();
        for (field, input) in [
            ("namespace", self.namespace.clone()),
            ("workloadName", self.name.clone()),
        ] {
            let invalid =
                self.touched.contains(field) && validate::error_for(&errors, field).is_some();
            input.update(cx, |input, cx| input.set_invalid(invalid, cx));
        }

        let back = navigate_on_click(cx, Route::Clusters);
        let logo = back.clone();
        let cancel = back.clone();
        let editing = matches!(self.mode, ServiceFormMode::Edit { .. });
        let half = |content: gpui::Div| div().w_1_2().px(px(15.)).mb(px(30.)).child(content);

        div()
            .p(px(20.))
            .flex()
            .flex_col()
            .child(ui::header(Some(back), logo, div()))
            .when(self.cluster_created, |this| {
                this.child(ui::alert_success(
                    "We successfully added your cluster. Add your first resource by filling the form below.",
                ))
            })
            .child(
                div()
                    .py(px(20.))
                    .px(px(5.))
                    .rounded(px(RADIUS))
                    .bg(theme::bg_block())
                    .flex()
                    .flex_wrap()
                    .child(half(ui::control_group(
                        "Cluster Name",
                        None,
                        self.field_error(&errors, "clusterId"),
                        false,
                        Select::new(&self.cluster_select).placeholder("Select...").h(px(40.)),
                    )))
                    .child(half(ui::control_group(
                        "Namespace",
                        None,
                        self.field_error(&errors, "namespace"),
                        false,
                        self.namespace.clone(),
                    )))
                    .child(half(ui::control_group(
                        "Kind",
                        None,
                        self.field_error(&errors, "workloadType"),
                        false,
                        Select::new(&self.kind_select)
                            .placeholder("Select Workload Type")
                            .h(px(40.)),
                    )))
                    .child(half(ui::control_group(
                        "Name",
                        None,
                        self.field_error(&errors, "workloadName"),
                        self.kind.is_none(),
                        self.name.clone(),
                    )))
                    .child(half(ui::control_group(
                        "Alias",
                        None,
                        None,
                        false,
                        ui::text_input(&self.alias, 40.),
                    )))
                    .child(div().w_1_2())
                    .child(
                        div().w_full().px(px(15.)).mb(px(30.)).child(ui::control_group(
                            "Ports Forwarding",
                            None,
                            self.field_error(&errors, "forwards"),
                            false,
                            self.render_ports(&row_validity, cx),
                        )),
                    )
                    .child(
                        div().w_full().px(px(15.)).child(ui::checkbox(
                            "use-local-address",
                            "Use custom local address",
                            self.use_local_address,
                            cx.listener(|this, _, _, cx| {
                                this.use_local_address = !this.use_local_address;
                                cx.notify();
                            }),
                        )),
                    )
                    .when(self.use_local_address, |this| {
                        this.child(
                            div().w_full().px(px(15.)).mt(px(10.)).child(ui::control_group(
                                "",
                                None,
                                self.field_error(&errors, "localAddress"),
                                false,
                                ui::text_input(&self.local_address, 40.),
                            )),
                        )
                    })
                    .child(
                        div()
                            .w_full()
                            .px(px(15.))
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
                            .when_some(self.error.clone(), |this, error| {
                                this.child(
                                    div()
                                        .px(px(10.))
                                        .text_size(px(FONT_SMALL))
                                        .text_color(theme::danger())
                                        .child(error),
                                )
                            })
                            .child(
                                KfButton::new("submit-service")
                                    .label(if editing { "Save" } else { "Add a resource" })
                                    .primary()
                                    .dimmed(!valid)
                                    .on_click(cx.listener(|this, _, window, cx| this.submit(window, cx))),
                            ),
                    ),
            )
    }
}

#[cfg(test)]
mod tests {
    use gpui::TestAppContext;
    use kf_core::model::{AppState, Cluster, ClusterConfig};

    use super::*;
    use crate::test_support::{click, setup, type_into, value};
    use crate::views::clusters::ClustersPage;

    fn state() -> AppState {
        AppState {
            clusters: vec![Cluster {
                id: "c1".into(),
                name: "local".into(),
                folded: false,
                config: ClusterConfig::from_content("apiVersion: v1\n", None),
            }],
            ..AppState::default()
        }
    }

    fn new_resource() -> Route {
        Route::ServiceNew {
            cluster_id: Some("c1".into()),
            cluster_created: true,
        }
    }

    #[gpui::test]
    fn ports_are_mirrored_until_edited(cx: &mut TestAppContext) {
        let (harness, cx) = setup(cx, state());
        harness.navigate(cx, new_resource());
        let page = harness.page::<ServiceFormPage>(cx);
        let (local, remote) = page.read_with(cx, |page, _| {
            (page.new_local.clone(), page.new_remote.clone())
        });

        type_into(cx, &local, "8080");
        assert_eq!(value(cx, &remote), "8080");

        // Editing the resource port stops the mirroring.
        remote.update_in(cx, |input, window, cx| input.focus(window, cx));
        cx.simulate_keystrokes("cmd-a");
        cx.simulate_input("80");
        cx.run_until_parked();
        assert_eq!(value(cx, &remote), "80");
        assert_eq!(value(cx, &local), "8080");

        // Enter adds the row and clears the inputs.
        cx.simulate_keystrokes("enter");
        cx.run_until_parked();
        page.read_with(cx, |page, cx| {
            assert_eq!(page.rows.len(), 1);
            assert_eq!(page.rows[0].local.read(cx).value(), "8080");
            assert_eq!(page.rows[0].remote.read(cx).value(), "80");
        });
        assert_eq!(value(cx, &local), "");
        assert_eq!(value(cx, &remote), "");

        // The next row mirrors again, in both directions.
        type_into(cx, &remote, "443");
        assert_eq!(value(cx, &local), "443");
    }

    #[gpui::test]
    fn creates_a_resource(cx: &mut TestAppContext) {
        let (harness, cx) = setup(cx, state());
        harness.navigate(cx, new_resource());
        let page = harness.page::<ServiceFormPage>(cx);

        // Incomplete: nothing is saved and an error is shown.
        click(cx, "button-submit-service");
        assert!(harness.state(cx).services.is_empty());
        assert!(page.read_with(cx, |page, _| page.error.is_some()));

        let (namespace, name, kind, local, alias) = page.read_with(cx, |page, cx| {
            (
                page.namespace.read(cx).input.clone(),
                page.name.read(cx).input.clone(),
                page.kind_select.clone(),
                page.new_local.clone(),
                page.alias.clone(),
            )
        });
        type_into(cx, &namespace, "kf-e2e");
        // Choosing "Deployment" in the dropdown.
        kind.update(cx, |_, cx| {
            cx.emit(SelectEvent::Confirm(Some("deployment".to_string())))
        });
        type_into(cx, &name, "web");
        type_into(cx, &alias, "frontend");
        type_into(cx, &local, "18080");
        click(cx, "button-submit-service");

        let state = harness.state(cx);
        assert_eq!(
            state.services.len(),
            1,
            "{:?}",
            page.read_with(cx, |page, _| page.error.clone())
        );
        let service = &state.services[0];
        assert_eq!(service.cluster_id, "c1");
        assert_eq!(service.namespace, "kf-e2e");
        assert_eq!(service.workload_type, WorkloadType::Deployment);
        assert_eq!(service.workload_name, "web");
        assert_eq!(service.alias.as_deref(), Some("frontend"));
        assert_eq!(service.forwards.len(), 1);
        assert_eq!(
            (
                service.forwards[0].local_port,
                service.forwards[0].remote_port
            ),
            (18080, 18080)
        );
        harness.page::<ClustersPage>(cx);
    }

    #[gpui::test]
    fn long_suggestion_lists_scroll(cx: &mut TestAppContext) {
        let (harness, cx) = setup(cx, state());
        // No cluster selected: no namespaces are requested from a cluster.
        harness.navigate(
            cx,
            Route::ServiceNew {
                cluster_id: None,
                cluster_created: false,
            },
        );
        let page = harness.page::<ServiceFormPage>(cx);
        let namespace = page.read_with(cx, |page, _| page.namespace.clone());
        namespace.update(cx, |namespace, cx| {
            namespace.set_options((0..40).map(|ix| format!("namespace-{ix:02}")).collect(), cx)
        });
        let input = namespace.read_with(cx, |namespace, _| namespace.input.clone());
        type_into(cx, &input, "");
        assert!(namespace.read_with(cx, |namespace, _| namespace.is_open()));

        let list = crate::test_support::bounds(cx, "autocomplete-options");
        assert_eq!(
            list.size.height,
            gpui::px(200.),
            "the list is limited to 200px"
        );
        let before = crate::test_support::bounds(cx, "autocomplete-option-0");
        crate::test_support::scroll_down_at(cx, list.center(), 100.);
        let after = crate::test_support::bounds(cx, "autocomplete-option-0");
        assert!(
            after.origin.y < before.origin.y,
            "the suggestions didn't scroll"
        );
    }

    #[gpui::test]
    fn edits_a_resource(cx: &mut TestAppContext) {
        let mut state = state();
        state.services.push(Service {
            id: "s1".into(),
            cluster_id: "c1".into(),
            alias: None,
            namespace: "default".into(),
            workload_type: WorkloadType::Pod,
            workload_name: "db-0".into(),
            forwards: vec![Forward::new(5432, 5432)],
            local_address: Some("127.0.0.1".into()),
        });
        let (harness, cx) = setup(cx, state);
        harness.navigate(cx, Route::ServiceEdit { id: "s1".into() });
        let page = harness.page::<ServiceFormPage>(cx);
        let (row_local, address) = page.read_with(cx, |page, _| {
            (page.rows[0].local.clone(), page.local_address.clone())
        });
        assert_eq!(value(cx, &address), "127.0.0.1");

        row_local.update_in(cx, |input, window, cx| input.focus(window, cx));
        cx.simulate_keystrokes("cmd-a");
        cx.simulate_input("15432");
        // Turning the custom address off removes it.
        click(cx, "choice-use-local-address");
        click(cx, "button-submit-service");

        let service = harness.state(cx).services[0].clone();
        assert_eq!(service.id, "s1");
        assert_eq!(service.forwards[0].local_port, 15432);
        assert_eq!(service.local_address, None);
    }
}

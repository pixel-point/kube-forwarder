//! Kube Forwarder: an easy to use Kubernetes port forwarding manager, written with GPUI.

mod assets;
mod dialogs;
mod menus;
mod store;
#[cfg(test)]
mod test_support;
mod theme;
mod ui;
mod views;

use gpui::{
    App, AppContext, Application, Bounds, Entity, TitlebarOptions, WindowBounds, WindowOptions,
    point, px, size,
};
use gpui_component::Root;
use kf_core::storage::default_storage;

use crate::store::{AppStore, Tokio};
use crate::views::MainView;

struct MainStore(Entity<AppStore>);

impl gpui::Global for MainStore {}

fn main() {
    // Applications started from Finder get a minimal environment; credential plugins of
    // kube configs (aws, gcloud, kubelogin, ...) need the PATH of the login shell.
    // SAFETY: no other thread has been started yet.
    let imported = unsafe { kf_core::shell_env::import_login_shell_env() };

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_env("KUBE_FORWARDER_LOG")
                .unwrap_or_else(|_| "kube_forwarder=info,kf_core=info,warn".into()),
        )
        .with_writer(std::io::stderr)
        .init();
    if imported > 0 {
        tracing::info!(
            variables = imported,
            "imported the environment of the login shell"
        );
    }
    tracing::debug!(path = %std::env::var("PATH").unwrap_or_default(), "PATH");

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .thread_name("kube-forwarder-io")
        .enable_all()
        .build()
        .expect("failed to start the Tokio runtime");
    let tokio = Tokio(runtime.handle().clone());

    let app = Application::new().with_assets(assets::Assets);
    // Like the original app on macOS: closing the window keeps forwarding running,
    // clicking the Dock icon opens it again.
    app.on_reopen(|cx| {
        if cx.windows().is_empty()
            && let Some(store) = cx.try_global::<MainStore>().map(|store| store.0.clone())
        {
            open_main_window(store, cx);
        }
    });
    app.run(move |cx: &mut App| {
        cx.set_global(tokio);
        gpui_component::init(cx);
        theme::init(cx);
        menus::init(cx);

        let storage = default_storage();
        tracing::info!(path = %storage.path().display(), "state file");
        let store = cx.new(|cx| AppStore::new(storage, cx));
        cx.set_global(MainStore(store.clone()));

        cx.on_app_quit({
            let store = store.clone();
            move |cx| {
                store.update(cx, |store, _| store.stop_all());
                async {}
            }
        })
        .detach();

        cx.on_window_closed(|cx| {
            if cx.windows().is_empty() && !cfg!(target_os = "macos") {
                cx.quit();
            }
        })
        .detach();

        open_main_window(store, cx);
        cx.activate(true);
    });

    drop(runtime);
}

fn open_main_window(store: Entity<AppStore>, cx: &mut App) {
    let bounds = Bounds::centered(None, size(px(800.), px(580.)), cx);
    let options = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        titlebar: Some(TitlebarOptions {
            title: Some("Kube Forwarder".into()),
            appears_transparent: true,
            traffic_light_position: Some(point(px(13.), px(12.))),
        }),
        window_min_size: Some(size(px(720.), px(480.))),
        app_id: Some("com.pixelpoint.kube-forwarder".into()),
        ..Default::default()
    };
    let result = cx.open_window(options, |window, cx| {
        let view = cx.new(|cx| MainView::new(store, window, cx));
        cx.new(|cx| Root::new(view, window, cx))
    });
    if let Err(error) = result {
        tracing::error!(%error, "failed to open the main window");
    }
}

//! Helpers for UI tests: a window with the real views, a Tokio runtime and isolated storage.

use std::time::Duration;

use gpui::{
    AppContext, Bounds, Entity, Modifiers, MouseMoveEvent, Pixels, ScrollDelta, ScrollWheelEvent,
    TestAppContext, TouchPhase, VisualTestContext, point, px,
};
use gpui_component::Root;
use gpui_component::input::InputState;
use kf_core::model::AppState;
use kf_core::storage::Storage;

use crate::store::{AppStore, Tokio};
use crate::views::{MainView, Route};

pub struct Harness {
    pub store: Entity<AppStore>,
    pub main: Entity<MainView>,
    _runtime: tokio::runtime::Runtime,
    _dir: tempfile::TempDir,
}

pub fn setup(cx: &mut TestAppContext, state: AppState) -> (Harness, &mut VisualTestContext) {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .unwrap();
    let dir = tempfile::tempdir().unwrap();
    let storage = Storage::without_legacy_import(dir.path().join("state.json"));
    storage.save(&state).unwrap();

    let handle = runtime.handle().clone();
    cx.update(|cx| {
        cx.set_global(Tokio(handle));
        gpui_component::init(cx);
        crate::theme::init(cx);
    });
    let store = cx.new(|cx| AppStore::new(storage, cx));
    let mut main = None;
    let (_, cx) = cx.add_window_view(|window, cx| {
        let view = cx.new(|cx| MainView::new(store.clone(), window, cx));
        main = Some(view.clone());
        Root::new(view, window, cx)
    });
    // Focus events are only delivered to an active window (like a window the user clicks).
    cx.update(|window, _| window.activate_window());
    cx.run_until_parked();
    let harness = Harness {
        store,
        main: main.unwrap(),
        _runtime: runtime,
        _dir: dir,
    };
    (harness, cx)
}

impl Harness {
    pub fn navigate(&self, cx: &mut VisualTestContext, route: Route) {
        self.main
            .update_in(cx, |main, window, cx| main.navigate(route, window, cx));
        cx.run_until_parked();
    }

    /// The entity of the current page.
    pub fn page<T: 'static>(&self, cx: &mut VisualTestContext) -> Entity<T> {
        self.main
            .read_with(cx, |main, _| main.page())
            .downcast::<T>()
            .unwrap_or_else(|_| panic!("the current page is not a {}", std::any::type_name::<T>()))
    }

    pub fn state(&self, cx: &mut VisualTestContext) -> AppState {
        self.store.read_with(cx, |store, _| store.state.clone())
    }
}

/// Clicks the center of an element marked with `debug_selector`.
pub fn click(cx: &mut VisualTestContext, selector: &str) {
    cx.run_until_parked();
    let selector: &'static str = Box::leak(selector.to_string().into_boxed_str());
    let bounds = cx
        .debug_bounds(selector)
        .unwrap_or_else(|| panic!("`{selector}` is not rendered"));
    cx.simulate_click(bounds.center(), Modifiers::none());
    cx.run_until_parked();
}

pub fn is_rendered(cx: &mut VisualTestContext, selector: &str) -> bool {
    cx.run_until_parked();
    let selector: &'static str = Box::leak(selector.to_string().into_boxed_str());
    cx.debug_bounds(selector).is_some()
}

/// Focuses an input and types into it like a user.
pub fn type_into(cx: &mut VisualTestContext, input: &Entity<InputState>, text: &str) {
    input.update_in(cx, |input, window, cx| input.focus(window, cx));
    cx.run_until_parked();
    cx.simulate_input(text);
    cx.run_until_parked();
}

pub fn value(cx: &mut VisualTestContext, input: &Entity<InputState>) -> String {
    input.read_with(cx, |input, _| input.value().to_string())
}

/// Waits for work done on the Tokio runtime (Kubernetes requests, port forwarding).
pub fn wait_until(
    cx: &mut VisualTestContext,
    mut condition: impl FnMut(&mut VisualTestContext) -> bool,
) {
    cx.executor().allow_parking();
    for _ in 0..400 {
        cx.run_until_parked();
        if condition(cx) {
            return;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    panic!("the condition was not met in time");
}

/// Bounds of an element marked with `debug_selector` in the last frame.
pub fn bounds(cx: &mut VisualTestContext, selector: &str) -> Bounds<Pixels> {
    cx.run_until_parked();
    let selector: &'static str = Box::leak(selector.to_string().into_boxed_str());
    cx.debug_bounds(selector)
        .unwrap_or_else(|| panic!("`{selector}` is not rendered"))
}

/// Scrolls with the mouse wheel (or the trackpad) over the middle of the window.
pub fn scroll_down(cx: &mut VisualTestContext, distance: f32) {
    scroll_down_at(cx, point(px(400.), px(300.)), distance);
}

pub fn scroll_down_at(cx: &mut VisualTestContext, position: gpui::Point<Pixels>, distance: f32) {
    cx.simulate_event(MouseMoveEvent {
        position,
        pressed_button: None,
        modifiers: Modifiers::none(),
    });
    cx.simulate_event(ScrollWheelEvent {
        position,
        delta: ScrollDelta::Pixels(point(px(0.), px(-distance))),
        modifiers: Modifiers::none(),
        touch_phase: TouchPhase::Moved,
    });
    cx.run_until_parked();
}

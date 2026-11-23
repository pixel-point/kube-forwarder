//! Application menu (`menuTemplate.js`).

use gpui::{App, KeyBinding, Menu, MenuItem, SystemMenuType, actions};
use gpui_component::input;

use crate::dialogs;
use crate::views::VERSION;

actions!(
    kube_forwarder,
    [
        About,
        CheckForUpdates,
        Hide,
        HideOthers,
        ShowAll,
        Quit,
        Minimize,
        Zoom,
        CloseWindow,
        LearnMore,
    ]
);

const WEBSITE: &str = "https://kube-forwarder.pixelpoint.io/";
const RELEASES: &str = "https://github.com/pixel-point/kube-forwarder/releases/latest";

pub fn init(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("cmd-q", Quit, None),
        KeyBinding::new("cmd-h", Hide, None),
        KeyBinding::new("alt-cmd-h", HideOthers, None),
        KeyBinding::new("cmd-m", Minimize, None),
        KeyBinding::new("cmd-w", CloseWindow, None),
    ]);

    cx.on_action(|_: &Quit, cx| cx.quit());
    cx.on_action(|_: &Hide, cx| cx.hide());
    cx.on_action(|_: &HideOthers, cx| cx.hide_other_apps());
    cx.on_action(|_: &ShowAll, cx| cx.unhide_other_apps());
    cx.on_action(|_: &LearnMore, cx| cx.open_url(WEBSITE));
    cx.on_action(|_: &CheckForUpdates, cx| cx.open_url(RELEASES));
    cx.on_action(|_: &About, cx| {
        with_active_window(cx, |window, cx| {
            dialogs::show_message(
                window,
                cx,
                "Kube Forwarder",
                Some(&format!(
                    "Version {VERSION}\nEasy to use Kubernetes port forwarding manager.\n\n{WEBSITE}"
                )),
            )
        })
    });
    cx.on_action(|_: &Minimize, cx| with_active_window(cx, |window, _| window.minimize_window()));
    cx.on_action(|_: &Zoom, cx| with_active_window(cx, |window, _| window.zoom_window()));
    cx.on_action(|_: &CloseWindow, cx| with_active_window(cx, |window, _| window.remove_window()));

    cx.set_menus(vec![
        Menu {
            name: "Kube Forwarder".into(),
            items: vec![
                MenuItem::action("About Kube Forwarder", About),
                MenuItem::action("Check for Updates...", CheckForUpdates),
                MenuItem::separator(),
                MenuItem::os_submenu("Services", SystemMenuType::Services),
                MenuItem::separator(),
                MenuItem::action("Hide Kube Forwarder", Hide),
                MenuItem::action("Hide Others", HideOthers),
                MenuItem::action("Show All", ShowAll),
                MenuItem::separator(),
                MenuItem::action("Quit Kube Forwarder", Quit),
            ],
        },
        Menu {
            name: "Edit".into(),
            items: vec![
                MenuItem::action("Undo", input::Undo),
                MenuItem::action("Redo", input::Redo),
                MenuItem::separator(),
                MenuItem::action("Cut", input::Cut),
                MenuItem::action("Copy", input::Copy),
                MenuItem::action("Paste", input::Paste),
                MenuItem::action("Select All", input::SelectAll),
            ],
        },
        Menu {
            name: "Window".into(),
            items: vec![
                MenuItem::action("Minimize", Minimize),
                MenuItem::action("Zoom", Zoom),
                MenuItem::separator(),
                MenuItem::action("Close Window", CloseWindow),
            ],
        },
        Menu {
            name: "Help".into(),
            items: vec![MenuItem::action("Learn More", LearnMore)],
        },
    ]);
}

fn with_active_window(cx: &mut App, f: impl FnOnce(&mut gpui::Window, &mut App)) {
    if let Some(window) = cx.active_window() {
        window.update(cx, |_, window, cx| f(window, cx)).ok();
    }
}

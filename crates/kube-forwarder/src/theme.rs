//! Colors and sizes of the original design (`_variables.scss`).

use gpui::{App, Hsla, Rgba, px, rgb};
use gpui_component::{Theme, ThemeMode};

pub const FONT_SMALL: f32 = 13.;
pub const FONT_BASE: f32 = 14.;
pub const FONT_BIG: f32 = 15.;
pub const FONT_LARGE: f32 = 17.;

pub const TOOLBAR_HEIGHT: f32 = 37.;
pub const RADIUS: f32 = 2.;

fn hsla(color: Rgba) -> Hsla {
    color.into()
}

pub fn with_alpha(color: Hsla, alpha: f32) -> Hsla {
    Hsla { a: alpha, ..color }
}

/// Mixes a color with black like `mix($color, #000, 90%)`.
pub fn darken(color: Hsla) -> Hsla {
    Hsla {
        l: color.l * 0.9,
        ..color
    }
}

pub fn white() -> Hsla {
    gpui::white()
}

pub fn text() -> Hsla {
    hsla(rgb(0x142d55))
}

pub fn text_secondary() -> Hsla {
    with_alpha(text(), 0.75)
}

pub fn text_tertiary() -> Hsla {
    with_alpha(text(), 0.5)
}

pub fn text_quaternary() -> Hsla {
    with_alpha(text(), 0.25)
}

pub fn primary() -> Hsla {
    hsla(rgb(0x3273e1))
}

pub fn secondary() -> Hsla {
    hsla(rgb(0x1ecde1))
}

pub fn warning() -> Hsla {
    hsla(rgb(0xff7d00))
}

pub fn danger() -> Hsla {
    hsla(rgb(0xeb5569))
}

pub fn success() -> Hsla {
    hsla(rgb(0x19d78c))
}

pub fn border() -> Hsla {
    with_alpha(text(), 0.1)
}

pub fn input_border() -> Hsla {
    with_alpha(text(), 0.15)
}

pub fn bg_secondary() -> Hsla {
    with_alpha(text(), 0.02)
}

/// Background of form blocks (`rgba($color-text, 0.04)`).
pub fn bg_block() -> Hsla {
    with_alpha(text(), 0.04)
}

pub fn bg_danger() -> Hsla {
    with_alpha(danger(), 0.15)
}

/// `mix($color-text, #fff, 15%)`
pub fn table_border() -> Hsla {
    hsla(rgb(0xdcdfe5))
}

/// Configures gpui-component widgets (inputs, selects, menus, dialogs) to match.
pub fn init(cx: &mut App) {
    Theme::change(ThemeMode::Light, None, cx);
    let theme = Theme::global_mut(cx);
    theme.radius = px(RADIUS);
    theme.radius_lg = px(8.);
    theme.shadow = false;

    theme.background = white();
    theme.foreground = text();
    theme.border = border();
    theme.input = input_border();
    theme.ring = primary();
    theme.caret = text();
    theme.selection = with_alpha(primary(), 0.2);
    theme.muted = bg_block();
    theme.muted_foreground = text_secondary();

    theme.primary = primary();
    theme.primary_hover = darken(primary());
    theme.primary_active = darken(primary());
    theme.primary_foreground = white();
    theme.danger = danger();
    theme.danger_hover = darken(danger());
    theme.danger_active = darken(danger());
    theme.danger_foreground = white();
    theme.success = success();
    theme.success_foreground = white();
    theme.warning = warning();
    theme.warning_foreground = white();

    theme.secondary = bg_block();
    theme.secondary_hover = with_alpha(text(), 0.08);
    theme.secondary_active = with_alpha(text(), 0.1);
    theme.secondary_foreground = text();

    theme.accent = bg_secondary();
    theme.accent_foreground = text();
    theme.popover = white();
    theme.popover_foreground = text();
    theme.list_hover = with_alpha(text(), 0.04);
    theme.list_active = with_alpha(primary(), 0.1);
    theme.list_active_border = with_alpha(primary(), 0.4);

    theme.link = primary();
    theme.link_hover = darken(primary());
    theme.link_active = darken(primary());
    theme.overlay = with_alpha(hsla(rgb(0x0d1325)), 0.5);
    theme.title_bar = white();
    theme.title_bar_border = border();
}

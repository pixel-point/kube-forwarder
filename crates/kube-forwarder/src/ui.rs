//! Small building blocks that reproduce the look of the original application.

use std::rc::Rc;

use gpui::prelude::FluentBuilder;
use gpui::{
    App, ClickEvent, CursorStyle, Div, ElementId, FontWeight, Hsla, InteractiveElement,
    IntoElement, ParentElement, RenderOnce, SharedString, StatefulInteractiveElement, Styled,
    Window, div, img, px, relative, svg,
};
use gpui_component::spinner::Spinner;
use gpui_component::tooltip::Tooltip;
use gpui_component::{Sizable, Size};

use crate::theme::{self, FONT_LARGE, FONT_SMALL, RADIUS};

pub type ClickHandler = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>;

/// A single line text input of the given height. (`Input::h` only applies to multi line
/// inputs; the style refinement is applied last, so it overrides the size based height.)
pub fn text_input(
    state: &gpui::Entity<gpui_component::input::InputState>,
    height: f32,
) -> gpui_component::input::Input {
    Styled::h(gpui_component::input::Input::new(state), px(height))
}

/// A monochrome icon from `assets/icons`.
pub fn icon(name: &str, width: f32, height: f32, color: Hsla) -> gpui::Svg {
    svg()
        .path(SharedString::from(format!("icons/{name}.svg")))
        .w(px(width))
        .h(px(height))
        .flex_none()
        .text_color(color)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    Default,
    Primary,
    Danger,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Variant {
    Filled,
    Outline,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonSize {
    Small,
    Medium,
    Large,
}

/// The button of the original application (uppercase label, three sizes, filled,
/// outline and text variants).
#[derive(IntoElement)]
pub struct KfButton {
    id: ElementId,
    label: Option<SharedString>,
    icon: Option<(&'static str, f32, f32)>,
    tone: Tone,
    variant: Variant,
    size: ButtonSize,
    width: Option<f32>,
    disabled: bool,
    dimmed: bool,
    loading: bool,
    cancelable: bool,
    tooltip: Option<SharedString>,
    on_click: Option<ClickHandler>,
}

impl KfButton {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            label: None,
            icon: None,
            tone: Tone::Default,
            variant: Variant::Filled,
            size: ButtonSize::Medium,
            width: None,
            disabled: false,
            dimmed: false,
            loading: false,
            cancelable: false,
            tooltip: None,
            on_click: None,
        }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn icon(mut self, name: &'static str, width: f32, height: f32) -> Self {
        self.icon = Some((name, width, height));
        self
    }

    pub fn primary(mut self) -> Self {
        self.tone = Tone::Primary;
        self
    }

    pub fn danger(mut self) -> Self {
        self.tone = Tone::Danger;
        self
    }

    pub fn outline(mut self) -> Self {
        self.variant = Variant::Outline;
        self
    }

    pub fn variant(mut self, variant: Variant) -> Self {
        self.variant = variant;
        self
    }

    pub fn small(mut self) -> Self {
        self.size = ButtonSize::Small;
        self
    }

    pub fn large(mut self) -> Self {
        self.size = ButtonSize::Large;
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Not clickable and dimmed.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Dimmed but still clickable (`disabledStyle` of the original).
    pub fn dimmed(mut self, dimmed: bool) -> Self {
        self.dimmed = dimmed;
        self
    }

    pub fn loading(mut self, loading: bool) -> Self {
        self.loading = loading;
        self
    }

    /// Keeps the button clickable while it's loading (to cancel the operation).
    pub fn cancelable(mut self, cancelable: bool) -> Self {
        self.cancelable = cancelable;
        self
    }

    pub fn tooltip(mut self, tooltip: impl Into<SharedString>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }

    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for KfButton {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let (height, padding, font_size) = match self.size {
            ButtonSize::Small => (30., 10., FONT_SMALL),
            ButtonSize::Medium => (36., 15., FONT_SMALL),
            ButtonSize::Large => (50., 25., FONT_LARGE),
        };
        let color = match self.tone {
            Tone::Default => theme::text_quaternary(),
            Tone::Primary => theme::primary(),
            Tone::Danger => theme::danger(),
        };
        let is_default = self.tone == Tone::Default;
        let content_color = match self.variant {
            Variant::Filled => theme::white(),
            _ => color,
        };

        let interactive = !self.disabled && (!self.loading || self.cancelable);
        let selector = format!("button-{}", self.id);
        let mut button = div()
            .id(self.id)
            .debug_selector(|| selector)
            .flex()
            .flex_none()
            .items_center()
            .justify_center()
            .gap(px(7.))
            .h(px(height))
            .when_some(self.width, |this, width| this.w(px(width)))
            .when(self.width.is_none(), |this| {
                this.px(if self.label.is_some() {
                    px(padding)
                } else {
                    px(0.)
                })
                .min_w(px(height))
            })
            .rounded(px(RADIUS))
            .border_1()
            .text_size(px(font_size))
            .font_weight(FontWeight::MEDIUM)
            .text_color(content_color);

        button = match self.variant {
            Variant::Filled => button
                .bg(color)
                .border_color(gpui::transparent_black())
                .when(interactive, |this| {
                    this.hover(move |style| style.bg(theme::darken(color)))
                }),
            Variant::Outline => button
                .border_color(if is_default {
                    theme::border()
                } else {
                    theme::with_alpha(color, 0.5)
                })
                .when(interactive, |this| {
                    this.hover(move |style| {
                        if is_default {
                            style
                                .text_color(theme::text_secondary())
                                .border_color(theme::text_tertiary())
                        } else {
                            style.bg(theme::with_alpha(color, 0.1))
                        }
                    })
                }),
        };

        if self.disabled || self.dimmed {
            button = button.opacity(0.25);
        }

        if self.loading {
            button = button.child(Spinner::new().color(content_color).with_size(Size::Small));
        } else {
            if let Some((name, width, height)) = self.icon {
                button = button.child(icon(name, width, height, content_color));
            }
            if let Some(label) = self.label {
                button = button.child(SharedString::from(label.to_uppercase()));
            }
        }

        if let Some(tooltip) = self.tooltip {
            button =
                button.tooltip(move |window, cx| Tooltip::new(tooltip.clone()).build(window, cx));
        }

        match (interactive, self.on_click) {
            (true, Some(handler)) => button
                .cursor(CursorStyle::PointingHand)
                .on_click(move |event, window, cx| handler(event, window, cx)),
            (false, _) if self.disabled => button.cursor(CursorStyle::OperationNotAllowed),
            _ => button,
        }
    }
}

/// The header of a page: back arrow, custom content and the centered logo.
pub fn header(
    on_back: Option<ClickHandler>,
    on_logo: ClickHandler,
    content: impl IntoElement,
) -> impl IntoElement {
    div()
        .relative()
        .flex()
        .items_center()
        .h(px(36.))
        .w_full()
        .mb(px(20.))
        .when_some(on_back, |this, on_back| {
            // `svg()` doesn't inherit the text color, the hover color is set with a group.
            this.child(
                div()
                    .id("header-back")
                    .group("header-back")
                    .cursor(CursorStyle::PointingHand)
                    .mr(px(20.))
                    .on_click(move |event, window, cx| on_back(event, window, cx))
                    .child(
                        icon("kf-arrow-back", 25., 18., theme::text_quaternary())
                            .group_hover("header-back", |style| {
                                style.text_color(theme::text_secondary())
                            }),
                    ),
            )
        })
        .child(div().flex_1().flex().items_center().child(content))
        .child(
            div()
                .id("header-logo")
                .absolute()
                .left(relative(0.5))
                .ml(px(-24.))
                .top(px(-6.))
                .cursor(CursorStyle::PointingHand)
                .on_click(move |event, window, cx| on_logo(event, window, cx))
                .child(img("images/logo.svg").size(px(48.))),
        )
}

/// A form field with a label, an optional hint and a validation error.
pub fn control_group(
    label: impl Into<SharedString>,
    hint: Option<&'static str>,
    error: Option<SharedString>,
    disabled: bool,
    control: impl IntoElement,
) -> Div {
    let label: SharedString = label.into();
    div()
        .flex()
        .flex_col()
        .w_full()
        .when(!label.is_empty() || hint.is_some(), |this| {
            this.child(
                div()
                    .flex()
                    .items_end()
                    .mb(px(4.))
                    .child(
                        div()
                            .text_size(px(FONT_SMALL))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(if disabled {
                                theme::text_quaternary()
                            } else {
                                theme::text_tertiary()
                            })
                            .child(label),
                    )
                    .child(div().flex_1())
                    .when_some(hint, |this, hint| {
                        this.child(
                            div()
                                .text_size(px(FONT_SMALL))
                                .italic()
                                .text_color(theme::text_tertiary())
                                .child(hint),
                        )
                    }),
            )
        })
        .child(
            div()
                .relative()
                .w_full()
                .child(control)
                .when_some(error, |this, error| {
                    this.child(
                        div()
                            .absolute()
                            .top_full()
                            .left_0()
                            .pt(px(2.))
                            .text_size(px(FONT_SMALL))
                            .text_color(theme::danger())
                            .child(error),
                    )
                }),
        )
}

/// A grey block (`.page__block`, `.base-form`).
pub fn block() -> Div {
    div().p(px(20.)).rounded(px(RADIUS)).bg(theme::bg_block())
}

/// A colored banner (`Alert.vue`).
pub fn alert_success(message: impl Into<SharedString>) -> impl IntoElement {
    div()
        .w_full()
        .mb(px(20.))
        .px(px(15.))
        .py(px(10.))
        .rounded(px(RADIUS))
        .bg(theme::success())
        .text_color(theme::white())
        .font_weight(FontWeight::SEMIBOLD)
        .flex()
        .justify_center()
        .child(message.into())
}

/// A row of radio buttons styled like the original (filled circle with a check mark).
pub fn radio(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    checked: bool,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    choice_control(id, label.into(), checked, true, on_click)
}

/// A checkbox styled like the original.
pub fn checkbox(
    id: impl Into<ElementId>,
    label: impl IntoElement,
    checked: bool,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    choice_control(id, label, checked, false, on_click)
}

fn choice_control(
    id: impl Into<ElementId>,
    label: impl IntoElement,
    checked: bool,
    round: bool,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    let checked_color: Hsla = gpui::rgb(0x0564d7).into();
    let id: ElementId = id.into();
    let selector = format!("choice-{id}");
    div()
        .id(id)
        .debug_selector(|| selector)
        .group("choice")
        .flex()
        .items_center()
        .gap(px(9.))
        .cursor(CursorStyle::PointingHand)
        .on_click(on_click)
        .child(
            div()
                .flex()
                .flex_none()
                .items_center()
                .justify_center()
                .size(px(18.))
                .rounded(if round { px(9.) } else { px(3.) })
                .border_1()
                .map(|this| {
                    if checked {
                        this.bg(checked_color)
                            .border_color(gpui::transparent_black())
                    } else {
                        this.bg(theme::white())
                            .border_color(gpui::rgba(0x192d4640))
                            .group_hover("choice", move |style| style.border_color(checked_color))
                    }
                })
                .when(checked, |this| {
                    this.child(icon("check", 12., 12., theme::white()))
                }),
        )
        .child(div().line_height(px(20.)).child(label))
}

/// The status indicator of a resource.
pub fn status_dot(running: bool) -> impl IntoElement {
    div()
        .id(if running {
            "status-running"
        } else {
            "status-stopped"
        })
        .flex_none()
        .mt(px(5.))
        .size(px(10.))
        .rounded(px(5.))
        .bg(if running {
            theme::secondary()
        } else {
            theme::danger()
        })
        .tooltip(move |window, cx| {
            Tooltip::new(if running { "Running" } else { "Stopped" }).build(window, cx)
        })
}

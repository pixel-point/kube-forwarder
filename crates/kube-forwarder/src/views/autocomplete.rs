//! A text input with suggestions (`AutocompleteInput.vue`).

use std::time::Duration;

use gpui::prelude::FluentBuilder;
use gpui::{
    App, AppContext, Context, CursorStyle, Entity, EventEmitter, FontWeight, InteractiveElement,
    IntoElement, MouseButton, ParentElement, Render, SharedString, StatefulInteractiveElement,
    Styled, Subscription, Task, Window, deferred, div, px,
};
use gpui_component::input::{InputEvent, InputState};
use gpui_component::spinner::Spinner;
use gpui_component::{Sizable, Size};

use crate::theme::{self, FONT_BASE, RADIUS};
use crate::ui;

pub enum AutocompleteEvent {
    Focus,
    Change,
}

pub struct Autocomplete {
    pub input: Entity<InputState>,
    options: Vec<String>,
    loading: bool,
    open: bool,
    invalid: bool,
    close_task: Option<Task<()>>,
    _subscription: Subscription,
}

impl EventEmitter<AutocompleteEvent> for Autocomplete {}

impl Autocomplete {
    /// `value` is the initial value (set without emitting change events).
    pub fn new(
        placeholder: &str,
        value: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let placeholder = SharedString::from(placeholder.to_string());
        let value = SharedString::from(value.to_string());
        let input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder(placeholder)
                .default_value(value)
        });
        let subscription = cx.subscribe_in(&input, window, |this, _, event: &InputEvent, _, cx| {
            match event {
                InputEvent::Focus => {
                    this.close_task = None;
                    this.open = true;
                    cx.emit(AutocompleteEvent::Focus);
                }
                InputEvent::Blur => {
                    // Close a moment later, so that a click on a suggestion is handled first.
                    this.close_task = Some(cx.spawn(async move |this, cx| {
                        cx.background_executor()
                            .timer(Duration::from_millis(150))
                            .await;
                        this.update(cx, |this, cx| {
                            this.open = false;
                            cx.notify();
                        })
                        .ok();
                    }));
                }
                InputEvent::Change => {
                    cx.emit(AutocompleteEvent::Change);
                }
                InputEvent::PressEnter { .. } => this.open = false,
            }
            cx.notify();
        });
        Self {
            input,
            options: Vec::new(),
            loading: false,
            open: false,
            invalid: false,
            close_task: None,
            _subscription: subscription,
        }
    }

    pub fn value(&self, cx: &App) -> String {
        self.input.read(cx).value().trim().to_string()
    }

    pub fn set_value(&mut self, value: &str, window: &mut Window, cx: &mut Context<Self>) {
        let value = value.to_string();
        self.input
            .update(cx, |input, cx| input.set_value(value, window, cx));
    }

    pub fn set_options(&mut self, options: Vec<String>, cx: &mut Context<Self>) {
        self.options = options;
        cx.notify();
    }

    pub fn set_loading(&mut self, loading: bool, cx: &mut Context<Self>) {
        self.loading = loading;
        cx.notify();
    }

    #[cfg(test)]
    pub fn is_open(&self) -> bool {
        self.open
    }

    pub fn set_invalid(&mut self, invalid: bool, cx: &mut Context<Self>) {
        if self.invalid != invalid {
            self.invalid = invalid;
            cx.notify();
        }
    }

    /// Suggestions starting with the typed text first, then the ones containing it.
    fn matches(&self, cx: &App) -> Vec<String> {
        let value = self.value(cx).to_lowercase();
        let mut starts: Vec<String> = Vec::new();
        let mut contains: Vec<String> = Vec::new();
        for option in &self.options {
            let lower = option.to_lowercase();
            if lower.starts_with(&value) {
                starts.push(option.clone());
            } else if lower.contains(&value) {
                contains.push(option.clone());
            }
        }
        starts.extend(contains);
        starts
    }

    fn select(&mut self, option: String, window: &mut Window, cx: &mut Context<Self>) {
        self.set_value(&option, window, cx);
        self.open = false;
        cx.emit(AutocompleteEvent::Change);
        cx.notify();
    }
}

impl Render for Autocomplete {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let show_popup = self.open && (self.loading || !self.options.is_empty());
        let matches = if show_popup {
            self.matches(cx)
        } else {
            Vec::new()
        };

        let popup = div()
            .id("autocomplete-popup")
            .absolute()
            .top_full()
            .left_0()
            .right_0()
            .mt(px(5.))
            .bg(theme::white())
            .border_1()
            .border_color(gpui::rgb(0xedeef1))
            .rounded(px(RADIUS))
            .shadow_sm()
            .text_size(px(FONT_BASE))
            .map(|this| {
                if self.loading {
                    this.child(
                        div()
                            .h(px(40.))
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(
                                Spinner::new()
                                    .color(theme::primary())
                                    .with_size(Size::Small),
                            ),
                    )
                } else if matches.is_empty() {
                    this.child(
                        div()
                            .h(px(40.))
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_color(theme::text_secondary())
                            .child("Not found"),
                    )
                } else {
                    this.child(
                        div()
                            .id("autocomplete-options")
                            .debug_selector(|| "autocomplete-options".into())
                            .max_h(px(200.))
                            .overflow_y_scroll()
                            .children(matches.into_iter().enumerate().map(|(ix, option)| {
                                let value = option.clone();
                                div()
                                    .id(("option", ix))
                                    .debug_selector(move || format!("autocomplete-option-{ix}"))
                                    .h(px(36.))
                                    .px(px(15.))
                                    .flex()
                                    .items_center()
                                    .font_weight(FontWeight::MEDIUM)
                                    .cursor(CursorStyle::PointingHand)
                                    .when(ix > 0, |this| {
                                        this.border_t_1().border_color(theme::border())
                                    })
                                    .hover(|style| style.bg(theme::bg_secondary()))
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(move |this, _, window, cx| {
                                            cx.stop_propagation();
                                            this.select(value.clone(), window, cx);
                                        }),
                                    )
                                    .child(option)
                            })),
                    )
                }
            });

        div()
            .relative()
            .w_full()
            .child(
                div()
                    .rounded(px(RADIUS))
                    .when(self.invalid, |this| {
                        this.border_1().border_color(theme::danger())
                    })
                    .child(ui::text_input(&self.input, 40.)),
            )
            .when(show_popup, |this| {
                this.child(deferred(popup).with_priority(1))
            })
    }
}

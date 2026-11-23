//! Native alerts (like the message boxes of the original application).

use gpui::{App, PromptButton, PromptLevel, SharedString, Window};
use kf_core::KfError;

fn ok(label: &str) -> PromptButton {
    PromptButton::Ok(SharedString::from(label.to_string()))
}

fn cancel() -> PromptButton {
    PromptButton::Cancel("Cancel".into())
}

/// Shows an error with its details.
pub fn show_error(window: &mut Window, cx: &mut App, error: &KfError) {
    if error.is_cancelled() {
        return;
    }
    // The answer isn't needed: dropping the receiver keeps the alert open.
    drop(window.prompt(
        PromptLevel::Critical,
        &error.message,
        error.details.as_deref(),
        &[ok("OK")],
        cx,
    ));
}

pub fn show_message(window: &mut Window, cx: &mut App, message: &str, detail: Option<&str>) {
    drop(window.prompt(PromptLevel::Info, message, detail, &[ok("OK")], cx));
}

/// Asks for a confirmation. Resolves to `true` when `ok_label` was chosen.
pub fn confirm(
    window: &mut Window,
    cx: &mut App,
    message: &str,
    detail: Option<&str>,
    ok_label: &str,
) -> impl Future<Output = bool> + use<> {
    let answer = window.prompt(
        PromptLevel::Warning,
        message,
        detail,
        &[ok(ok_label), cancel()],
        cx,
    );
    async move { matches!(answer.await, Ok(0)) }
}

/// Asks a question. The first answer is the default one, a `Cancel` answer is added last.
/// Resolves to the index of the chosen answer (`None` for Cancel).
pub fn choose(
    window: &mut Window,
    cx: &mut App,
    message: &str,
    detail: Option<&str>,
    answers: &[&str],
) -> impl Future<Output = Option<usize>> + use<> {
    let mut buttons: Vec<PromptButton> = answers
        .iter()
        .enumerate()
        .map(|(ix, label)| {
            if ix == 0 {
                ok(label)
            } else {
                PromptButton::Other(SharedString::from(label.to_string()))
            }
        })
        .collect();
    let count = buttons.len();
    buttons.push(cancel());
    let answer = window.prompt(PromptLevel::Info, message, detail, &buttons, cx);
    async move { answer.await.ok().filter(|ix| *ix < count) }
}

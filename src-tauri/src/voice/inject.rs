use crate::settings::ClipboardRestore;
use arboard::Clipboard;
use rdev::{simulate, EventType, Key};
use serde::Serialize;
use std::{thread, time::Duration};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InjectionDeliveryMode {
    Pasted,
    Copied,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TextInjectionResult {
    pub delivery_mode: InjectionDeliveryMode,
    pub clipboard_restored: bool,
    pub manual_action_required: bool,
    pub error_code: Option<String>,
    pub message: String,
}

impl TextInjectionResult {
    pub fn injected(&self) -> bool {
        matches!(self.delivery_mode, InjectionDeliveryMode::Pasted)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TextInjectionFailure {
    pub code: String,
    pub message: String,
    pub manual_action_required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TextInjectionEvent {
    pub delivery_mode: Option<InjectionDeliveryMode>,
    pub clipboard_restored: bool,
    pub manual_action_required: bool,
    pub error_code: Option<String>,
    pub message: String,
}

impl From<TextInjectionResult> for TextInjectionEvent {
    fn from(result: TextInjectionResult) -> Self {
        Self {
            delivery_mode: Some(result.delivery_mode),
            clipboard_restored: result.clipboard_restored,
            manual_action_required: result.manual_action_required,
            error_code: result.error_code,
            message: result.message,
        }
    }
}

impl From<TextInjectionFailure> for TextInjectionEvent {
    fn from(failure: TextInjectionFailure) -> Self {
        Self {
            delivery_mode: None,
            clipboard_restored: false,
            manual_action_required: failure.manual_action_required,
            error_code: Some(failure.code),
            message: failure.message,
        }
    }
}

pub fn inject_text(
    text: &str,
    clipboard_restore: &ClipboardRestore,
) -> Result<TextInjectionResult, TextInjectionFailure> {
    let mut adapter = SystemInjectionAdapter::new()?;
    perform_injection(&mut adapter, text, clipboard_restore)
}

trait InjectionAdapter {
    fn capture_clipboard_text(&mut self) -> Result<Option<String>, String>;
    fn write_plain_text_to_clipboard(&mut self, text: &str) -> Result<(), String>;
    fn has_editable_focus(&self) -> bool;
    fn simulate_paste(&mut self) -> Result<(), String>;
}

struct SystemInjectionAdapter {
    clipboard: Clipboard,
}

impl SystemInjectionAdapter {
    fn new() -> Result<Self, TextInjectionFailure> {
        let clipboard = Clipboard::new().map_err(|error| {
            injection_failure(
                "INJECT_CLIPBOARD_WRITE_FAILED",
                format!("Failed to access the system clipboard: {error}"),
                false,
            )
        })?;
        Ok(Self { clipboard })
    }
}

impl InjectionAdapter for SystemInjectionAdapter {
    fn capture_clipboard_text(&mut self) -> Result<Option<String>, String> {
        Ok(self.clipboard.get_text().ok())
    }

    fn write_plain_text_to_clipboard(&mut self, text: &str) -> Result<(), String> {
        self.clipboard
            .set_text(text.to_string())
            .map_err(|error| error.to_string())
    }

    fn has_editable_focus(&self) -> bool {
        is_probably_editable_focus()
    }

    fn simulate_paste(&mut self) -> Result<(), String> {
        paste_from_clipboard().map_err(|error| format!("{error:?}"))
    }

}

fn perform_injection<A: InjectionAdapter>(
    adapter: &mut A,
    text: &str,
    _clipboard_restore: &ClipboardRestore,
) -> Result<TextInjectionResult, TextInjectionFailure> {
    let text = text.trim();
    if text.is_empty() {
        return Err(injection_failure(
            "INJECT_EMPTY_TEXT",
            "Recognized text is empty.",
            false,
        ));
    }

    let previous_text = adapter.capture_clipboard_text().ok().flatten();
    adapter
        .write_plain_text_to_clipboard(text)
        .map_err(|error| {
            injection_failure(
                "INJECT_CLIPBOARD_WRITE_FAILED",
                format!("Failed to write recognized text to the clipboard: {error}"),
                false,
            )
        })?;

    if !adapter.has_editable_focus() {
        return Ok(TextInjectionResult {
            delivery_mode: InjectionDeliveryMode::Copied,
            clipboard_restored: false,
            manual_action_required: true,
            error_code: None,
            message: "Text copied to clipboard. Focus an input box and paste it manually.".to_string(),
        });
    }

    adapter.simulate_paste().map_err(|error| {
        injection_failure(
            "INJECT_PASTE_FAILED",
            format!("Failed to paste recognized text into the target app: {error}"),
            true,
        )
    })?;

    let _ = previous_text;
    Ok(TextInjectionResult {
        delivery_mode: InjectionDeliveryMode::Pasted,
        clipboard_restored: false,
        manual_action_required: false,
        error_code: None,
        message: "Text injected successfully and kept in clipboard.".to_string(),
    })
}

fn injection_failure(
    code: &str,
    message: impl Into<String>,
    manual_action_required: bool,
) -> TextInjectionFailure {
    TextInjectionFailure {
        code: code.to_string(),
        message: message.into(),
        manual_action_required,
    }
}

fn paste_from_clipboard() -> Result<(), rdev::SimulateError> {
    simulate(&EventType::KeyPress(Key::ControlLeft))?;
    simulate(&EventType::KeyPress(Key::KeyV))?;
    thread::sleep(Duration::from_millis(30));
    simulate(&EventType::KeyRelease(Key::KeyV))?;
    simulate(&EventType::KeyRelease(Key::ControlLeft))?;
    Ok(())
}

pub fn is_probably_editable_focus() -> bool {
    cfg!(target_os = "windows")
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[derive(Debug, Default)]
    struct FakeInjectionAdapter {
        previous_text: Option<String>,
        capture_error: Option<String>,
        initial_write_error: Option<String>,
        paste_error: Option<String>,
        has_focus: bool,
        writes: Vec<String>,
    }

    impl InjectionAdapter for FakeInjectionAdapter {
        fn capture_clipboard_text(&mut self) -> Result<Option<String>, String> {
            match &self.capture_error {
                Some(error) => Err(error.clone()),
                None => Ok(self.previous_text.clone()),
            }
        }

        fn write_plain_text_to_clipboard(&mut self, text: &str) -> Result<(), String> {
            if let Some(error) = &self.initial_write_error {
                return Err(error.clone());
            }
            self.writes.push(text.to_string());
            Ok(())
        }

        fn has_editable_focus(&self) -> bool {
            self.has_focus
        }

        fn simulate_paste(&mut self) -> Result<(), String> {
            match &self.paste_error {
                Some(error) => Err(error.clone()),
                None => Ok(()),
            }
        }
    }

    #[test]
    fn empty_text_is_rejected_before_touching_clipboard() {
        let mut adapter = FakeInjectionAdapter::default();
        let result = perform_injection(&mut adapter, "", &ClipboardRestore::Always);

        assert_eq!(
            result,
            Err(TextInjectionFailure {
                code: "INJECT_EMPTY_TEXT".to_string(),
                message: "Recognized text is empty.".to_string(),
                manual_action_required: false,
            })
        );
        assert!(adapter.writes.is_empty());
    }

    #[test]
    fn editable_detection_uses_windows_first_policy() {
        assert_eq!(is_probably_editable_focus(), cfg!(target_os = "windows"));
    }

    #[test]
    fn copied_only_result_keeps_text_in_clipboard_for_manual_paste() {
        let mut adapter = FakeInjectionAdapter {
            has_focus: false,
            ..Default::default()
        };

        let result = perform_injection(&mut adapter, "hello world", &ClipboardRestore::Always).unwrap();

        assert_eq!(result.delivery_mode, InjectionDeliveryMode::Copied);
        assert!(!result.injected());
        assert!(!result.clipboard_restored);
        assert!(result.manual_action_required);
        assert_eq!(result.error_code, None);
        assert_eq!(adapter.writes, vec!["hello world".to_string()]);
    }

    #[test]
    fn pasted_result_keeps_recognized_text_in_clipboard() {
        let mut adapter = FakeInjectionAdapter {
            has_focus: true,
            previous_text: Some("previous text".to_string()),
            ..Default::default()
        };

        let result = perform_injection(&mut adapter, "hello world", &ClipboardRestore::Always).unwrap();

        assert_eq!(result.delivery_mode, InjectionDeliveryMode::Pasted);
        assert!(result.injected());
        assert!(!result.clipboard_restored);
        assert!(!result.manual_action_required);
        assert_eq!(result.error_code, None);
        assert_eq!(adapter.writes, vec!["hello world".to_string()]);
    }

    #[test]
    fn paste_failure_returns_structured_error() {
        let mut adapter = FakeInjectionAdapter {
            has_focus: true,
            paste_error: Some("target rejected paste".to_string()),
            ..Default::default()
        };

        let result = perform_injection(&mut adapter, "hello world", &ClipboardRestore::Always);

        assert_eq!(
            result,
            Err(TextInjectionFailure {
                code: "INJECT_PASTE_FAILED".to_string(),
                message:
                    "Failed to paste recognized text into the target app: target rejected paste"
                        .to_string(),
                manual_action_required: true,
            })
        );
    }
}

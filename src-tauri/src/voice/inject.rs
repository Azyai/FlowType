use crate::{
    error::{AppError, AppResult},
    settings::ClipboardRestore,
};
use arboard::Clipboard;
use rdev::{simulate, EventType, Key};
use std::{thread, time::Duration};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InjectionOutcome {
    Pasted,
    Copied,
}

pub fn inject_text(text: &str, clipboard_restore: &ClipboardRestore) -> AppResult<InjectionOutcome> {
    if text.trim().is_empty() {
        return Err(AppError::TextInjection("recognized text is empty".to_string()));
    }

    let mut clipboard = Clipboard::new().map_err(|error| AppError::TextInjection(error.to_string()))?;
    let previous = clipboard.get_text().ok();
    clipboard
        .set_text(text.to_string())
        .map_err(|error| AppError::TextInjection(error.to_string()))?;

    if is_probably_editable_focus() {
        paste_from_clipboard()?;
        restore_clipboard_if_needed(&mut clipboard, previous, clipboard_restore);
        Ok(InjectionOutcome::Pasted)
    } else {
        Ok(InjectionOutcome::Copied)
    }
}

fn paste_from_clipboard() -> AppResult<()> {
    simulate(&EventType::KeyPress(Key::ControlLeft)).map_err(|error| AppError::TextInjection(format!("{error:?}")))?;
    simulate(&EventType::KeyPress(Key::KeyV)).map_err(|error| AppError::TextInjection(format!("{error:?}")))?;
    thread::sleep(Duration::from_millis(30));
    simulate(&EventType::KeyRelease(Key::KeyV)).map_err(|error| AppError::TextInjection(format!("{error:?}")))?;
    simulate(&EventType::KeyRelease(Key::ControlLeft)).map_err(|error| AppError::TextInjection(format!("{error:?}")))?;
    Ok(())
}

fn restore_clipboard_if_needed(
    clipboard: &mut Clipboard,
    previous: Option<String>,
    clipboard_restore: &ClipboardRestore,
) {
    if !matches!(clipboard_restore, ClipboardRestore::Always | ClipboardRestore::TextOnly | ClipboardRestore::Delayed) {
        return;
    }

    let Some(previous) = previous else {
        return;
    };

    if matches!(clipboard_restore, ClipboardRestore::Delayed) {
        thread::sleep(Duration::from_millis(800));
    }

    if let Err(error) = clipboard.set_text(previous) {
        log::warn!("failed to restore clipboard after injection: {error}");
    }
}

pub fn is_probably_editable_focus() -> bool {
    cfg!(target_os = "windows")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_text_is_rejected_before_touching_clipboard() {
        let result = inject_text("", &ClipboardRestore::Always);

        assert!(result.is_err());
    }

    #[test]
    fn editable_detection_uses_windows_first_policy() {
        assert_eq!(is_probably_editable_focus(), cfg!(target_os = "windows"));
    }
}

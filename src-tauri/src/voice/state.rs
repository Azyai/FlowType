use crate::{
    error::{AppError, AppResult},
    settings::AppSettings,
};
use serde::{Deserialize, Serialize};
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum VoiceStatus {
    Idle,
    Listening,
    Uploading,
    Recognizing,
    Injecting,
    Success,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VoiceTrigger {
    Hotkey,
    Mascot,
    Tray,
}

#[derive(Debug, Clone, Serialize)]
pub struct VoiceSessionEvent {
    pub status: VoiceStatus,
    pub transcript_partial: Option<String>,
    pub transcript_final: Option<String>,
    pub error_code: Option<String>,
    pub message: Option<String>,
}

impl VoiceSessionEvent {
    pub fn status(status: VoiceStatus) -> Self {
        Self {
            status,
            transcript_partial: None,
            transcript_final: None,
            error_code: None,
            message: None,
        }
    }

    pub fn message(status: VoiceStatus, message: impl Into<String>) -> Self {
        Self {
            status,
            transcript_partial: None,
            transcript_final: None,
            error_code: None,
            message: Some(message.into()),
        }
    }

    pub fn partial(text: impl Into<String>) -> Self {
        Self {
            status: VoiceStatus::Recognizing,
            transcript_partial: Some(text.into()),
            transcript_final: None,
            error_code: None,
            message: None,
        }
    }

    pub fn final_text(text: impl Into<String>) -> Self {
        Self {
            status: VoiceStatus::Success,
            transcript_partial: None,
            transcript_final: Some(text.into()),
            error_code: None,
            message: None,
        }
    }

    pub fn failed(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            status: VoiceStatus::Failed,
            transcript_partial: None,
            transcript_final: None,
            error_code: Some(code.into()),
            message: Some(message.into()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ActiveVoiceSession {
    pub trigger: VoiceTrigger,
    pub started_at: Instant,
}

#[derive(Debug)]
pub struct VoiceStateMachine {
    status: VoiceStatus,
    active: Option<ActiveVoiceSession>,
}

impl Default for VoiceStateMachine {
    fn default() -> Self {
        Self {
            status: VoiceStatus::Idle,
            active: None,
        }
    }
}

impl VoiceStateMachine {
    pub fn status(&self) -> VoiceStatus {
        self.status
    }

    pub fn start(&mut self, trigger: VoiceTrigger) -> AppResult<VoiceSessionEvent> {
        if matches!(self.status, VoiceStatus::Listening | VoiceStatus::Uploading | VoiceStatus::Recognizing | VoiceStatus::Injecting) {
            return Err(AppError::Voice("voice input is already active".to_string()));
        }

        self.status = VoiceStatus::Listening;
        self.active = Some(ActiveVoiceSession {
            trigger,
            started_at: Instant::now(),
        });
        Ok(VoiceSessionEvent::status(VoiceStatus::Listening))
    }

    pub fn stop(&mut self, settings: &AppSettings) -> AppResult<StopDecision> {
        let Some(active) = self.active.take() else {
            return Err(AppError::Voice("voice input is not active".to_string()));
        };

        let elapsed_ms = active.started_at.elapsed().as_millis() as u64;
        if elapsed_ms < settings.min_recording_ms {
            self.status = VoiceStatus::Idle;
            return Ok(StopDecision::DiscardTooShort {
                elapsed_ms,
                message: "Recording was too short and has been discarded.".to_string(),
            });
        }

        self.status = VoiceStatus::Uploading;
        Ok(StopDecision::Process { elapsed_ms })
    }

    pub fn cancel(&mut self) -> VoiceSessionEvent {
        self.active = None;
        self.status = VoiceStatus::Idle;
        VoiceSessionEvent::message(VoiceStatus::Idle, "Recording canceled.")
    }

    pub fn transition(&mut self, status: VoiceStatus) -> VoiceSessionEvent {
        if matches!(status, VoiceStatus::Idle | VoiceStatus::Success | VoiceStatus::Failed) {
            self.active = None;
        }
        self.status = status;
        VoiceSessionEvent::status(status)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StopDecision {
    Process { elapsed_ms: u64 },
    DiscardTooShort { elapsed_ms: u64, message: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_moves_idle_to_listening_and_rejects_duplicate_start() {
        let mut machine = VoiceStateMachine::default();

        let event = machine.start(VoiceTrigger::Hotkey).unwrap();

        assert_eq!(event.status, VoiceStatus::Listening);
        assert!(machine.start(VoiceTrigger::Mascot).is_err());
    }

    #[test]
    fn cancel_returns_to_idle_without_processing() {
        let mut machine = VoiceStateMachine::default();
        machine.start(VoiceTrigger::Mascot).unwrap();

        let event = machine.cancel();

        assert_eq!(event.status, VoiceStatus::Idle);
        assert_eq!(machine.status(), VoiceStatus::Idle);
    }

    #[test]
    fn stop_discards_short_recording() {
        let mut settings = AppSettings::default();
        settings.min_recording_ms = 60_000;
        let mut machine = VoiceStateMachine::default();
        machine.start(VoiceTrigger::Hotkey).unwrap();

        let decision = machine.stop(&settings).unwrap();

        assert!(matches!(decision, StopDecision::DiscardTooShort { .. }));
        assert_eq!(machine.status(), VoiceStatus::Idle);
    }

    #[test]
    fn stop_processes_recording_after_threshold() {
        let mut settings = AppSettings::default();
        settings.min_recording_ms = 0;
        let mut machine = VoiceStateMachine::default();
        machine.start(VoiceTrigger::Hotkey).unwrap();

        let decision = machine.stop(&settings).unwrap();

        assert!(matches!(decision, StopDecision::Process { .. }));
        assert_eq!(machine.status(), VoiceStatus::Uploading);
    }
}

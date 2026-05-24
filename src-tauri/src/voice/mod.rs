pub mod audio;
pub mod iflytek;
pub mod inject;
pub mod output;
pub mod state;

use crate::{
    error::{AppError, AppResult},
    settings::{AppSettings, OutputStyle},
    storage::NewTranscriptHistory,
    voice::{
        audio::{AudioRecorder, RecordedAudio},
        output::transform_output_text,
        state::{StopDecision, VoiceSessionEvent, VoiceStateMachine, VoiceStatus, VoiceTrigger},
    },
};
use std::sync::Mutex;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter, Manager};

pub struct VoiceController {
    machine: Mutex<VoiceStateMachine>,
    recorder: Mutex<Option<AudioRecorder>>,
}

impl std::fmt::Debug for VoiceController {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_struct("VoiceController").finish_non_exhaustive()
    }
}

impl Default for VoiceController {
    fn default() -> Self {
        Self {
            machine: Mutex::new(VoiceStateMachine::default()),
            recorder: Mutex::new(None),
        }
    }
}

impl VoiceController {
    pub fn status(&self) -> AppResult<VoiceStatus> {
        Ok(self
            .machine
            .lock()
            .map_err(|_| AppError::StateLock)?
            .status())
    }

    pub fn start(&self, app: &AppHandle, settings: &AppSettings, trigger: VoiceTrigger) -> AppResult<VoiceSessionEvent> {
        let mut machine = self.machine.lock().map_err(|_| AppError::StateLock)?;
        let event = machine.start(trigger)?;
        drop(machine);

        let app_handle = app.clone();
        let recorder = AudioRecorder::start(move |level| {
            emit_level(&app_handle, level);
        })?;
        *self.recorder.lock().map_err(|_| AppError::StateLock)? = Some(recorder);
        emit_level(app, 0.0);
        self.emit(app, &event);
        self.schedule_max_duration_stop(app.clone(), settings.max_recording_ms);
        Ok(event)
    }

    pub fn stop(&self, app: AppHandle, settings: AppSettings, trigger: VoiceTrigger) -> AppResult<VoiceSessionEvent> {
        let mut machine = self.machine.lock().map_err(|_| AppError::StateLock)?;
        let decision = machine.stop(&settings)?;
        drop(machine);

        let audio = self
            .recorder
            .lock()
            .map_err(|_| AppError::StateLock)?
            .take()
            .map(AudioRecorder::stop)
            .unwrap_or_else(|| RecordedAudio {
                pcm: Vec::new(),
                sample_rate: 16_000,
            });
        emit_level(&app, 0.0);

        match decision {
            StopDecision::DiscardTooShort { message, .. } => {
                let event = VoiceSessionEvent::message(VoiceStatus::Idle, message);
                self.emit(&app, &event);
                Ok(event)
            }
            StopDecision::Process { .. } => {
                let uploading = self.transition(&app, VoiceStatus::Uploading)?;
                self.run_recognition_pipeline(app, settings, trigger, audio);
                Ok(uploading)
            }
        }
    }

    pub fn cancel(&self, app: &AppHandle) -> AppResult<VoiceSessionEvent> {
        self.recorder.lock().map_err(|_| AppError::StateLock)?.take();
        emit_level(app, 0.0);
        let event = self.machine.lock().map_err(|_| AppError::StateLock)?.cancel();
        self.emit(app, &event);
        Ok(event)
    }

    pub fn transition(&self, app: &AppHandle, status: VoiceStatus) -> AppResult<VoiceSessionEvent> {
        let event = self.machine.lock().map_err(|_| AppError::StateLock)?.transition(status);
        self.emit(app, &event);
        Ok(event)
    }

    fn emit(&self, app: &AppHandle, event: &VoiceSessionEvent) {
        if let Err(error) = app.emit("voice_status_changed", event) {
            log::warn!("failed to emit voice status: {error}");
        }
        if let Err(error) = app.emit("status_changed", event.status) {
            log::warn!("failed to emit legacy status: {error}");
        }
    }

    fn schedule_max_duration_stop(&self, app: AppHandle, max_recording_ms: u64) {
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(max_recording_ms));
            let Some(state) = app.try_state::<crate::app::AppState>() else {
                return;
            };
            if matches!(state.voice_status().ok(), Some(VoiceStatus::Listening)) {
                if let Ok(settings) = state.settings() {
                    let _ = state.stop_voice_input(app.clone(), settings, VoiceTrigger::Hotkey);
                }
            }
        });
    }

    fn run_recognition_pipeline(
        &self,
        app: AppHandle,
        settings: AppSettings,
        _trigger: VoiceTrigger,
        audio: RecordedAudio,
    ) {
        std::thread::spawn(move || {
            let Some(state) = app.try_state::<crate::app::AppState>() else {
                return;
            };
            let recognition_started_at = now_unix_seconds().to_string();
            let recognition_started = Instant::now();
            let _ = state.transition_voice(&app, VoiceStatus::Recognizing);
            let recognition = iflytek::recognize_blocking(&settings, audio, |partial| {
                state.emit_voice_event(
                    &app,
                    VoiceSessionEvent::partial(partial),
                );
            });
            let recognition_duration_ms = elapsed_millis_i64(recognition_started);

            let raw_text = match recognition {
                Ok(result) if !result.text.trim().is_empty() => result.text.trim().to_string(),
                Ok(_) => {
                    let message = "No speech text was recognized.".to_string();
                    record_history_if_enabled(
                        &state,
                        &settings,
                        "",
                        "",
                        &recognition_started_at,
                        recognition_duration_ms,
                        false,
                        Some("ASR_EMPTY"),
                        Some(message.as_str()),
                    );
                    state.fail_voice(&app, "ASR_EMPTY", "No speech text was recognized.");
                    return;
                }
                Err(error) => {
                    let message = error.to_string();
                    record_history_if_enabled(
                        &state,
                        &settings,
                        "",
                        "",
                        &recognition_started_at,
                        recognition_duration_ms,
                        false,
                        Some("ASR_FAILED"),
                        Some(message.as_str()),
                    );
                    state.fail_voice(&app, "ASR_FAILED", message);
                    return;
                }
            };
            let final_text = transformed_output_text(&raw_text, &settings.output_style);

            let _ = state.transition_voice(&app, VoiceStatus::Injecting);
            match inject::inject_text(&final_text, &settings.clipboard_restore) {
                Ok(outcome) => {
                    record_history_if_enabled(
                        &state,
                        &settings,
                        &raw_text,
                        &final_text,
                        &recognition_started_at,
                        recognition_duration_ms,
                        matches!(outcome, inject::InjectionOutcome::Pasted),
                        None,
                        None,
                    );
                    state.emit_voice_event(&app, VoiceSessionEvent::final_text(final_text));
                    let _ = state.transition_voice(&app, VoiceStatus::Success);
                }
                Err(error) => {
                    let message = error.to_string();
                    record_history_if_enabled(
                        &state,
                        &settings,
                        &raw_text,
                        &final_text,
                        &recognition_started_at,
                        recognition_duration_ms,
                        false,
                        Some("INJECT_FAILED"),
                        Some(message.as_str()),
                    );
                    state.fail_voice(&app, "INJECT_FAILED", message);
                }
            }
        });
    }
}

fn transformed_output_text(raw_text: &str, output_style: &OutputStyle) -> String {
    let transformed = transform_output_text(raw_text, output_style);
    if transformed.is_empty() {
        raw_text.to_string()
    } else {
        transformed
    }
}

fn record_history_if_enabled(
    state: &crate::app::AppState,
    settings: &AppSettings,
    raw_text: &str,
    final_text: &str,
    recognition_started_at: &str,
    recognition_duration_ms: i64,
    injected: bool,
    error_code: Option<&str>,
    error_summary: Option<&str>,
) {
    if !settings.save_history {
        return;
    }

    if let Err(error) = state.record_transcript_history(NewTranscriptHistory {
        raw_text,
        final_text,
        output_style: output_style_label(&settings.output_style),
        recognition_started_at,
        recognition_duration_ms,
        injected,
        error_code,
        error_summary,
    }) {
        log::warn!("failed to record transcript history: {error}");
    }
}

fn output_style_label(output_style: &OutputStyle) -> &'static str {
    match output_style {
        OutputStyle::Raw => "raw",
        OutputStyle::Clean => "clean",
        OutputStyle::Formal => "formal",
    }
}

fn now_unix_seconds() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or_default()
}

fn elapsed_millis_i64(started_at: Instant) -> i64 {
    started_at.elapsed().as_millis().min(i64::MAX as u128) as i64
}

fn emit_level(app: &AppHandle, level: f32) {
    if let Err(error) = app.emit("voice_level_changed", level.clamp(0.0, 1.0)) {
        log::warn!("failed to emit voice level: {error}");
    }
}

pub mod audio;
pub mod inject;
pub mod output;
pub mod rtasr;
pub mod state;

use crate::{
    error::{AppError, AppResult},
    settings::{AppSettings, OutputStyle},
    storage::NewTranscriptHistory,
    voice::{
        audio::AudioRecorder,
        output::transform_output_text,
        rtasr::StreamingRecognizer,
        state::{StopDecision, VoiceSessionEvent, VoiceStateMachine, VoiceStatus, VoiceTrigger},
    },
};
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc, Mutex,
};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter, Manager};

const AUTO_STOP_SILENCE_MS: u64 = 1_200;
const AUTO_STOP_POLL_INTERVAL_MS: u64 = 120;
const AUTO_STOP_LEVEL_THRESHOLD: f32 = 0.035;

pub struct VoiceController {
    machine: Mutex<VoiceStateMachine>,
    recorder: Mutex<Option<AudioRecorder>>,
    recognizer: Mutex<Option<StreamingRecognizer>>,
    active_session_id: Arc<AtomicU64>,
    next_session_id: AtomicU64,
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
            recognizer: Mutex::new(None),
            active_session_id: Arc::new(AtomicU64::new(0)),
            next_session_id: AtomicU64::new(0),
        }
    }
}

#[derive(Debug)]
struct VoiceActivityTracker {
    speech_detected: AtomicBool,
    last_speech_at: Mutex<Instant>,
}

impl VoiceActivityTracker {
    fn new() -> Self {
        Self {
            speech_detected: AtomicBool::new(false),
            last_speech_at: Mutex::new(Instant::now()),
        }
    }

    fn observe(&self, level: f32) {
        if level < AUTO_STOP_LEVEL_THRESHOLD {
            return;
        }
        self.speech_detected.store(true, Ordering::Relaxed);
        if let Ok(mut last_speech_at) = self.last_speech_at.lock() {
            *last_speech_at = Instant::now();
        }
    }

    fn speech_detected(&self) -> bool {
        self.speech_detected.load(Ordering::Relaxed)
    }

    fn silence_elapsed(&self) -> Option<Duration> {
        self.last_speech_at.lock().ok().map(|last_speech_at| last_speech_at.elapsed())
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
        emit_level(app, 0.0);
        self.emit(app, &event);

        let session_id = self.next_session_id.fetch_add(1, Ordering::Relaxed) + 1;
        let activity = Arc::new(VoiceActivityTracker::new());
        let partial_app = app.clone();
        let recognizer = match rtasr::start_streaming_session(settings.clone(), move |partial| {
            let Some(state) = partial_app.try_state::<crate::app::AppState>() else {
                return;
            };
            let partial_status = state.voice_status().unwrap_or(VoiceStatus::Listening);
            state.emit_voice_event(
                &partial_app,
                VoiceSessionEvent::partial_with_status(partial_status, partial),
            );
        }) {
            Ok(recognizer) => recognizer,
            Err(error) => {
                let cancel_event = self.machine.lock().map_err(|_| AppError::StateLock)?.cancel();
                emit_level(app, 0.0);
                self.emit(app, &cancel_event);
                return Err(error);
            }
        };
        let recognizer_sink = recognizer.sink();
        let app_handle = app.clone();
        let activity_handle = activity.clone();
        let recorder = AudioRecorder::start(
            move |level| {
                activity_handle.observe(level);
                emit_level(&app_handle, level);
            },
            {
                let recognizer_sink = recognizer_sink.clone();
                move |samples, sample_rate| {
                    if let Err(error) = recognizer_sink.push_audio(samples, sample_rate) {
                        log::warn!("failed to stream audio chunk to RTASR: {error}");
                    }
                }
            },
        );
        let recorder = match recorder {
            Ok(recorder) => recorder,
            Err(error) => {
                let cancel_event = self.machine.lock().map_err(|_| AppError::StateLock)?.cancel();
                emit_level(app, 0.0);
                recognizer.cancel();
                self.emit(app, &cancel_event);
                return Err(error);
            }
        };

        self.active_session_id.store(session_id, Ordering::Relaxed);
        *self.recorder.lock().map_err(|_| AppError::StateLock)? = Some(recorder);
        *self.recognizer.lock().map_err(|_| AppError::StateLock)? = Some(recognizer);
        emit_level(app, 0.0);
        self.schedule_max_duration_stop(app.clone(), settings.max_recording_ms, trigger, session_id);
        if should_enable_auto_stop(trigger, settings) {
            self.schedule_auto_stop_on_silence(
                app.clone(),
                settings.min_recording_ms,
                trigger,
                session_id,
                activity,
            );
        }
        Ok(event)
    }

    pub fn stop(&self, app: AppHandle, settings: AppSettings, trigger: VoiceTrigger) -> AppResult<VoiceSessionEvent> {
        let mut machine = self.machine.lock().map_err(|_| AppError::StateLock)?;
        let decision = machine.stop(&settings)?;
        drop(machine);

        self.recorder
            .lock()
            .map_err(|_| AppError::StateLock)?
            .take()
            .map(AudioRecorder::stop);
        let recognizer = self.recognizer.lock().map_err(|_| AppError::StateLock)?.take();
        self.active_session_id.store(0, Ordering::Relaxed);
        emit_level(&app, 0.0);

        match decision {
            StopDecision::DiscardTooShort { message, .. } => {
                if let Some(recognizer) = recognizer {
                    recognizer.cancel();
                }
                let event = VoiceSessionEvent::message(VoiceStatus::Idle, message);
                self.emit(&app, &event);
                Ok(event)
            }
            StopDecision::Process { .. } => {
                let recognizing = self.transition(&app, VoiceStatus::Recognizing)?;
                if let Some(recognizer) = recognizer {
                    self.run_recognition_pipeline(app, settings, trigger, recognizer);
                } else {
                    return Err(AppError::Voice("RTASR session was not available during stop.".to_string()));
                }
                Ok(recognizing)
            }
        }
    }

    pub fn cancel(&self, app: &AppHandle) -> AppResult<VoiceSessionEvent> {
        self.recorder.lock().map_err(|_| AppError::StateLock)?.take();
        if let Some(recognizer) = self.recognizer.lock().map_err(|_| AppError::StateLock)?.take() {
            recognizer.cancel();
        }
        self.active_session_id.store(0, Ordering::Relaxed);
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

    fn schedule_max_duration_stop(
        &self,
        app: AppHandle,
        max_recording_ms: u64,
        trigger: VoiceTrigger,
        session_id: u64,
    ) {
        let active_session_id = self.active_session_id.clone();
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(max_recording_ms));
            if active_session_id.load(Ordering::Relaxed) != session_id {
                return;
            }
            let Some(state) = app.try_state::<crate::app::AppState>() else {
                return;
            };
            if matches!(state.voice_status().ok(), Some(VoiceStatus::Listening)) {
                if let Ok(settings) = state.settings() {
                    let _ = state.stop_voice_input(app.clone(), settings, trigger);
                }
            }
        });
    }

    fn schedule_auto_stop_on_silence(
        &self,
        app: AppHandle,
        min_recording_ms: u64,
        trigger: VoiceTrigger,
        session_id: u64,
        activity: Arc<VoiceActivityTracker>,
    ) {
        let active_session_id = self.active_session_id.clone();
        std::thread::spawn(move || {
            let started_at = Instant::now();
            loop {
                std::thread::sleep(Duration::from_millis(AUTO_STOP_POLL_INTERVAL_MS));
                if active_session_id.load(Ordering::Relaxed) != session_id {
                    return;
                }
                let Some(state) = app.try_state::<crate::app::AppState>() else {
                    return;
                };
                if !matches!(state.voice_status().ok(), Some(VoiceStatus::Listening)) {
                    return;
                }
                if started_at.elapsed() < Duration::from_millis(min_recording_ms) {
                    continue;
                }
                if !activity.speech_detected() {
                    continue;
                }
                let Some(silence_elapsed) = activity.silence_elapsed() else {
                    continue;
                };
                if silence_elapsed < Duration::from_millis(AUTO_STOP_SILENCE_MS) {
                    continue;
                }
                if let Ok(settings) = state.settings() {
                    let _ = state.stop_voice_input(app.clone(), settings, trigger);
                }
                return;
            }
        });
    }

    fn run_recognition_pipeline(
        &self,
        app: AppHandle,
        settings: AppSettings,
        _trigger: VoiceTrigger,
        recognizer: StreamingRecognizer,
    ) {
        std::thread::spawn(move || {
            let Some(state) = app.try_state::<crate::app::AppState>() else {
                return;
            };
            let recognition_started_at = now_unix_seconds().to_string();
            let recognition_started = Instant::now();
            let recognition = recognizer.finish();
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
                    let history_error_summary = outcome
                        .error_code
                        .as_ref()
                        .map(|_| outcome.message.as_str());
                    record_history_if_enabled(
                        &state,
                        &settings,
                        &raw_text,
                        &final_text,
                        &recognition_started_at,
                        recognition_duration_ms,
                        outcome.injected(),
                        outcome.error_code.as_deref(),
                        history_error_summary,
                    );
                    state.emit_voice_event(&app, VoiceSessionEvent::final_text(final_text));
                    emit_text_injection_event(&app, outcome.into());
                    let _ = state.transition_voice(&app, VoiceStatus::Success);
                }
                Err(failure) => {
                    let code = failure.code.clone();
                    let message = failure.message.clone();
                    record_history_if_enabled(
                        &state,
                        &settings,
                        &raw_text,
                        &final_text,
                        &recognition_started_at,
                        recognition_duration_ms,
                        false,
                        Some(code.as_str()),
                        Some(message.as_str()),
                    );
                    emit_text_injection_event(&app, failure.clone().into());
                    state.fail_voice(&app, code.as_str(), message);
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
    if !settings.save_history || !should_record_history_entry(error_code, error_summary) {
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

fn should_record_history_entry(error_code: Option<&str>, error_summary: Option<&str>) -> bool {
    error_code.is_none() && error_summary.map(str::trim).unwrap_or_default().is_empty()
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

fn emit_text_injection_event(app: &AppHandle, event: inject::TextInjectionEvent) {
    if let Err(error) = app.emit("text_injection_result", event) {
        log::warn!("failed to emit text injection result: {error}");
    }
}

fn emit_level(app: &AppHandle, level: f32) {
    if let Err(error) = app.emit("voice_level_changed", level.clamp(0.0, 1.0)) {
        log::warn!("failed to emit voice level: {error}");
    }
}

fn should_enable_auto_stop(trigger: VoiceTrigger, settings: &AppSettings) -> bool {
    !matches!(trigger, VoiceTrigger::Mascot | VoiceTrigger::HotkeyToggle) && settings.vad_enabled
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mascot_trigger_keeps_listening_until_manually_stopped() {
        let settings = AppSettings::default();

        assert!(!should_enable_auto_stop(VoiceTrigger::Mascot, &settings));
        assert!(!should_enable_auto_stop(VoiceTrigger::HotkeyToggle, &settings));
    }

    #[test]
    fn non_mascot_trigger_requires_vad_for_auto_stop() {
        let mut settings = AppSettings::default();

        assert!(!should_enable_auto_stop(VoiceTrigger::Hotkey, &settings));

        settings.vad_enabled = true;
        assert!(should_enable_auto_stop(VoiceTrigger::Hotkey, &settings));
        assert!(should_enable_auto_stop(VoiceTrigger::Tray, &settings));
    }

    #[test]
    fn activity_tracker_ignores_quiet_levels_and_tracks_speech() {
        let tracker = VoiceActivityTracker::new();

        tracker.observe(0.01);
        assert!(!tracker.speech_detected());

        tracker.observe(0.2);
        assert!(tracker.speech_detected());
        assert!(tracker.silence_elapsed().is_some());
    }

    #[test]
    fn history_recording_skips_failed_results() {
        assert!(should_record_history_entry(None, None));
        assert!(should_record_history_entry(None, Some("   ")));
        assert!(!should_record_history_entry(Some("ASR_EMPTY"), None));
        assert!(!should_record_history_entry(None, Some("No speech text was recognized.")));
    }
}

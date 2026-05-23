pub mod audio;
pub mod iflytek;
pub mod inject;
pub mod state;

use crate::{
    error::{AppError, AppResult},
    settings::AppSettings,
    voice::{
        audio::{AudioRecorder, RecordedAudio},
        state::{StopDecision, VoiceSessionEvent, VoiceStateMachine, VoiceStatus, VoiceTrigger},
    },
};
use std::sync::Mutex;
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

        let recorder = AudioRecorder::start()?;
        *self.recorder.lock().map_err(|_| AppError::StateLock)? = Some(recorder);
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
            let _ = state.transition_voice(&app, VoiceStatus::Recognizing);
            let recognition = iflytek::recognize_blocking(&settings, audio, |partial| {
                state.emit_voice_event(
                    &app,
                    VoiceSessionEvent::partial(partial),
                );
            });

            let recognized = match recognition {
                Ok(result) if !result.text.trim().is_empty() => result.text,
                Ok(_) => {
                    state.fail_voice(&app, "ASR_EMPTY", "No speech text was recognized.");
                    return;
                }
                Err(error) => {
                    state.fail_voice(&app, "ASR_FAILED", error.to_string());
                    return;
                }
            };

            let _ = state.transition_voice(&app, VoiceStatus::Injecting);
            match inject::inject_text(&recognized, &settings.clipboard_restore) {
                Ok(_) => {
                    state.emit_voice_event(&app, VoiceSessionEvent::final_text(recognized));
                    let _ = state.transition_voice(&app, VoiceStatus::Success);
                }
                Err(error) => {
                    state.fail_voice(&app, "INJECT_FAILED", error.to_string());
                }
            }
        });
    }
}

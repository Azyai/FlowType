use crate::{
    settings::{AppSettings, ConfigStore, OutputStyle},
    storage::{Database, DatabaseHealth, NewTranscriptHistory, TranscriptHistoryPage},
    voice::{
        state::{VoiceSessionEvent, VoiceStatus, VoiceTrigger},
        VoiceController,
    },
    error::{AppError, AppResult},
};
use std::sync::{atomic::{AtomicBool, Ordering}, Mutex};
use tauri::Emitter;

#[derive(Debug)]
pub struct AppState {
    config_store: ConfigStore,
    settings: Mutex<AppSettings>,
    database: Database,
    voice: VoiceController,
    paused: AtomicBool,
}

impl AppState {
    pub fn new(config_store: ConfigStore, settings: AppSettings, database: Database) -> Self {
        Self {
            config_store,
            settings: Mutex::new(settings),
            database,
            voice: VoiceController::default(),
            paused: AtomicBool::new(false),
        }
    }

    pub fn settings(&self) -> AppResult<AppSettings> {
        self.settings
            .lock()
            .map_err(|_| AppError::StateLock)
            .map(|settings| settings.clone())
    }

    pub fn save_settings(&self, mut settings: AppSettings) -> AppResult<AppSettings> {
        settings.enforce_hidden_defaults();
        self.config_store.save(&settings)?;
        *self.settings.lock().map_err(|_| AppError::StateLock)? = settings.clone();
        Ok(settings)
    }

    pub fn reset_settings(&self) -> AppResult<AppSettings> {
        self.save_settings(AppSettings::default())
    }

    pub fn update_output_style(&self, output_style: OutputStyle) -> AppResult<AppSettings> {
        let mut settings = self.settings()?;
        settings.output_style = output_style;
        self.save_settings(settings)
    }

    pub fn database_health(&self) -> DatabaseHealth {
        self.database.health()
    }

    pub fn clear_history(&self) -> AppResult<usize> {
        self.database.clear_transcript_history()
    }

    pub fn delete_history_item(&self, id: i64) -> AppResult<usize> {
        self.database.delete_transcript_history_item(id)
    }

    pub fn get_history(&self, limit: u32, offset: u32) -> AppResult<TranscriptHistoryPage> {
        self.database.get_transcript_history(limit, offset)
    }

    pub fn record_transcript_history(&self, entry: NewTranscriptHistory<'_>) -> AppResult<i64> {
        self.database.insert_transcript_history(entry)
    }

    pub fn voice_status(&self) -> AppResult<VoiceStatus> {
        self.voice.status()
    }

    pub fn start_voice_input(
        &self,
        app: &tauri::AppHandle,
        settings: &AppSettings,
        trigger: VoiceTrigger,
    ) -> AppResult<VoiceSessionEvent> {
        self.voice.start(app, settings, trigger)
    }

    pub fn stop_voice_input(
        &self,
        app: tauri::AppHandle,
        settings: AppSettings,
        trigger: VoiceTrigger,
    ) -> AppResult<VoiceSessionEvent> {
        self.voice.stop(app, settings, trigger)
    }

    pub fn toggle_voice_input(
        &self,
        app: tauri::AppHandle,
        settings: AppSettings,
        trigger: VoiceTrigger,
    ) -> AppResult<VoiceSessionEvent> {
        if self.voice_status()? == VoiceStatus::Listening {
            self.stop_voice_input(app, settings, trigger)
        } else {
            self.start_voice_input(&app, &settings, trigger)
        }
    }

    pub fn cancel_voice_input(&self, app: &tauri::AppHandle) -> AppResult<VoiceSessionEvent> {
        self.voice.cancel(app)
    }

    pub fn transition_voice(&self, app: &tauri::AppHandle, status: VoiceStatus) -> AppResult<VoiceSessionEvent> {
        self.voice.transition(app, status)
    }

    pub fn emit_voice_event(&self, app: &tauri::AppHandle, event: VoiceSessionEvent) {
        if let Err(error) = app.emit("voice_status_changed", &event) {
            log::warn!("failed to emit voice event: {error}");
        }
        if let Err(error) = app.emit("status_changed", event.status) {
            log::warn!("failed to emit legacy voice status: {error}");
        }
    }

    pub fn fail_voice(&self, app: &tauri::AppHandle, code: &str, message: impl Into<String>) {
        let event = VoiceSessionEvent::failed(code, message);
        self.emit_voice_event(app, event);
        let _ = self.transition_voice(app, VoiceStatus::Failed);
    }

    pub fn toggle_paused(&self) -> bool {
        let current = self.paused.load(Ordering::Relaxed);
        self.paused.store(!current, Ordering::Relaxed);
        !current
    }

    pub fn paused(&self) -> bool {
        self.paused.load(Ordering::Relaxed)
    }
}

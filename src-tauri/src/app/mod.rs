use crate::{
    settings::{AppSettings, ConfigStore, OutputStyle},
    storage::{Database, DatabaseHealth},
    error::{AppError, AppResult},
};
use std::sync::{atomic::{AtomicBool, Ordering}, Mutex};

#[derive(Debug)]
pub struct AppState {
    config_store: ConfigStore,
    settings: Mutex<AppSettings>,
    database: Database,
    paused: AtomicBool,
}

impl AppState {
    pub fn new(config_store: ConfigStore, settings: AppSettings, database: Database) -> Self {
        Self {
            config_store,
            settings: Mutex::new(settings),
            database,
            paused: AtomicBool::new(false),
        }
    }

    pub fn settings(&self) -> AppResult<AppSettings> {
        self.settings
            .lock()
            .map_err(|_| AppError::StateLock)
            .map(|settings| settings.clone())
    }

    pub fn save_settings(&self, settings: AppSettings) -> AppResult<AppSettings> {
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

    pub fn toggle_paused(&self) -> bool {
        let current = self.paused.load(Ordering::Relaxed);
        self.paused.store(!current, Ordering::Relaxed);
        !current
    }

    pub fn paused(&self) -> bool {
        self.paused.load(Ordering::Relaxed)
    }
}

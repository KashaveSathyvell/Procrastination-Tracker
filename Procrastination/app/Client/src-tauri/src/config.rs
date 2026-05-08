use std::path::PathBuf;
use tauri::Manager;

/// Root configuration object used throughout the application
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub paths: PathsConfig,
    pub capture: CaptureConfig,
    pub ml: MLConfig,
    pub intervention: InterventionConfig,
}

/// Application storage paths
#[derive(Debug, Clone)]
pub struct PathsConfig {
    pub app_data_dir: PathBuf,
    pub database_path: PathBuf,
    pub model_path: PathBuf,
    pub logs_dir: PathBuf,
}

/// Human-computer interaction capture settings
#[derive(Debug, Clone)]
pub struct CaptureConfig {
    pub keyboard_poll_interval_ms: u64,
    pub mouse_poll_interval_ms: u64,
    pub idle_threshold_seconds: u64,
}

/// Machine learning inference settings
#[derive(Debug, Clone)]
pub struct MLConfig {
    pub feature_window_seconds: u64,
    pub prediction_interval_seconds: u64,
}

/// Intervention system settings (JITAI logic)
#[derive(Debug, Clone)]
pub struct InterventionConfig {
    pub risk_prediction_threshold: u32,
    pub cooldown_minutes: u64,
}

impl AppConfig {
    /// Create a new configuration instance
    pub fn new(app_handle: &tauri::AppHandle) -> Self {
        let paths = PathsConfig::new(app_handle);

        Self {
            paths,
            capture: CaptureConfig::default(),
            ml: MLConfig::default(),
            intervention: InterventionConfig::default(),
        }
    }
}

impl PathsConfig {
    fn new(app_handle: &tauri::AppHandle) -> Self {
        // Determine application data directory
        let base_dir = app_handle.path().app_data_dir().expect("Failed to resolve AppData directory");

        let app_dir = base_dir.join("ProcrastinationAI");

        let database_path = app_dir.join("behavior.db");
        let model_path = app_dir.join("model/current_model.onnx");
        let logs_dir = app_dir.join("logs");

        // Ensure directories exist
        std::fs::create_dir_all(&logs_dir).expect("Failed to create logs directory");
        std::fs::create_dir_all(&app_dir).expect("Failed to create app directory");
        std::fs::create_dir_all(&app_dir.join("model")).expect("Failed to create model directory");

        Self {
            app_data_dir: app_dir,
            database_path,
            model_path,
            logs_dir,
        }
    }
}

impl Default for CaptureConfig {
    fn default() -> Self {
        Self {
            keyboard_poll_interval_ms: 100,
            mouse_poll_interval_ms: 100,
            idle_threshold_seconds: 15,
        }
    }
}

impl Default for MLConfig {
    fn default() -> Self {
        Self {
            feature_window_seconds: 30,
            prediction_interval_seconds: 15,
        }
    }
}

impl Default for InterventionConfig {
    fn default() -> Self {
        Self {
            risk_prediction_threshold: 3,
            cooldown_minutes: 15,
        }
    }
}
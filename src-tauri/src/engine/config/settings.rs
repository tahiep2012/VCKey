use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppSettings {
    pub mode: String,                // "Vietnamese", "Chinese", "English"
    pub input_type: String,          // "Telex", "VNI", "Simple Telex"
    pub charset: String,             // "Unicode Dựng sẵn", "Unicode Tổ hợp", "VNI Windows", "TCVN3 (ABC)"
    pub hotkey_ctrl: bool,
    pub hotkey_shift: bool,
    pub hotkey_alt: bool,
    pub hotkey_win: bool,
    pub beep_on_switch: bool,
    pub traditional_chinese: bool,
    pub cand_page_size: usize,       // 5 or 9
    pub cand_font_size: usize,       // 14 to 25 (default 17)
    pub chinese_sentence_compose: bool,
    pub chinese_fuzzy_pinyin: bool,
    pub chinese_emoji: bool,
    pub chinese_learning: bool,
    pub chinese_punctuation: bool,
    pub chinese_jianpin: bool,
    pub vn_free_mark: bool,
    pub vn_auto_english_restore: bool,
    pub vn_spelling_check: bool,
    pub vn_new_accent_style: bool,
    pub run_on_startup: bool,
    pub minimize_to_tray: bool,
    pub show_osd_toast: bool,
    pub theme: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            mode: "Vietnamese".to_string(),
            input_type: "Telex".to_string(),
            charset: "Unicode Dựng sẵn".to_string(),
            hotkey_ctrl: true,
            hotkey_shift: true,
            hotkey_alt: false,
            hotkey_win: false,
            beep_on_switch: false,
            traditional_chinese: false,
            cand_page_size: 5,
            cand_font_size: 17,
            chinese_sentence_compose: true,
            chinese_fuzzy_pinyin: true,
            chinese_emoji: true,
            chinese_learning: true,
            chinese_punctuation: true,
            chinese_jianpin: true,
            vn_free_mark: true,
            vn_auto_english_restore: true,
            vn_spelling_check: true,
            vn_new_accent_style: false,
            run_on_startup: false,
            minimize_to_tray: true,
            show_osd_toast: true,
            theme: "system".to_string(),
        }
    }
}

pub struct ConfigStore {
    config_dir: PathBuf,
}

impl ConfigStore {
    pub fn new(base_dir: PathBuf) -> Self {
        let config_dir = base_dir.join("config");
        let _ = fs::create_dir_all(&config_dir);
        Self { config_dir }
    }

    pub fn load_settings(&self) -> AppSettings {
        let file_path = self.config_dir.join("settings.json");
        if let Ok(content) = fs::read_to_string(&file_path) {
            if let Ok(settings) = serde_json::from_str::<AppSettings>(&content) {
                return settings;
            }
        }
        let default_settings = AppSettings::default();
        let _ = self.save_settings(&default_settings);
        default_settings
    }

    pub fn save_settings(&self, settings: &AppSettings) -> Result<(), String> {
        let file_path = self.config_dir.join("settings.json");
        let json = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
        fs::write(&file_path, json).map_err(|e| e.to_string())?;
        Ok(())
    }
}

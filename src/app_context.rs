/**
 * v1.4.0 04/07/2025
 * Author: Marco Maffei
 * 
 */

use crate::logger::Logger;
use crate::audio_assets::AudioAssets;
use crate::audio_manager::AudioManager;
use crate::Options;

use std::error::Error;
use std::path::Path;

pub struct AppContext {
    pub logger: Option<Logger>,
    pub audio_manager: AudioManager,
    pub audio_assets: AudioAssets,
    pub options: Options,
}

impl AppContext {
    pub fn new(options: Options) -> Self {
        let audio_manager = AudioManager::new();
        let mut audio_assets = AudioAssets::new();
        audio_assets.load_asset("menu_move", "./audio/menu/menu_move_1.wav")
            .expect("Impossibile caricare asset menu_move_1.wav");
        audio_assets.load_asset("menu_click", "./audio/menu/menu_click_1.wav")
            .expect("Impossibile caricare asset menu_click_1.wav");

        AppContext { logger: None, audio_manager, audio_assets, options }
    }

    pub fn set_logger(&mut self, logger: Logger) {
        self.logger = Some(logger);
    }

    pub fn clear_logger(&mut self) {
        self.logger = None;
    }

    pub fn set_audio_asset<K, P>(&mut self, key: K, path: P)
        -> Result<(), Box<dyn Error>>
    where
        K: Into<String>,
        P: AsRef<Path>,
    {
        let key = key.into();
        let path_ref = path.as_ref();

        if path_ref.to_string_lossy().is_empty() {
            self.audio_assets.unload_asset(&key);
            return Ok(());
        }

        let path_str = path_ref
            .to_str()
            .ok_or("Percorso non UTF-8 valido")?;
        self.audio_assets.load_asset(&key, path_str)?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn unload_asset(&mut self, key: &str) {
        if self.audio_assets.unload_asset(key) {
            
        }
    }

    pub fn play_sound(&mut self, key: &str) {
        if self.options.sound_enabled == true {
            if let Some(sample) = self.audio_assets.get(key) {
                self.audio_manager.play_feedback(sample.clone());
            }
        }
    }

    pub fn play_menu_sound(&mut self, key: &str) {
        if self.options.menu_sound_enabled == true {
            if let Some(sample) = self.audio_assets.get(key) {
                self.audio_manager.play_feedback(sample.clone());
            }
        }
    }

}
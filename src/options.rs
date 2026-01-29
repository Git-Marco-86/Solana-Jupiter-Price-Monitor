/**
 * v3.4.0 29/01/2026
 * Author: Marco Maffei
 *
 */

// <custom
use crate::view_type::ViewType;
// </custom

use std::{
    fs::{self, File},
    io::{BufReader, Result as IoResult},
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;


/// Struttura per memorizzare le opzioni di monitoraggio
#[derive(Serialize, Deserialize)]
pub struct Options {
    pub locale: String, // "en", "it", ...
    pub view_type: ViewType,
    pub logs_enabled: bool,
    pub log_file_name: String,
    pub stop_when_price_reached: bool,
    pub menu_sound_enabled: bool,
    pub sound_enabled: bool,
    pub audio_min_price_file_name: String,
    pub audio_max_price_file_name: String,
    pub audio_down_price_file_name: String,
    pub audio_up_price_file_name: String,
    pub chrome_path: String,
    pub url: String,
    pub update_element: String,
    pub input_element: String,
    pub output_element: String,
    pub token_id: String, // TODO: U8? o come ViewType? COMMENTO OBSOLETO PERCHé PROPRIETà CONVERTITA IN STRINGA
    pub token_quantity: u32,
    pub datetime_format: String,
    pub monitoring_interval_secs: u64,
    pub response_timeout_secs: u64,
    pub min_price: f32,
    pub max_price: f32,
    pub down_price_perc: f32,
    pub up_price_perc: f32,
}

impl Options {
    pub fn reset_fields<F>(&mut self, f: F)
    where
        F: FnOnce(&mut Self, Options),
    {
        let defaults = Options::default();
        f(self, defaults);
    }

    /// Carica le opzioni da `dir/file_name`. Se il file non esiste o è malformato,
    /// lo scrive con i valori di default e ritorna `Options::default()`.
    pub fn load<D, F>(dir: D, file_name: F) -> Self
    where
        D: AsRef<Path>,
        F: AsRef<Path>,
    {
        // 1) Costruisco il path completo: dir + file_name
        let mut full_path = PathBuf::from(dir.as_ref());
        full_path.push(file_name.as_ref());

        // 2) Provo ad aprire e deserializzare
        if let Ok(f) = File::open(&full_path) {
            if let Ok(opts) = serde_json::from_reader::<_, Options>(BufReader::new(f)) {
                return opts;
            }
        }

        // 3) Se arrivo qui, il file non esiste o è malformato: scrivo il default
        if let Err(e) = Options::write_default(dir, &file_name) {
            eprintln!("Errore scrittura config su {:?}: {}", full_path, e);
        }

        // 4) Determina quale default restituire in base al nome del file
        let fname = file_name.as_ref().to_string_lossy().to_lowercase();
        let launch_type = if fname.contains("config-2") { 2 } else { 1 };

        Options::default_for(launch_type)
    }

    /// Salva le opzioni su un file JSON.
    pub fn save(file_name: &str, options: &Options) -> std::io::Result<()> {
        let config_file = format!("./config/{}", file_name);
        let json = serde_json::to_string_pretty(options).expect("Errore durante la serializzazione");
        fs::write(config_file, json)
    }

    /// Restituisce i default specifici per un dato launch_type.
    /// launch_type == 2 -> config-2.json style; altrimenti -> default() (config-1 style).
    pub fn default_for(launch_type: u8) -> Self {
        match launch_type {
            2 => Options {
                locale: "it".to_string(),
                view_type: ViewType::FIXED,
                logs_enabled: true,
                log_file_name: "log_1.txt".to_string(),
                stop_when_price_reached: true,
                menu_sound_enabled: true,
                sound_enabled: true,
                audio_min_price_file_name: "fail_5.wav".to_string(),
                audio_max_price_file_name: "win_4.wav".to_string(),
                audio_down_price_file_name: "fail_7.wav".to_string(),
                audio_up_price_file_name: "win_1.wav".to_string(),
                chrome_path: if cfg!(target_os = "windows") {
                    r"C:\Program Files\Google\Chrome\Application\chrome.exe"
                } else {
                    r"/usr/bin/google-chrome"
                }
                .to_string(),
                url: "https://jup.ag/tokens/So11111111111111111111111111111111111111112".to_string(),
                update_element: ".flex.size-7.cursor-pointer.items-center.justify-center.rounded-full.border.border-neutral-800.text-neutral-500.hover\\:bg-neutral-800.hover\\:text-neutral-300".to_string(),
                input_element: ".flex.shrink-0.items-center.overflow-hidden.py-2\\.5.pr-2\\.5 > .flex.flex-1.justify-between.gap-2\\.5.overflow-hidden > .flex.shrink-0.flex-col.items-end.justify-center.gap-0\\.5 > .relative.inline-flex.items-center.rounded-sm.leading-none.tracking-tight > span".to_string(),
                output_element: ".flex.flex-1.justify-between.overflow-hidden > .flex.shrink-0.flex-col.items-end.gap-1 > .relative.inline-flex.items-center.rounded-sm.leading-none > span".to_string(),
                token_id: "SOL".to_string(),
                token_quantity: 1,
                datetime_format: "%d-%m-%Y %H:%M:%S".to_string(),
                monitoring_interval_secs: 1,
                response_timeout_secs: 2,
                min_price: 0.0,
                max_price: 0.0,
                down_price_perc: 0.0,
                up_price_perc: 0.0,
            },
            _ => Options::default(),
        }
    }

    /// Scrive le opzioni di default nel file specificato.
    /// Se il nome del file contiene "config-2" usa i default per launch_type 2.
    pub fn write_default<D, F>(dir: D, file_name: F) -> IoResult<()>
    where
        D: AsRef<Path>,
        F: AsRef<Path>,
    {
        // Determina launch_type dal nome del file (semplice heuristica)
        let fname = file_name.as_ref().to_string_lossy().to_lowercase();
        let launch_type = if fname.contains("config-2") { 2 } else { 1 };

        // Prendi i default corretti
        let default = Options::default_for(launch_type);

        // Serializza e scrivi
        let s = serde_json::to_string_pretty(&default)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        let mut full_path = PathBuf::from(dir.as_ref());

        #[cfg_attr(windows, allow(unused_variables))]
        let _ = std::fs::create_dir_all(dir.as_ref());

        #[cfg(unix)]
        {
            if let Err(e) = fs::set_permissions(&full_path, fs::Permissions::from_mode(0o777)) {
                eprintln!("⚠️ errore nell'impostare i permessi su {:?}: {}", full_path, e);
            }
        }

        full_path.push(file_name);
        let _ = std::fs::write(&full_path, s);

        #[cfg(unix)]
        {
            if let Err(e) = fs::set_permissions(&full_path, fs::Permissions::from_mode(0o777)) {
                eprintln!("⚠️ errore nell'impostare i permessi su {:?}: {}", full_path, e);
            }
        }

        Ok(())
    }
}

/// Implementazione dei valori di default per Options
impl Default for Options {
    fn default() -> Self {
        Options {
            locale: "it".to_string(),
            view_type: ViewType::FIXED,
            logs_enabled: true,
            log_file_name: "log_1.txt".to_string(), // TODO: NON USATA
            stop_when_price_reached: true,
            menu_sound_enabled: true,
            sound_enabled: true,
            audio_min_price_file_name: "fail_5.wav".to_string(),
            audio_max_price_file_name: "win_4.wav".to_string(),
            audio_down_price_file_name: "fail_7.wav".to_string(),
            audio_up_price_file_name: "win_1.wav".to_string(),
            chrome_path: if cfg!(target_os = "windows") {
                r"C:\Program Files\Google\Chrome\Application\chrome.exe"
            } else {
                r"/usr/bin/google-chrome"
            }
            .to_string(),
            url: "https://jup.ag/swap?sell=:sell_addr&buy=:buy_addr".to_string(),
            update_element: ".outline-none.flex.size-7.cursor-pointer.items-center.justify-center.rounded-full.text-neutral-500.hover\\:bg-neutral-800.hover\\:text-neutral-300".to_string(),
            input_element: "input[name=fromValue]".to_string(),
            output_element: "input[name=toValue]".to_string(),
            token_id: "SOL".to_string(),
            token_quantity: 1,
            datetime_format: "%d-%m-%Y %H:%M:%S".to_string(),
            monitoring_interval_secs: 1,
            response_timeout_secs: 2,
            min_price: 0.0,
            max_price: 0.0,
            down_price_perc: 0.0,
            up_price_perc: 0.0,
        }
    }
}
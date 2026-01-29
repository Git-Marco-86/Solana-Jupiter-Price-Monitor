/**
 * v1.0.1 29/01/2026
 * Author: Marco Maffei
 * 
 */

use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{BufReader, Result as IoResult},
    path::{Path, PathBuf},
};

#[cfg(unix)]
use std::{
    fs,
    os::unix::fs::PermissionsExt
};


#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigRun {
    pub config_file_name: String,
    pub launch_type: u8,
}

impl Default for ConfigRun {
    fn default() -> Self {
        ConfigRun {
            config_file_name: "config-2.json".into(),
            launch_type: 2,
        }
    }
}

impl ConfigRun {
    /// Carica la config da `path`, altrimenti ritorna i valori di default.
    pub fn load<P: AsRef<Path>>(path: P) -> Self {
        if let Ok(f) = File::open(&path) {
            if let Ok(cfg) = serde_json::from_reader(BufReader::new(f)) {
                return cfg;
            }
        }

        if let Err(e) = ConfigRun::write_default("config-run.json") {
            eprintln!("Errore scrittura config: {}", e);
        }

        ConfigRun::default()
    }

    /// Scrive sul disco un file JSON con i valori di default.
    pub fn write_default<P: AsRef<Path>>(file_name: P) -> IoResult<()> {
        let cfg = ConfigRun::default();
        let s   = serde_json::to_string_pretty(&cfg)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        let mut full_path = PathBuf::from("./config");

        #[cfg_attr(windows, allow(unused_variables))]
        let dir = std::fs::create_dir_all("./config");

        #[cfg(unix)]
        {
            if let Err(e) = fs::set_permissions(&full_path, fs::Permissions::from_mode(0o777)) {
                eprintln!("⚠️ errore nell'impostare i permessi su {:?}: {}", dir, e);
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
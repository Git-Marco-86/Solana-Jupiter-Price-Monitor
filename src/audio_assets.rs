/**
 * v1.1.0 04/07/2025
 * Author: Marco Maffei
 * 
 */

use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::io::Cursor;
use std::path::Path;
use rodio::{Decoder, source::Buffered, Source};

// Creiamo un alias per il tipo di sample precaricato
type AudioSample = Buffered<Decoder<Cursor<Vec<u8>>>>;

/// AudioAssets gestisce più suoni precaricati associati a una chiave (ad esempio, String)
pub struct AudioAssets {
    samples: HashMap<String, AudioSample>,
}

impl AudioAssets {
    /// Crea un nuovo AudioAssets manager vuoto.
    pub fn new() -> Self {
        AudioAssets {
            samples: HashMap::new(),
        }
    }

    /// Carica un asset audio da un file, lo decodifica e bufferizza, e lo salva sotto il nome specificato.
    pub fn load_asset(&mut self, key: impl Into<String>, path: &str) -> Result<(), Box<dyn Error>> {
        let bytes = fs::read(path)?;
        let decoder = Decoder::new(Cursor::new(bytes))?;
        let sample = decoder.buffered();
        self.samples.insert(key.into(), sample);
        Ok(())
    }

    /// Rimuove l’asset associato a `key`, se presente.
    /// Ritorna `true` se l’asset esisteva e è stato rimosso.
    pub fn unload_asset(&mut self, key: &str) -> bool {
        self.samples.remove(key).is_some()
    }

    /// Restituisce in riferimento il sample associato alla chiave fornita.
    pub fn get(&self, key: &str) -> Option<&AudioSample> {
        self.samples.get(key)
    }

    pub fn get_audio_files<P: AsRef<Path>>(directory: P) -> std::io::Result<Vec<String>> {
        let mut files = Vec::new();
        for entry in fs::read_dir(directory)? {
            let entry = entry?;
            let path = entry.path();
            // Se vuoi filtrare per estensione, ad esempio per file con estensione ".wav":
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == "wav" {
                        if let Some(file_name) = path.file_name().and_then(|s| s.to_str()) {
                            files.push(file_name.to_string());
                        }
                    }
                }
            }
        }
        Ok(files)
    }
}
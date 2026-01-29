/**
 * v1.2.0 30/06/2025
 * Author: Marco Maffei
 *
 */

use std::io::{BufWriter, Write};
use std::fs::OpenOptions;
use std::thread;
use std::sync::mpsc::{self, Sender};
use std::path::PathBuf;
use chrono::Local;

#[cfg(unix)]
use std::{
    fs,
    os::unix::fs::PermissionsExt
};


/// Logger che spedisce i messaggi su un thread in background.
pub struct Logger {
    tx: Sender<String>,  // il canale per inviare i log
}

impl Logger {

    pub fn new() -> Self {
        Self::new_with_filename(None)
    }

    #[allow(dead_code)]
    pub fn new_with_file(name: &str) -> Self {
        Self::new_with_filename(Some(name))
    }

    /// Factory interna che accetta un nome personalizzato, oppure None
    /// per farne generare uno con `make_log_filename()`.
    pub fn new_with_filename(name: Option<&str>) -> Self {
        let (tx, rx) = mpsc::channel::<String>();
        let filename = name
            .map(str::to_string)
            .unwrap_or_else(Self::make_log_filename);
        let mut full_path = PathBuf::from("./logs");

        #[cfg_attr(windows, allow(unused_variables))]
        let dir = std::fs::create_dir_all("./logs");

        #[cfg(unix)]
        {
            let perms = fs::Permissions::from_mode(0o777);
            if let Err(e) = fs::set_permissions(&full_path, perms) {
                eprintln!("⚠️ errore nell'impostare i permessi su {:?}: {}", dir, e);
            }
        }

        /* // TODO:
        #[cfg(windows)]
        {
            // chiama la tua funzione Windows-only per settare ACL
            // win_perms::set_full_control_everyone(path)?;
        }
        */

        full_path.push(filename);

        thread::spawn(move || {
            // Apre (o crea) il file in append mode
            let file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(full_path)
                .expect("Impossibile aprire il file di log");

            let mut writer = BufWriter::new(file);

            // Continua finché arriva qualche messaggio
            while let Ok(line) = rx.recv() {
                // Scrive e forza il flush per avere subito il log su disco
                if let Err(e) = writeln!(writer, "{}", line) {
                    eprintln!("Logger error: {}", e);
                }
                let _ = writer.flush();
            }

            // Quando `tx` viene droppato, rx.recv() fallisce e il thread esce.
        });

        Logger { tx }
    }

    /// Invia un messaggio al logger. È non‐bloccante:
    /// se il canale è pieno (=okk se hai usato `channel()` senza bound),
    /// ritorna subito l’errore su `send`.
    pub fn log(&self, msg: String) {
        // puoi ignorare l’Err perché significa “logger già chiuso”
        let _ = self.tx.send(msg);
    }

    pub fn make_log_filename() -> String {
        let now = Local::now();
        // %d = giorno (2 cifre), %m = mese (2 cifre), %Y = anno (4 cifre)
        // %H = ora (00–23), %M = minuti, %S = secondi
        format!("log_{}.{}", now.format("%d%m%Y_%H%M%S"), "txt")
    }

}

impl Drop for Logger {
    fn drop(&mut self) {
        // Qui droppiamo tx, il thread si accorgerà di chiusura e uscirà
        // (nessun join, thread “detached” terminerà da solo)
    }
}
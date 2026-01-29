/**
 * v1.0.0 28/06/2025
 * Author: Marco Maffei
 * 
 */

use std::io::{stdout, Write};
use std::sync::{Arc, atomic::{Ordering, AtomicBool}};
use std::time::Duration;
use std::thread::{self, JoinHandle};

pub struct Spinner {
  msg: String,
  running: Arc<AtomicBool>,
  handle: Option<JoinHandle<()>>,
}

impl Spinner {
  /// Crea e avvia immediatamente lo spinner con `msg` + puntini animati
  pub fn start(msg: impl Into<String>) -> Self {
    let msg = msg.into();
    let running = Arc::new(AtomicBool::new(true));
    let flag = running.clone();
    let thread_msg = msg.clone();

    let handle = thread::spawn(move || {
      let mut frame = 0;
      let frames = ["   ", ".  ", ".. ", "..."];
      while flag.load(Ordering::SeqCst) {
        // sovrascrive la stessa riga
        print!("\r{}{}", thread_msg, frames[frame % frames.len()]);
        stdout().flush().ok();
        frame = frame.wrapping_add(1);
        thread::sleep(Duration::from_millis(400));
      }
    });

    Spinner {
      msg,
      running,
      handle: Some(handle),
    }
  }

  pub fn stop(mut self) {
    self.running.store(false, Ordering::SeqCst);
    if let Some(h) = self.handle.take() {
      let _ = h.join();
    }
    println!("\r{}... completato!    \r", self.msg);
  }
}
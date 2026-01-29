/**
 * v1.3.0 23/06/2025
 * Author: Marco Maffei
 * 
 */

use std::time::Duration;
use std::fs::File;
use std::io::BufReader;
use std::thread;

use rodio::{Decoder, OutputStream, Sink, source::{SineWave, Source}};

struct SoundData {
  _stream: OutputStream, // necessario per tenere lo stream in vita
  sink: Sink,
}

#[allow(dead_code)]
pub fn play_beep() {
  // 1) apri il default audio output
  let (_stream, stream_handle) = OutputStream::try_default()
      .expect("Impossibile aprire l'output audio");

  // 2) crea un “sink” per inviarvi il suono
  let sink = Sink::try_new(&stream_handle)
      .expect("Impossibile creare il sink audio");

  // 3) genera un’onda seno a 440 Hz, 20% volume, durata 200ms
  let source = SineWave::new(440.0)
      .amplify(0.2)
      .take_duration(Duration::from_millis(200));

  // 4) appendi al sink e attendi la fine
  sink.append(source);
  sink.detach();    // non blocca il thread, suona in background
}

#[allow(dead_code)]
pub fn play_audio(path: String, wait_for_end: bool) {
  // Crea uno stream audio di output (es. casse del computer)
  let (stream, stream_handle) = OutputStream::try_default().unwrap();

  // Crea un sink (gestisce il playback)
  let sink = Sink::try_new(&stream_handle).unwrap();

  // Apri il file audio
  let file = File::open(path).unwrap(); // Assicurati che il file sia nella stessa cartella o specifica il path completo
  let source = Decoder::new(BufReader::new(file)).unwrap();

  // Aggiungi il suono al sink e riproducilo
  sink.append(source);

  if wait_for_end == true { // Blocca l'esecuzione finché il suono non è finito
    sink.sleep_until_end();
  } else {
    // Sposta il sink (insieme allo stream, che deve rimanere in vita) in un thread separato
    // per permettere la riproduzione asincrona del suono
    
    // Raggruppiamo lo stream e il sink in una struttura per mantenerli attivi
    let sound_data = SoundData {
      _stream: stream,
      sink,
    };

    // Spostiamo SoundData in un thread separato
    thread::spawn(move || {
      sound_data.sink.sleep_until_end();
      // Quando il sink ha terminato la riproduzione, anche lo stream viene droppato.
      // Non serve fare niente qui.
    });
    // La funzione ritorna immediatamente, lasciando il thread separato far terminare il suono.
  }
}

#[allow(dead_code)]
pub fn play_system_beep() {
  print!("\x07");
}
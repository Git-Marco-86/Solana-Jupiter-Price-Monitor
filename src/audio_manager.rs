/**
 * v1.3.0 27/06/2025
 * Author: Marco Maffei
 * 
 */

use std::fs::{self, File};
use std::io::{BufReader, Cursor};
use std::time::Duration;
use std::marker::PhantomData;

use rodio::{Decoder, Source, OutputStream, Sink, Sample, source::Buffered};

use num_traits::NumCast;


pub struct AudioManager {
    pub _stream: OutputStream,
    pub stream_handle: rodio::OutputStreamHandle,
    pub active_sinks: Vec<Sink>,
    pub feedback_sink: Sink,
    current_sink: Option<Sink>,
}

impl AudioManager {
    pub fn new() -> Self {
        let (stream, stream_handle) = OutputStream::try_default().unwrap();
        let feedback_sink = Sink::try_new(&stream_handle).unwrap();
        AudioManager {
            _stream: stream,
            stream_handle,
            active_sinks: Vec::new(),
            feedback_sink,
            current_sink: None,
        }
    }

    /// Carica il file, decodifica, converte i campioni in f32 e bufferizza il risultato.
    #[allow(dead_code)]
    pub fn load_buffered_sample(path: &str) -> Result<Buffered<impl Source<Item = f32> + Send>, Box<dyn std::error::Error>> {
        let bytes = fs::read(path)?;
        let decoder = Decoder::new(Cursor::new(bytes))?;
        // Dopo la decodifica, convertiamo i campioni in f32.
        Ok(decoder.convert_samples::<f32>().buffered())
    }

    /// Riproduce un sample audio nel sink di feedback.
    /// Si assume che il sample sia di tipo Buffered e implementi Clone in maniera leggera.
    pub fn play_feedback<S>(&mut self, sample: S)
    where
        S: Source<Item = i16> + Clone + Send + 'static,
    {
        self.feedback_sink.stop(); // Pulisce eventuali dati residui
        // Converte i campioni da i16 a f32 e li appende al sink:
        self.feedback_sink.append(sample.clone().convert_samples::<f32>());
    }

    pub fn play_audio(&mut self, path: &str) {
        if let Some(old) = self.current_sink.take() {
            old.stop();
        }

        let sink = Sink::try_new(&self.stream_handle)
            .expect("failed to create Sink");
        let file = File::open(path)
            .expect("failed to open audio file");
        let source = Decoder::new(BufReader::new(file))
            .expect("failed to decode audio");
        sink.append(source);

        self.current_sink = Some(sink);
    }

    #[allow(dead_code)]
    pub fn play_audio_sample<S>(&mut self, sample: S)
    where
        S: Source + Send + 'static,
        S::Item: Sample + NumCast,
    {
        let sink = rodio::Sink::try_new(&self.stream_handle).unwrap();
        let converted = F32Converter::new(sample);
        sink.append(converted);
        self.active_sinks.push(sink);
    }

    #[allow(dead_code)]
    pub fn play_audio_sample_i16<S>(&mut self, sample: S)
    where
        S: Source<Item = i16> + Send + 'static,
    {
        let sink = Sink::try_new(&self.stream_handle).unwrap();
        let adapter = F32AdapterI16::new(sample);
        sink.append(adapter);
        self.active_sinks.push(sink);
    }
}

/// Adapter che converte i campioni dello source `S` nel tipo f32.
pub struct F32Converter<S>
where
    S: Source,
    S::Item: NumCast + Sample,
{
    inner: S,
    _marker: PhantomData<f32>,
}

impl<S> F32Converter<S>
where
    S: Source,
    S::Item: NumCast + Sample,
{
    #[allow(dead_code)]
    pub fn new(source: S) -> Self {
        Self {
            inner: source,
            _marker: PhantomData,
        }
    }
}

// Implementiamo Iterator per F32Converter.
impl<S> Iterator for F32Converter<S>
where
    S: Source,
    S::Item: NumCast + Sample,
{
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().and_then(|sample| NumCast::from(sample))
    }
}

// Implementiamo Source per F32Converter delegando le funzioni allo source interno.
impl<S> Source for F32Converter<S>
where
    S: Source,
    S::Item: NumCast + Sample,
{
    #[inline]
    fn current_frame_len(&self) -> Option<usize> {
        self.inner.current_frame_len()
    }

    #[inline]
    fn channels(&self) -> u16 {
        self.inner.channels()
    }

    #[inline]
    fn sample_rate(&self) -> u32 {
        self.inner.sample_rate()
    }

    #[inline]
    fn total_duration(&self) -> Option<Duration> {
        self.inner.total_duration()
    }
}

pub struct F32AdapterI16<S>
where
    S: Source<Item = i16>,
{
    inner: S,
}

impl<S> F32AdapterI16<S>
where
    S: Source<Item = i16>,
{
    #[allow(dead_code)]
    pub fn new(source: S) -> Self {
        Self { inner: source }
    }
}

impl<S> Iterator for F32AdapterI16<S>
where
    S: Source<Item = i16>,
{
    // Restituirà campioni normalizzati in f32: nell'intervallo [-1.0, 1.0].
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        // Per normalizzare dividiamo per 32768.0.
        self.inner.next().map(|sample| sample as f32 / 32768.0)
    }
}

impl<S> Source for F32AdapterI16<S>
where
    S: Source<Item = i16>,
{
    fn current_frame_len(&self) -> Option<usize> {
        self.inner.current_frame_len()
    }

    fn channels(&self) -> u16 {
        self.inner.channels()
    }

    fn sample_rate(&self) -> u32 {
        self.inner.sample_rate()
    }

    fn total_duration(&self) -> Option<Duration> {
        self.inner.total_duration()
    }
}
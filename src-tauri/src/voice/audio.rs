use crate::error::{AppError, AppResult};
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    SampleFormat,
};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Sender};

#[derive(Debug, Clone)]
pub struct RecordedAudio {
    pub pcm: Vec<i16>,
    pub sample_rate: u32,
}

pub struct AudioRecorder {
    stop_tx: Option<Sender<()>>,
    samples: Arc<Mutex<Vec<i16>>>,
    sample_rate: u32,
}

impl AudioRecorder {
    pub fn start() -> AppResult<Self> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or_else(|| AppError::Audio("no default microphone input device".to_string()))?;
        let config = device
            .default_input_config()
            .map_err(|error| AppError::Audio(error.to_string()))?;
        let sample_rate = config.sample_rate().0;
        let samples = Arc::new(Mutex::new(Vec::new()));
        let writer = samples.clone();
        
        let (stop_tx, stop_rx) = channel::<()>();
        let (ready_tx, ready_rx) = channel::<Result<(), String>>();

        std::thread::spawn(move || {
            let error_callback = |error| log::error!("audio input stream failed: {error}");
            
            let stream_result = match config.sample_format() {
                SampleFormat::I16 => device.build_input_stream(
                    &config.into(),
                    move |data: &[i16], _| push_i16_samples(&writer, data.iter().copied()),
                    error_callback,
                    None,
                ),
                SampleFormat::U16 => device.build_input_stream(
                    &config.into(),
                    move |data: &[u16], _| push_i16_samples(&writer, data.iter().map(|sample| *sample as i32 - 32768).map(|sample| sample as i16)),
                    error_callback,
                    None,
                ),
                SampleFormat::F32 => device.build_input_stream(
                    &config.into(),
                    move |data: &[f32], _| push_i16_samples(&writer, data.iter().map(|sample| (*sample * i16::MAX as f32) as i16)),
                    error_callback,
                    None,
                ),
                other => {
                    let _ = ready_tx.send(Err(format!("unsupported microphone sample format: {other:?}")));
                    return;
                }
            };

            let stream = match stream_result {
                Ok(s) => s,
                Err(e) => {
                    let _ = ready_tx.send(Err(e.to_string()));
                    return;
                }
            };

            if let Err(e) = stream.play() {
                let _ = ready_tx.send(Err(e.to_string()));
                return;
            }

            let _ = ready_tx.send(Ok(()));
            
            // Block until stop signal is received
            let _ = stop_rx.recv();
            
            // Stream will be dropped here
        });

        match ready_rx.recv() {
            Ok(Ok(())) => {
                Ok(Self {
                    stop_tx: Some(stop_tx),
                    samples,
                    sample_rate,
                })
            }
            Ok(Err(msg)) => Err(AppError::Audio(msg)),
            Err(_) => Err(AppError::Audio("audio thread crashed during setup".to_string())),
        }
    }

    pub fn stop(mut self) -> RecordedAudio {
        if let Some(tx) = self.stop_tx.take() {
            let _ = tx.send(());
        }
        let pcm = self.samples.lock().map(|samples| samples.clone()).unwrap_or_default();
        RecordedAudio {
            pcm,
            sample_rate: self.sample_rate,
        }
    }
}

fn push_i16_samples(samples: &Arc<Mutex<Vec<i16>>>, incoming: impl Iterator<Item = i16>) {
    if let Ok(mut buffer) = samples.lock() {
        buffer.extend(incoming);
    }
}

pub fn resample_to_16khz(audio: &RecordedAudio) -> RecordedAudio {
    if audio.sample_rate == 16_000 || audio.pcm.is_empty() {
        return audio.clone();
    }

    let ratio = audio.sample_rate as f64 / 16_000.0;
    let target_len = (audio.pcm.len() as f64 / ratio).ceil() as usize;
    let mut pcm = Vec::with_capacity(target_len);
    for index in 0..target_len {
        let source_index = (index as f64 * ratio).floor() as usize;
        if let Some(sample) = audio.pcm.get(source_index) {
            pcm.push(*sample);
        }
    }

    RecordedAudio {
        pcm,
        sample_rate: 16_000,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resample_to_16khz_keeps_16khz_audio_unchanged() {
        let audio = RecordedAudio {
            pcm: vec![1, 2, 3],
            sample_rate: 16_000,
        };

        let resampled = resample_to_16khz(&audio);

        assert_eq!(resampled.sample_rate, 16_000);
        assert_eq!(resampled.pcm, vec![1, 2, 3]);
    }

    #[test]
    fn resample_to_16khz_downsamples_by_position() {
        let audio = RecordedAudio {
            pcm: (0..48).collect(),
            sample_rate: 48_000,
        };

        let resampled = resample_to_16khz(&audio);

        assert_eq!(resampled.sample_rate, 16_000);
        assert_eq!(resampled.pcm.len(), 16);
        assert_eq!(resampled.pcm[1], 3);
    }
}

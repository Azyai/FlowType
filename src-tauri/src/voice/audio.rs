use crate::error::{AppError, AppResult};
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    SampleFormat,
};
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub struct AudioRecorder {
    stop_tx: Option<Sender<()>>,
}

impl AudioRecorder {
    pub fn start(
        on_level: impl Fn(f32) + Send + Sync + 'static,
        on_samples: impl Fn(Vec<i16>, u32) + Send + Sync + 'static,
    ) -> AppResult<Self> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or_else(|| AppError::Audio("no default microphone input device".to_string()))?;
        let config = device
            .default_input_config()
            .map_err(|error| AppError::Audio(error.to_string()))?;
        let sample_rate = config.sample_rate().0;
        let channel_count = config.channels() as usize;
        let on_level = Arc::new(on_level);
        let on_samples = Arc::new(on_samples);
        let last_level_emit = Arc::new(Mutex::new(Instant::now().checked_sub(Duration::from_millis(80)).unwrap_or_else(Instant::now)));
        
        let (stop_tx, stop_rx) = channel::<()>();
        let (ready_tx, ready_rx) = channel::<Result<(), String>>();

        std::thread::spawn(move || {
            let error_callback = |error| log::error!("audio input stream failed: {error}");
            
            let stream_result = match config.sample_format() {
                SampleFormat::I16 => {
                    let on_level = on_level.clone();
                    let on_samples = on_samples.clone();
                    let last_level_emit = last_level_emit.clone();
                    device.build_input_stream(
                        &config.into(),
                        move |data: &[i16], _| {
                            let mono = interleaved_to_mono_i16(data, channel_count);
                            emit_input_level(&on_level, &last_level_emit, mono.iter().copied());
                            on_samples(mono, sample_rate);
                        },
                        error_callback,
                        None,
                    )
                }
                SampleFormat::U16 => {
                    let on_level = on_level.clone();
                    let on_samples = on_samples.clone();
                    let last_level_emit = last_level_emit.clone();
                    device.build_input_stream(
                        &config.into(),
                        move |data: &[u16], _| {
                            let converted = data
                                .iter()
                                .map(|sample| *sample as i32 - 32768)
                                .map(|sample| sample as i16)
                                .collect::<Vec<_>>();
                            let mono = interleaved_to_mono_i16(&converted, channel_count);
                            emit_input_level(&on_level, &last_level_emit, mono.iter().copied());
                            on_samples(mono, sample_rate);
                        },
                        error_callback,
                        None,
                    )
                }
                SampleFormat::F32 => {
                    let on_level = on_level.clone();
                    let on_samples = on_samples.clone();
                    let last_level_emit = last_level_emit.clone();
                    device.build_input_stream(
                        &config.into(),
                        move |data: &[f32], _| {
                            let converted = data
                                .iter()
                                .map(|sample| (*sample * i16::MAX as f32) as i16)
                                .collect::<Vec<_>>();
                            let mono = interleaved_to_mono_i16(&converted, channel_count);
                            emit_input_level(&on_level, &last_level_emit, mono.iter().copied());
                            on_samples(mono, sample_rate);
                        },
                        error_callback,
                        None,
                    )
                }
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
                })
            }
            Ok(Err(msg)) => Err(AppError::Audio(msg)),
            Err(_) => Err(AppError::Audio("audio thread crashed during setup".to_string())),
        }
    }

    pub fn stop(mut self) {
        if let Some(tx) = self.stop_tx.take() {
            let _ = tx.send(());
        }
    }
}

fn interleaved_to_mono_i16(samples: &[i16], channel_count: usize) -> Vec<i16> {
    if channel_count <= 1 || samples.is_empty() {
        return samples.to_vec();
    }

    samples
        .chunks(channel_count)
        .map(|frame| {
            let sum = frame.iter().map(|sample| *sample as i32).sum::<i32>();
            (sum / frame.len() as i32) as i16
        })
        .collect()
}

fn emit_input_level(
    on_level: &Arc<impl Fn(f32) + Send + Sync + 'static>,
    last_level_emit: &Arc<Mutex<Instant>>,
    incoming: impl Iterator<Item = i16>,
) {
    let samples = incoming.collect::<Vec<_>>();
    if samples.is_empty() {
        return;
    }

    let Ok(mut last_emit) = last_level_emit.lock() else {
        return;
    };
    if last_emit.elapsed() < Duration::from_millis(70) {
        return;
    }
    *last_emit = Instant::now();

    on_level(normalized_rms_level(&samples));
}

fn normalized_rms_level(samples: &[i16]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }

    let mean_square = samples
        .iter()
        .map(|sample| {
            let normalized = *sample as f32 / i16::MAX as f32;
            normalized * normalized
        })
        .sum::<f32>()
        / samples.len() as f32;
    mean_square.sqrt().powf(0.65).clamp(0.0, 1.0)
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalized_rms_level_is_zero_for_silence() {
        assert_eq!(normalized_rms_level(&[0, 0, 0, 0]), 0.0);
    }

    #[test]
    fn normalized_rms_level_grows_with_louder_samples() {
        let quiet = normalized_rms_level(&[300, -300, 300, -300]);
        let loud = normalized_rms_level(&[8_000, -8_000, 8_000, -8_000]);

        assert!(quiet > 0.0);
        assert!(loud > quiet);
        assert!(loud <= 1.0);
    }

    #[test]
    fn mono_conversion_keeps_single_channel_audio() {
        let samples = vec![1, -2, 3, -4];

        assert_eq!(interleaved_to_mono_i16(&samples, 1), samples);
    }

    #[test]
    fn mono_conversion_averages_interleaved_frames() {
        let stereo = [1000, 3000, -2000, 2000, 500, 1500];

        assert_eq!(interleaved_to_mono_i16(&stereo, 2), vec![2000, 0, 1000]);
    }
}

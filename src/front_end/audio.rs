use hound::{WavSpec, WavWriter};
use nes_rs::{bus::AUDIO_BUFFER_SIZE, ring_buffer::RingBuffer};
use sdl2::audio::{AudioCallback, AudioDevice, AudioSpecDesired};
use std::{
    fs::File,
    io::BufWriter,
    sync::{Arc, Mutex},
};

use crate::front_end::FrontEndState;

pub struct AudioWrapper {
    last_sample: f32,
    #[allow(clippy::type_complexity)]
    func: Box<dyn FnMut(&mut f32, &mut [f32]) + Send>,
}

impl AudioCallback for AudioWrapper {
    type Channel = f32;
    fn callback(&mut self, out: &mut [f32]) {
        (self.func)(&mut self.last_sample, out);
    }
}

pub type ConcurrentWavWriter = Option<Arc<Mutex<Option<WavWriter<BufWriter<File>>>>>>;

pub struct AudioDeviceWrapper {
    pub audio_device: AudioDevice<AudioWrapper>,
    pub wav_writer: ConcurrentWavWriter,
}

impl AudioDeviceWrapper {
    pub fn new(
        front_end_state: &FrontEndState,
        audio_buffer: Arc<Mutex<RingBuffer<f32, AUDIO_BUFFER_SIZE>>>,
    ) -> Self {
        let desired_spec = AudioSpecDesired {
            freq: Some(44100),
            channels: Some(1),
            samples: Some(1024),
        };

        let audio_device = front_end_state
            .audio_subsystem
            .open_playback(None, &desired_spec, |_spec| AudioWrapper {
                last_sample: 0f32,
                func: Box::new(move |last_sample, out: &mut [f32]| {
                    let mut buf = audio_buffer.lock().unwrap();
                    for x in out {
                        let sample = buf.next().unwrap_or(*last_sample);
                        *last_sample = sample;
                        *x = sample;
                    }
                }),
            })
            .unwrap();

        Self {
            audio_device,
            wav_writer: None,
        }
    }

    pub fn new_recording(
        front_end_state: &FrontEndState,
        output_path: &str,
        audio_buffer: Arc<Mutex<RingBuffer<f32, AUDIO_BUFFER_SIZE>>>,
    ) -> Self {
        let desired_spec = AudioSpecDesired {
            freq: Some(44100),
            channels: Some(1),
            samples: Some(1024),
        };

        let wav_spec = WavSpec {
            channels: 1,
            sample_rate: 44100,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };

        let wav = Arc::new(Mutex::new(Some(
            WavWriter::create(output_path, wav_spec).unwrap(),
        )));
        let wav_clone = wav.clone();

        let audio_device = front_end_state
            .audio_subsystem
            .open_playback(None, &desired_spec, |_spec| AudioWrapper {
                last_sample: 0f32,
                func: Box::new(move |last_sample, out: &mut [f32]| {
                    let mut buf = audio_buffer.lock().unwrap();
                    let mut wav = wav_clone.lock().unwrap();
                    for x in out {
                        let sample = buf.next().unwrap_or(*last_sample);
                        *last_sample = sample;
                        *x = sample;
                        let sample_i16 = (sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
                        wav.as_mut().unwrap().write_sample(sample_i16).unwrap();
                    }
                }),
            })
            .unwrap();

        Self {
            audio_device,
            wav_writer: Some(wav),
        }
    }
}

impl Drop for AudioDeviceWrapper {
    /// writes back the recoding if the audio was recorded
    fn drop(&mut self) {
        let writer = {
            if let Some(writer) = self.wav_writer.as_ref() {
                let mut guard = writer.lock().unwrap();
                guard.take()
            } else {
                None
            }
        };
        if let Some(writer) = writer {
            writer.finalize().unwrap();
        }
    }
}

use std::error::Error;

use chip8_core::{Audio, Chip8Error};
use sdl2::{
    audio::{AudioCallback, AudioDevice, AudioSpecDesired},
    Sdl,
};

pub struct SdlAudio {
    audio_device: AudioDevice<SquareWave>,
}

impl SdlAudio {
    pub fn new(sdl_context: &Sdl) -> Result<SdlAudio, Box<dyn Error>> {
        let audio_subsystem = sdl_context.audio()?;
        let audio_spec = AudioSpecDesired {
            freq: Some(44100),
            channels: Some(1),
            samples: None,
        };
        let audio_device = audio_subsystem.open_playback(None, &audio_spec, |spec| SquareWave {
            phase_inc: 440.0 / spec.freq as f32,
            phase: 0.0,
            volume: 0.25,
        })?;

        Ok(SdlAudio { audio_device })
    }
}

impl Audio for SdlAudio {
    fn play(&self) -> Result<(), Chip8Error> {
        self.audio_device.resume();
        Ok(())
    }

    fn stop(&self) -> Result<(), Chip8Error> {
        self.audio_device.pause();
        Ok(())
    }
}

struct SquareWave {
    phase_inc: f32,
    phase: f32,
    volume: f32,
}

impl AudioCallback for SquareWave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        // Generate a square wave
        for x in out.iter_mut() {
            *x = if self.phase <= 0.5 {
                self.volume
            } else {
                -self.volume
            };
            self.phase = (self.phase + self.phase_inc) % 1.0;
        }
    }
}

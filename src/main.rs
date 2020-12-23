mod chip8;
mod font_set;
mod test_utils;
mod types;

use std::{error::Error, thread, time::Duration};

use chip8::Chip8;
use sdl2::{
    audio::{AudioCallback, AudioSpecDesired, AudioStatus},
    event::Event,
    keyboard::Keycode,
    pixels::Color,
    rect::Rect,
};

fn main() -> Result<(), Box<dyn Error>> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("chip8", 640, 320)
        .position_centered()
        .opengl()
        .build()?;

    let audio_subsystem = sdl_context.audio().unwrap();
    let audio_spec = AudioSpecDesired {
        freq: Some(44100),
        channels: Some(1),
        samples: None,
    };
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

    let audio_device = audio_subsystem
        .open_playback(None, &audio_spec, |spec| SquareWave {
            phase_inc: 440.0 / spec.freq as f32,
            phase: 0.0,
            volume: 0.25,
        })
        .unwrap();

    let mut canvas = window.into_canvas().build()?;
    let mut event_pump = sdl_context.event_pump()?;

    let mut chip8 = Chip8::new(Box::new(rand::thread_rng()));
    chip8.initialize();
    chip8.load_program("Space Invaders.ch8")?;

    let mut frame = 1;
    'main: loop {
        let on_wait_key_event = || {
            let key_pressed = match event_pump.wait_event() {
                Event::KeyDown { keycode, .. } => keycode.unwrap(),
                _ => panic!("Crashed while waiting for event"),
            };

            match key_pressed {
                Keycode::Num1 => 0x1,
                Keycode::Num2 => 0x2,
                Keycode::Num3 => 0x3,
                Keycode::Num4 => 0xC,
                Keycode::Q => 0x4,
                Keycode::W => 0x5,
                Keycode::E => 0x6,
                Keycode::R => 0xD,
                Keycode::A => 0x7,
                Keycode::S => 0x8,
                Keycode::D => 0x9,
                Keycode::F => 0xE,
                Keycode::Z => 0xA,
                Keycode::X => 0x0,
                Keycode::C => 0xB,
                Keycode::V => 0xF,
                _ => 0x0,
            }
        };
        chip8.emulate_cycle(on_wait_key_event);

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'main,
                Event::KeyDown { keycode, .. } => {
                    if let Some(keycode) = keycode {
                        match keycode {
                            Keycode::Num1 => chip8.keyboard[0] = 1,
                            Keycode::Num2 => chip8.keyboard[1] = 1,
                            Keycode::Num3 => chip8.keyboard[2] = 1,
                            Keycode::Num4 => chip8.keyboard[3] = 1,
                            Keycode::Q => chip8.keyboard[4] = 1,
                            Keycode::W => chip8.keyboard[5] = 1,
                            Keycode::E => chip8.keyboard[6] = 1,
                            Keycode::R => chip8.keyboard[7] = 1,
                            Keycode::A => chip8.keyboard[8] = 1,
                            Keycode::S => chip8.keyboard[9] = 1,
                            Keycode::D => chip8.keyboard[10] = 1,
                            Keycode::F => chip8.keyboard[11] = 1,
                            Keycode::Z => chip8.keyboard[12] = 1,
                            Keycode::X => chip8.keyboard[13] = 1,
                            Keycode::C => chip8.keyboard[14] = 1,
                            Keycode::V => chip8.keyboard[15] = 1,
                            _ => {}
                        }
                    }
                }
                Event::KeyUp { keycode, .. } => {
                    if let Some(keycode) = keycode {
                        match keycode {
                            Keycode::Num1 => chip8.keyboard[0] = 0,
                            Keycode::Num2 => chip8.keyboard[1] = 0,
                            Keycode::Num3 => chip8.keyboard[2] = 0,
                            Keycode::Num4 => chip8.keyboard[3] = 0,
                            Keycode::Q => chip8.keyboard[4] = 0,
                            Keycode::W => chip8.keyboard[5] = 0,
                            Keycode::E => chip8.keyboard[6] = 0,
                            Keycode::R => chip8.keyboard[7] = 0,
                            Keycode::A => chip8.keyboard[8] = 0,
                            Keycode::S => chip8.keyboard[9] = 0,
                            Keycode::D => chip8.keyboard[10] = 0,
                            Keycode::F => chip8.keyboard[11] = 0,
                            Keycode::Z => chip8.keyboard[12] = 0,
                            Keycode::X => chip8.keyboard[13] = 0,
                            Keycode::C => chip8.keyboard[14] = 0,
                            Keycode::V => chip8.keyboard[15] = 0,
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }

        let rects = chip8
            .graphics
            .iter()
            .enumerate()
            .filter(|(_, pixel)| **pixel == 1)
            .map(|(idx, _)| {
                let row = (idx / 64usize) * 10;
                let col = (idx % 64usize) * 10;
                Rect::new(col as i32, row as i32, 10, 10)
            })
            .collect::<Vec<Rect>>();

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.set_draw_color(Color::RGB(255, 255, 255));
        canvas.fill_rects(&rects)?;
        canvas.present();

        // 500hz
        thread::sleep(Duration::from_millis(2));
        frame += 1;

        // 60hz
        if frame % 8 == 0 {
            chip8.update_timers(|| audio_device.resume());
        }
        if frame % 32 == 0 && audio_device.status() == AudioStatus::Playing {
            audio_device.pause()
        }
    }

    Ok(())
}

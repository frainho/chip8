use std::{error::Error, thread, time::Duration};

use audio::SdlAudio;
use chip8_core::Chip8;
use graphics::SdlGraphics;
use sdl2::{event::Event, keyboard::Keycode};
mod audio;
mod graphics;

fn main() -> Result<(), Box<dyn Error>> {
    let sdl_context = sdl2::init()?;

    let sdl_audio = SdlAudio::new(&sdl_context)?;
    let mut sdl_graphics = SdlGraphics::new(&sdl_context)?;

    let mut event_pump = sdl_context.event_pump()?;

    let mut chip8 = Chip8::new(Box::new(|| 1), sdl_audio);
    chip8.initialize();
    chip8.load_program("Space Invaders.ch8")?;

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

        sdl_graphics.draw(&chip8.graphics)?;

        // 500hz
        thread::sleep(Duration::from_millis(2));
    }

    Ok(())
}

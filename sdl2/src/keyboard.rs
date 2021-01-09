use std::error::Error;

use chip8_core::KeyboardEvents;
use sdl2::{event::Event, keyboard::Keycode, Sdl};

pub struct SdlKeyboard<'a> {
    sdl_context: &'a Sdl,
}

impl<'a> SdlKeyboard<'a> {
    pub fn new(sdl_context: &'a Sdl) -> Result<SdlKeyboard<'a>, Box<dyn Error>> {
        Ok(SdlKeyboard { sdl_context })
    }

    pub fn should_exit(&self) -> bool {
        match self.sdl_context.event_pump() {
            Ok(mut event_pump) => {
                let mut should_exit = false;
                for event in event_pump.poll_iter() {
                    match event {
                        Event::Quit { .. }
                        | Event::KeyDown {
                            keycode: Some(Keycode::Escape),
                            ..
                        } => should_exit = true,
                        _ => (),
                    }
                }
                should_exit
            }
            Err(_) => false,
        }
    }

    pub fn get_keyboard_handler(&self) -> KeyboardHandler {
        KeyboardHandler {
            sdl_context: self.sdl_context,
        }
    }
}

pub struct KeyboardHandler<'a> {
    sdl_context: &'a Sdl,
}

impl<'a> KeyboardEvents for KeyboardHandler<'a> {
    fn wait_on_key_event(&self) -> u8 {
        match self.sdl_context.event_pump() {
            Ok(mut event_pump) => {
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
            }
            Err(_) => 0x0,
        }
    }

    fn handle_keyboard_events(&self, keyboard: &mut [u8; 16]) -> Result<(), Box<dyn Error>> {
        let mut event_pump = self.sdl_context.event_pump()?;
        for event in event_pump.poll_iter() {
            match event {
                Event::KeyDown { keycode, .. } => {
                    if let Some(keycode) = keycode {
                        match keycode {
                            Keycode::Num1 => keyboard[0] = 1,
                            Keycode::Num2 => keyboard[1] = 1,
                            Keycode::Num3 => keyboard[2] = 1,
                            Keycode::Num4 => keyboard[3] = 1,
                            Keycode::Q => keyboard[4] = 1,
                            Keycode::W => keyboard[5] = 1,
                            Keycode::E => keyboard[6] = 1,
                            Keycode::R => keyboard[7] = 1,
                            Keycode::A => keyboard[8] = 1,
                            Keycode::S => keyboard[9] = 1,
                            Keycode::D => keyboard[10] = 1,
                            Keycode::F => keyboard[11] = 1,
                            Keycode::Z => keyboard[12] = 1,
                            Keycode::X => keyboard[13] = 1,
                            Keycode::C => keyboard[14] = 1,
                            Keycode::V => keyboard[15] = 1,
                            _ => {}
                        }
                    }
                }
                Event::KeyUp { keycode, .. } => {
                    if let Some(keycode) = keycode {
                        match keycode {
                            Keycode::Num1 => keyboard[0] = 0,
                            Keycode::Num2 => keyboard[1] = 0,
                            Keycode::Num3 => keyboard[2] = 0,
                            Keycode::Num4 => keyboard[3] = 0,
                            Keycode::Q => keyboard[4] = 0,
                            Keycode::W => keyboard[5] = 0,
                            Keycode::E => keyboard[6] = 0,
                            Keycode::R => keyboard[7] = 0,
                            Keycode::A => keyboard[8] = 0,
                            Keycode::S => keyboard[9] = 0,
                            Keycode::D => keyboard[10] = 0,
                            Keycode::F => keyboard[11] = 0,
                            Keycode::Z => keyboard[12] = 0,
                            Keycode::X => keyboard[13] = 0,
                            Keycode::C => keyboard[14] = 0,
                            Keycode::V => keyboard[15] = 0,
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
}

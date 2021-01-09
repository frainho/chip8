use std::{error::Error, thread, time::Duration};

use audio::SdlAudio;
use chip8_core::Chip8;
use graphics::SdlGraphics;
use keyboard::SdlKeyboard;
mod audio;
mod graphics;
mod keyboard;

fn main() -> Result<(), Box<dyn Error>> {
    let sdl_context = sdl2::init()?;

    let sdl_audio = SdlAudio::new(&sdl_context)?;
    let mut sdl_graphics = SdlGraphics::new(&sdl_context)?;
    let sdl_keyboard = SdlKeyboard::new(&sdl_context)?;

    let mut chip8 = Chip8::new(
        Box::new(|| 1),
        Box::new(sdl_audio),
        Box::new(sdl_keyboard.get_keyboard_handler()),
    );
    chip8.initialize();
    chip8.load_program("Space Invaders.ch8")?;

    'main: loop {
        if sdl_keyboard.should_exit() == true {
            break 'main;
        }
        chip8.emulate_cycle()?;

        sdl_graphics.draw(&chip8.graphics)?;

        // 500hz
        thread::sleep(Duration::from_millis(2));
    }

    Ok(())
}

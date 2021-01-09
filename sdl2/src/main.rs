use rand::Rng;

use std::{error::Error, thread, time::Duration};

mod audio;
mod graphics;
mod keyboard;
mod rom_loader;

use audio::SdlAudio;
use chip8_core::Chip8;
use graphics::SdlGraphics;
use keyboard::SdlKeyboard;
use rom_loader::RomLoader;

fn main() -> Result<(), Box<dyn Error>> {
    let sdl_context = sdl2::init()?;

    let sdl_audio = SdlAudio::new(&sdl_context)?;
    let mut sdl_graphics = SdlGraphics::new(&sdl_context)?;
    let sdl_keyboard = SdlKeyboard::new(&sdl_context)?;

    let mut chip8 = Chip8::new(
        Box::new(|| rand::thread_rng().gen()),
        Box::new(sdl_audio),
        Box::new(sdl_keyboard.get_keyboard_handler()),
    );
    chip8.initialize();

    let rom_data = RomLoader::load_rom("Space Invaders.ch8")?;
    chip8.load_program(rom_data)?;

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

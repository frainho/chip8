use number_generator::RandomNumberGenerator;
use std::{error::Error, thread, time::Duration};

mod audio;
mod graphics;
mod keyboard;
mod number_generator;
mod rom_loader;

use audio::SdlAudio;
use chip8_core::{Chip8, State};
use graphics::SdlGraphics;
use keyboard::SdlKeyboard;
use rom_loader::RomLoader;

fn main() -> Result<(), Box<dyn Error>> {
    let sdl_context = sdl2::init()?;

    let sdl_audio = SdlAudio::new(&sdl_context)?;
    let sdl_graphics = SdlGraphics::new(&sdl_context)?;
    let sdl_keyboard = SdlKeyboard::new(&sdl_context)?;

    let mut chip8 = Chip8::new(
        Box::new(RandomNumberGenerator),
        Box::new(sdl_audio),
        Box::new(sdl_keyboard),
        Box::new(sdl_graphics),
    );
    let rom_data = RomLoader::load_rom("Space Invaders.ch8")?;
    chip8.load_program(rom_data)?;

    'main: loop {
        if let State::Exit = chip8.emulate_cycle() {
            break 'main;
        };

        // 500hz
        thread::sleep(Duration::from_millis(2));
    }

    Ok(())
}

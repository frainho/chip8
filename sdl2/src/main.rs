use std::{error::Error, path::PathBuf, thread, time::Duration};
use structopt::StructOpt;

mod audio;
mod graphics;
mod keyboard;
mod number_generator;
mod rom_loader;

use audio::SdlAudio;
use chip8_core::{Chip8, State};
use graphics::SdlGraphics;
use keyboard::SdlKeyboard;
use number_generator::RandomNumberGenerator;
use rom_loader::RomLoader;

#[derive(StructOpt, Debug)]
#[structopt(name = "chip8-sdl")]
struct CliArgs {
    #[structopt(long = "rom", short = "r")]
    rom: PathBuf,
    #[structopt(long = "hertz", short = "h", default_value = "500")]
    hertz: u32,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli_args = CliArgs::from_args();
    let rom_data = RomLoader::load_rom(&cli_args.rom)?;
    let sleep_time = 1000 / cli_args.hertz;

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

    chip8.load_program(rom_data)?;

    'main: loop {
        if let State::Exit = chip8.emulate_cycle()? {
            break 'main;
        };

        thread::sleep(Duration::from_millis(sleep_time.into()));
    }

    Ok(())
}

mod chip8;
mod font_set;
mod test_utils;
mod types;

use std::error::Error;

use chip8::Chip8;

fn main() -> Result<(), Box<dyn Error>> {
    let mut chip8 = Chip8::new();

    chip8.initialize();

    chip8.load_program("IBM Logo.ch8")?;

    loop {
        chip8.emulate_cycle();
        break;
    }

    Ok(())
}

use crate::errors::Chip8Error;

pub trait Keyboard {
    fn update_state(&mut self, keyboard: &mut [u8; 16]) -> bool;
    fn wait_next_key_press(&mut self) -> u8;
}

pub trait NumberGenerator {
    fn generate(&self) -> Result<u8, Chip8Error>;
}

pub trait Audio {
    fn play(&self) -> Result<(), Chip8Error>;
    fn stop(&self) -> Result<(), Chip8Error>;
}

pub trait Graphics {
    fn draw(&mut self, graphics: &[u8]) -> Result<(), Chip8Error>;
}

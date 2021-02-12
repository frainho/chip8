use chip8_core::{Chip8Error, NumberGenerator};
use rand::Rng;

pub struct RandomNumberGenerator;

impl NumberGenerator for RandomNumberGenerator {
    fn generate(&self) -> Result<u8, Chip8Error> {
        Ok(rand::thread_rng().gen())
    }
}

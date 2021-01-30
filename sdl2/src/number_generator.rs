use chip8_core::NumberGenerator;
use rand::Rng;

pub struct RandomNumberGenerator;

impl NumberGenerator for RandomNumberGenerator {
    fn generate(&self) -> u8 {
        rand::thread_rng().gen()
    }
}

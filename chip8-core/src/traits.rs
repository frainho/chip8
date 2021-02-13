use crate::errors::Chip8Error;

/// Trait to hook up keyboard events to the interpreter
pub trait Keyboard {
    /// Updates the current state of the keyboard
    ///
    /// Returns true if the user triggered an exit event
    fn update_state(&mut self, keyboard: &mut [u8; 16]) -> bool;
    /// Add support for blocking and waiting for the next key press
    fn wait_next_key_press(&mut self) -> u8;
}

/// Trait to generate a random number
pub trait NumberGenerator {
    /// Call to generate valid u8 number
    fn generate(&self) -> Result<u8, Chip8Error>;
}

/// Trait to handle the audio device used
pub trait Audio {
    /// Start audio output
    fn play(&self) -> Result<(), Chip8Error>;
    /// Stop audio output
    fn stop(&self) -> Result<(), Chip8Error>;
}

/// Trait to handle graphics drawing on the screen
pub trait Graphics {
    /// Provides the current state of the graphics so it can be drawn on screen
    fn draw(&mut self, graphics: &[u8]) -> Result<(), Chip8Error>;
}

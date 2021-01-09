use std::error::Error;

use crate::types::Keyboard;

pub trait KeyboardEvents {
    fn wait_on_key_event(&self) -> u8;
    fn handle_keyboard_events(&self, keyboard: &mut Keyboard) -> Result<(), Box<dyn Error>>;
}

use crate::types;

pub trait Keyboard {
    fn update_state(&mut self, keyboard: &mut types::Keyboard) -> bool;
    fn wait_next_key_press(&mut self) -> u8;
}

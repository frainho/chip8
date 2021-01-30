use crate::types;

pub trait Keyboard {
    fn update_state(&self, keyboard: &mut types::Keyboard) -> bool;
    fn wait_next_key_press(&self) -> u8;
}

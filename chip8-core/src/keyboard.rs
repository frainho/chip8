pub trait Keyboard {
    fn update_state(&mut self, keyboard: &mut [u8; 16]) -> bool;
    fn wait_next_key_press(&mut self) -> u8;
}

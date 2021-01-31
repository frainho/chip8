pub trait Keyboard {
    fn update_state(&mut self, keyboard: &mut [u8; 16]) -> bool;
    fn wait_next_key_press(&mut self) -> u8;
}

pub trait NumberGenerator {
    fn generate(&self) -> u8;
}

pub trait Audio {
    fn play(&self);
    fn stop(&self);
}

pub trait Graphics {
    fn draw(&mut self, graphics: &[u8]);
}

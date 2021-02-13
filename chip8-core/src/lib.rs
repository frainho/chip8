#![warn(missing_docs)]

//! A chip8 interpreter built for fun
//!
//! This crate is contains the core functionality with the chip8's base opcodes
//!
//! This allows it to be used by different frontends as long as it compiles for that target
//!
//! It also tries to expose a few traits in order to allow that

mod errors;
mod traits;

use std::io::prelude::*;

pub use errors::Chip8Error;
pub use traits::{Audio, Graphics, Keyboard, NumberGenerator};

const FONT_SET: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

/// Basic enum to keep track of wether the user wants to quit
///
/// This is important because the chip8 will be the one
/// listening for keyboard events
pub enum State {
    /// No key was pressed to exit
    Continue,
    /// Should exit immediately
    Exit,
}

/// This struct is the main part of the Chip8 implementation
///
/// It contains all the specs of the interpreter
/// and stores the frontends implementations of the required traits
pub struct Chip8 {
    delay_timer: u8,
    graphics: [u8; 2048],
    index_register: u16,
    keyboard: [u8; 16],
    memory: [u8; 4096],
    opcode: u16,
    program_counter: u16,
    sound_timer: u8,
    stack: [u16; 16],
    stack_pointer: u16,
    v_registers: [u8; 16],
    random_number_generator: Box<dyn NumberGenerator>,
    audio_device: Box<dyn Audio>,
    keyboard_device: Box<dyn Keyboard>,
    graphics_device: Box<dyn Graphics>,
}

impl Chip8 {
    /// Instantiates the Chip8 with the provided implementations
    pub fn new(
        random_number_generator: Box<dyn NumberGenerator>,
        audio_device: Box<dyn Audio>,
        keyboard_device: Box<dyn Keyboard>,
        graphics_device: Box<dyn Graphics>,
    ) -> Chip8 {
        let mut chip8 = Chip8 {
            delay_timer: 0,
            graphics: [0; 2048],
            index_register: 0,
            keyboard: [0; 16],
            memory: [0; 4096],
            opcode: 0,
            program_counter: 0x200,
            sound_timer: 0,
            stack: [0; 16],
            stack_pointer: 0,
            v_registers: [0; 16],
            random_number_generator,
            audio_device,
            keyboard_device,
            graphics_device,
        };
        chip8.load_font_set();
        chip8
    }
    /// Loads a rom onto memory
    pub fn load_program(&mut self, rom_data: Vec<u8>) -> Result<(), Chip8Error> {
        let mut program_memory = &mut self.memory[self.program_counter as usize..];
        program_memory.write_all(&rom_data)?;

        Ok(())
    }

    /// Emulates a cycle of the interpreter
    ///
    /// It retrieves the next opcode to execute, it draws the next frame, updates the timers and listens to keyboard events
    ///
    /// In case the user wants to exit, either by clicking the `X` on the window or pressing the escape key
    /// this state is returned to the caller so it can interrupt the loop
    pub fn emulate_cycle(&mut self) -> Result<State, Chip8Error> {
        self.fetch_opcode();
        self.interpret_opcode()?;
        self.graphics_device.draw(&self.graphics)?;
        self.update_timers()?;

        let state = match self.keyboard_device.update_state(&mut self.keyboard) {
            true => State::Exit,
            false => State::Continue,
        };

        Ok(state)
    }

    fn interpret_opcode(&mut self) -> Result<(), Chip8Error> {
        let leading_opcode_number = ((self.opcode & 0xF000) >> 12) as usize;
        let vx_index = ((self.opcode & 0x0F00) >> 8) as usize;
        let vy_index = ((self.opcode & 0x00F0) >> 4) as usize;
        let nnn_address = self.opcode & 0x0FFF;
        let nn_address = self.opcode & 0x00FF;
        let n_address = self.opcode & 0x000F;

        match self.opcode {
            0x00E0 => self.clear_display(),
            0x00EE => self.return_from_routine(),
            0x1000..=0x1FFF => self.jump_to_address(nnn_address),
            0x2000..=0x2FFF => self.jump_to_routine(nnn_address),
            0x3000..=0x3FFF => self.skip_instruction_if_vx_equals_nn(vx_index, nn_address),
            0x4000..=0x4FFF => self.skip_instruction_if_vx_not_equals_nn(vx_index, nn_address),
            0x5000..=0x5FFF => self.skip_instruction_if_vx_equals_vy(vx_index, vy_index),
            0x6000..=0x6FFF => self.set_vx_to_nn(vx_index, nn_address),
            0x7000..=0x7FFF => self.add_nn_to_vx(vx_index, nn_address),
            0x8000..=0x8FFF => match n_address {
                0x0000 => self.sets_vx_to_vy(vx_index, vy_index),
                0x0001 => self.sets_vx_to_vx_bitwise_or_vy(vx_index, vy_index),
                0x0002 => self.sets_vx_to_vx_bitwise_and_vy(vx_index, vy_index),
                0x0003 => self.sets_vx_to_vx_bitwise_xor_vy(vx_index, vy_index),
                0x0004 => self.adds_vy_to_vx_setting_vf_on_borrow(vx_index, vy_index),
                0x0005 => self.subtracts_vy_from_vx_setting_vf_on_borrow(vx_index, vy_index),
                0x0006 => self.store_lsb_of_vx_in_vf_shifting_vx_by_1(vx_index),
                0x0007 => self.set_vx_to_vy_minus_vx_setting_vf_on_borrow(vx_index, vy_index),
                0x000E => self.store_msb_of_vx_in_vf_shifting_vx_by_1(vx_index),
                _ => return Err(Chip8Error::InvalidOpcode(self.opcode)),
            },
            0x9000..=0x9FFF => self.skip_instruction_if_vx_not_equals_vy(vx_index, vy_index),
            0xA000..=0xAFFF => self.set_index_register_to_nnn(nnn_address),
            0xB000..=0xBFFF => self.jump_to_address_nnn_plus_v0(nnn_address),
            0xC000..=0xCFFF => self.set_vx_to_random_number_bitwise_and_nn(vx_index, nn_address)?,
            0xD000..=0xDFFF => self.set_graphics(vx_index, vy_index, n_address),
            0xE000..=0xEFFF => match nn_address {
                0x009E => self.skips_instruction_if_vx_key_is_pressed(vx_index),
                0x00A1 => self.skips_instruction_if_vx_key_is_not_pressed(vx_index),
                _ => return Err(Chip8Error::InvalidOpcode(self.opcode)),
            },
            0xF000..=0xFFFF => match nn_address {
                0x0007 => self.sets_vx_to_delay_timer(vx_index),
                0x000A => self.sets_vx_to_key_press(vx_index),
                0x0015 => self.sets_delay_timer_to_vx(vx_index),
                0x0018 => self.sets_sound_timer_to_vx(vx_index),
                0x001E => self.adds_vx_to_i(vx_index),
                0x0029 => self.sets_i_to_vx(vx_index),
                0x0033 => self.store_bcd_of_vx_from_i(vx_index),
                0x0055 => self.stores_v0_to_vx_in_memory_from_i(vx_index),
                0x0065 => self.writes_v0_to_vx_from_memory_i(vx_index),
                _ => return Err(Chip8Error::InvalidOpcode(self.opcode)),
            },
            _ => return Err(Chip8Error::InvalidOpcode(self.opcode)),
        };

        let jumping_operations = [0x1usize, 0x2, 0xB];
        if !jumping_operations.contains(&leading_opcode_number) {
            self.program_counter += 2;
        }

        Ok(())
    }

    fn clear_display(&mut self) {
        for i in self.graphics.iter_mut() {
            *i = 0;
        }
    }

    fn return_from_routine(&mut self) {
        self.stack_pointer -= 1;
        self.program_counter = self.stack[self.stack_pointer as usize];
    }

    fn jump_to_address(&mut self, nnn_address: u16) {
        self.program_counter = nnn_address
    }

    fn jump_to_routine(&mut self, nnn_address: u16) {
        self.stack[self.stack_pointer as usize] = self.program_counter;
        self.stack_pointer += 1;
        self.program_counter = nnn_address;
    }

    fn skip_instruction_if_vx_equals_nn(&mut self, vx_index: usize, nn_address: u16) {
        let v_register_value = self.v_registers[vx_index];
        let value = nn_address as u8;

        if v_register_value == value {
            self.program_counter += 2;
        }
    }

    fn skip_instruction_if_vx_not_equals_nn(&mut self, vx_index: usize, nn_address: u16) {
        let v_register_value = self.v_registers[vx_index];
        let value = nn_address as u8;

        if v_register_value != value {
            self.program_counter += 2;
        }
    }

    fn skip_instruction_if_vx_equals_vy(&mut self, vx_index: usize, vy_index: usize) {
        let x_register_value = self.v_registers[vx_index];
        let y_register_value = self.v_registers[vy_index];

        if x_register_value == y_register_value {
            self.program_counter += 2;
        }
    }

    fn set_vx_to_nn(&mut self, vx_index: usize, nn_address: u16) {
        let new_v_register_value = nn_address as u8;
        self.v_registers[vx_index] = new_v_register_value;
    }

    fn add_nn_to_vx(&mut self, vx_index: usize, nn_address: u16) {
        let value_to_add = nn_address as u8;

        let (sum, _) = self.v_registers[vx_index].overflowing_add(value_to_add);
        self.v_registers[vx_index] = sum;
    }

    fn skip_instruction_if_vx_not_equals_vy(&mut self, vx_index: usize, vy_index: usize) {
        let vy = self.v_registers[vy_index];
        let vx = self.v_registers[vx_index];

        if vx != vy {
            self.program_counter += 2;
        }
    }

    fn set_index_register_to_nnn(&mut self, nnn_address: u16) {
        self.index_register = nnn_address;
    }

    fn jump_to_address_nnn_plus_v0(&mut self, nnn_address: u16) {
        let value_to_add = nnn_address;
        let v0_value = self.v_registers[0] as u16;
        self.program_counter += value_to_add + v0_value;
    }

    fn set_vx_to_random_number_bitwise_and_nn(
        &mut self,
        vx_index: usize,
        nn_address: u16,
    ) -> Result<(), Chip8Error> {
        let opcode_value = nn_address as u8;
        let random_number = self.random_number_generator.generate()?;
        self.v_registers[vx_index] = random_number & opcode_value;
        Ok(())
    }

    fn set_graphics(&mut self, vx_index: usize, vy_index: usize, n_address: u16) {
        let vx = self.v_registers[vx_index] as usize;
        let vy = self.v_registers[vy_index] as usize;

        let bytes_to_draw =
            &self.memory[self.index_register as usize..(self.index_register + n_address) as usize];

        self.v_registers[15usize] = 0;
        for (row, byte) in bytes_to_draw.iter().enumerate() {
            for col in 0..8 {
                if byte & 0x80 >> col > 0 {
                    let col = (vx + col) % 64;
                    let row = (vy + row) % 32;
                    let index = col + (row * 64);

                    self.v_registers[0xF] = if self.graphics[index] == 1 { 1 } else { 0 };

                    self.graphics[index] ^= 1;
                }
            }
        }
    }

    fn skips_instruction_if_vx_key_is_pressed(&mut self, vx_index: usize) {
        let vx_value = self.v_registers[vx_index];
        if self.keyboard[vx_value as usize] == 1 {
            self.program_counter += 2;
        }
    }

    fn skips_instruction_if_vx_key_is_not_pressed(&mut self, vx_index: usize) {
        let vx_value = self.v_registers[vx_index];
        if self.keyboard[vx_value as usize] == 0 {
            self.program_counter += 2;
        }
    }

    fn sets_vx_to_delay_timer(&mut self, vx_index: usize) {
        self.v_registers[vx_index] = self.delay_timer
    }

    fn sets_vx_to_key_press(&mut self, vx_index: usize) {
        self.v_registers[vx_index] = self.keyboard_device.wait_next_key_press();
    }

    fn sets_delay_timer_to_vx(&mut self, vx_index: usize) {
        self.delay_timer = self.v_registers[vx_index];
    }

    fn sets_sound_timer_to_vx(&mut self, vx_index: usize) {
        self.sound_timer = self.v_registers[vx_index];
    }

    fn adds_vx_to_i(&mut self, vx_index: usize) {
        self.index_register += self.v_registers[vx_index] as u16;
    }

    fn sets_i_to_vx(&mut self, vx_index: usize) {
        self.index_register = self.v_registers[vx_index] as u16;
    }

    fn store_bcd_of_vx_from_i(&mut self, vx_index: usize) {
        let vx_value = self.v_registers[vx_index];

        self.memory[self.index_register as usize] = vx_value / 100;
        self.memory[self.index_register as usize + 1] = (vx_value / 10) % 10;
        self.memory[self.index_register as usize + 2] = vx_value % 10;
    }

    fn stores_v0_to_vx_in_memory_from_i(&mut self, vx_index: usize) {
        let v_registers_to_copy = &self.v_registers[0..=vx_index];

        for (index, v_register_value) in v_registers_to_copy.iter().enumerate() {
            self.memory[self.index_register as usize + index] = *v_register_value;
        }
    }

    fn writes_v0_to_vx_from_memory_i(&mut self, vx_index: usize) {
        let v_registers_to_write = &mut self.v_registers[0..=vx_index];

        for (index, v_register_to_write) in v_registers_to_write.iter_mut().enumerate() {
            *v_register_to_write = self.memory[self.index_register as usize + index];
        }
    }

    fn sets_vx_to_vy(&mut self, vx_index: usize, vy_index: usize) {
        self.v_registers[vx_index] = self.v_registers[vy_index]
    }

    fn sets_vx_to_vx_bitwise_or_vy(&mut self, vx_index: usize, vy_index: usize) {
        self.v_registers[vx_index] |= self.v_registers[vy_index]
    }

    fn sets_vx_to_vx_bitwise_and_vy(&mut self, vx_index: usize, vy_index: usize) {
        self.v_registers[vx_index] &= self.v_registers[vy_index]
    }

    fn sets_vx_to_vx_bitwise_xor_vy(&mut self, vx_index: usize, vy_index: usize) {
        self.v_registers[vx_index] ^= self.v_registers[vy_index]
    }

    fn adds_vy_to_vx_setting_vf_on_borrow(&mut self, vx_index: usize, vy_index: usize) {
        let vy = self.v_registers[vy_index];
        let vx = self.v_registers[vx_index];

        let (result, overflowed) = vx.overflowing_add(vy);

        if overflowed {
            self.v_registers[0xF] = 1;
        }

        self.v_registers[vx_index] = result;
    }

    fn subtracts_vy_from_vx_setting_vf_on_borrow(&mut self, vx_index: usize, vy_index: usize) {
        let vy = self.v_registers[vy_index];
        let vx = self.v_registers[vx_index];

        let (result, overflowed) = vx.overflowing_sub(vy);

        if overflowed {
            self.v_registers[0xF] = 1;
        }

        self.v_registers[vx_index] = result;
    }

    fn store_lsb_of_vx_in_vf_shifting_vx_by_1(&mut self, vx_index: usize) {
        let vx = self.v_registers[vx_index];
        self.v_registers[0xF] = vx & 1;
        self.v_registers[vx_index] >>= 1;
    }

    fn set_vx_to_vy_minus_vx_setting_vf_on_borrow(&mut self, vx_index: usize, vy_index: usize) {
        let vy = self.v_registers[vy_index];
        let vx = self.v_registers[vx_index];

        let (result, overflowed) = vx.overflowing_sub(vy);

        if overflowed {
            self.v_registers[15] = 1;
        } else {
            self.v_registers[15] = 0;
        }

        self.v_registers[vx_index] = result;
    }

    fn store_msb_of_vx_in_vf_shifting_vx_by_1(&mut self, vx_index: usize) {
        let vx_msb = self.v_registers[vx_index] >> 7;
        self.v_registers[15usize] = vx_msb;
        self.v_registers[vx_index] <<= 1;
    }

    fn load_font_set(&mut self) {
        for (i, _) in FONT_SET.iter().enumerate() {
            self.memory[i] = FONT_SET[i];
        }
    }

    fn fetch_opcode(&mut self) {
        self.opcode = (self.memory[self.program_counter as usize] as u16) << 8;
        self.opcode |= self.memory[self.program_counter as usize + 1] as u16;
    }

    fn update_timers(&mut self) -> Result<(), Chip8Error> {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            if self.sound_timer == 1 {
                self.audio_device.play()?;
            }
            self.sound_timer -= 1;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub fn set_initial_opcode_to(opcode: u16, memory: &mut [u8; 4096]) {
        memory[0x200] = ((opcode & 0xFF00) >> 8) as u8;
        memory[0x201] = (opcode & 0x00FF) as u8;
    }

    struct MockAudio;
    impl Audio for MockAudio {
        fn play(&self) -> Result<(), Chip8Error> {
            Ok(())
        }

        fn stop(&self) -> Result<(), Chip8Error> {
            Ok(())
        }
    }

    struct MockNumberGenerator;
    impl NumberGenerator for MockNumberGenerator {
        fn generate(&self) -> Result<u8, Chip8Error> {
            Ok(1)
        }
    }

    struct MockKeyboardDevice;
    impl Keyboard for MockKeyboardDevice {
        fn wait_next_key_press(&mut self) -> u8 {
            1
        }

        fn update_state(&mut self, _keyboard: &mut [u8; 16]) -> bool {
            true
        }
    }

    struct MockGraphicsDevice;
    impl Graphics for MockGraphicsDevice {
        fn draw(&mut self, _graphics: &[u8]) -> Result<(), Chip8Error> {
            Ok(())
        }
    }

    fn get_chip8_instance() -> Chip8 {
        Chip8::new(
            Box::new(MockNumberGenerator),
            Box::new(MockAudio),
            Box::new(MockKeyboardDevice),
            Box::new(MockGraphicsDevice),
        )
    }

    #[test]
    fn it_sets_the_correct_default_values() {
        let chip8 = get_chip8_instance();

        assert_eq!(chip8.opcode, 0);
        assert_eq!(chip8.program_counter, 0x200);
        assert_eq!(chip8.index_register, 0);
        assert_eq!(chip8.stack_pointer, 0);
        assert_eq!(chip8.graphics, [0; 2048]);
        assert_eq!(chip8.v_registers, [0; 16]);
        assert_eq!(chip8.stack, [0; 16]);
        assert_eq!(chip8.delay_timer, 0);
        assert_eq!(chip8.sound_timer, 0);
    }

    #[test]
    fn it_loads_the_font_set_on_initialization() {
        let chip8 = get_chip8_instance();

        assert_eq!(&chip8.memory[0..80], FONT_SET);
    }

    // #[test]
    // fn it_loads_the_program_to_memory() -> Result<(), std::io::Error> {
    //     let fake_data = b"fake_data";
    //     let _file = TestFile::create("test.ch8", fake_data)?;
    //     let mut chip8 = get_chip8_instance();

    //     chip8.load_program("test.ch8")?;

    //     assert_eq!(&chip8.memory[0x200..0x200 + fake_data.len()], fake_data);
    //     Ok(())
    // }

    #[test]
    fn it_fetches_correct_opcode_when_emulating_the_first_cycle() -> Result<(), Chip8Error> {
        let mut chip8 = get_chip8_instance();
        chip8.memory[0x200] = 0x10;
        chip8.memory[0x201] = 0x20;

        chip8.emulate_cycle()?;

        assert_eq!(chip8.opcode, 4128);
        Ok(())
    }

    #[test]
    fn it_correctly_counts_down_the_timers() -> Result<(), Chip8Error> {
        let mut chip8 = get_chip8_instance();
        set_initial_opcode_to(0x00E0, &mut chip8.memory);

        chip8.delay_timer = 1;
        chip8.sound_timer = 1;

        chip8.emulate_cycle()?;

        assert_eq!(chip8.delay_timer, 0);
        assert_eq!(chip8.sound_timer, 0);

        chip8.memory[0x202] = 0x00;
        chip8.memory[0x203] = 0xE0;

        chip8.emulate_cycle()?;

        assert_eq!(chip8.delay_timer, 0);
        assert_eq!(chip8.sound_timer, 0);

        Ok(())
    }

    #[test]
    fn it_clears_the_display() -> Result<(), Chip8Error> {
        let mut chip8 = get_chip8_instance();
        chip8.graphics[1] = 69;
        chip8.graphics[2] = 98;
        set_initial_opcode_to(0x00E0, &mut chip8.memory);

        chip8.emulate_cycle()?;

        assert_eq!(chip8.graphics, [0u8; 2048]);

        Ok(())
    }

    #[test]
    fn it_calls_the_subroutine_at_the_correct_address() -> Result<(), Chip8Error> {
        let mut chip8 = get_chip8_instance();
        set_initial_opcode_to(0x2010, &mut chip8.memory);

        chip8.emulate_cycle()?;

        assert_eq!(chip8.stack[0], 0x200);
        assert_eq!(chip8.stack_pointer, 1);
        assert_eq!(chip8.program_counter, 0x010);

        Ok(())
    }

    #[test]
    fn it_returns_from_a_subroutine() -> Result<(), Chip8Error> {
        let mut chip8 = get_chip8_instance();

        chip8.stack[0] = 0x123;
        chip8.stack_pointer = 1;

        set_initial_opcode_to(0x00EE, &mut chip8.memory);

        chip8.emulate_cycle()?;

        assert_eq!(chip8.stack_pointer, 0);
        assert_eq!(chip8.program_counter, 0x125);

        Ok(())
    }

    #[test]
    fn it_jumps_to_the_correct_address() -> Result<(), Chip8Error> {
        let mut chip8 = get_chip8_instance();

        set_initial_opcode_to(0x176C, &mut chip8.memory);

        chip8.emulate_cycle()?;

        assert_eq!(chip8.program_counter, 0x76C);

        Ok(())
    }

    #[test]
    fn it_skips_the_next_instruction_if_vx_equals_nn() -> Result<(), Chip8Error> {
        let mut chip8 = get_chip8_instance();
        chip8.v_registers[2] = 0x6C;
        chip8.program_counter = 0x200;

        set_initial_opcode_to(0x326C, &mut chip8.memory);

        chip8.emulate_cycle()?;

        assert_eq!(chip8.program_counter, 0x204);

        Ok(())
    }

    #[test]
    fn it_skips_the_next_instruction_if_vx_not_equals_nn() -> Result<(), Chip8Error> {
        let mut chip8 = get_chip8_instance();
        chip8.v_registers[2] = 0x6A;
        chip8.program_counter = 0x200;

        set_initial_opcode_to(0x426C, &mut chip8.memory);

        chip8.emulate_cycle()?;

        assert_eq!(chip8.program_counter, 0x204);

        Ok(())
    }

    #[test]
    fn it_skips_the_next_instruction_if_vx_equals_vy() -> Result<(), Chip8Error> {
        let mut chip8 = get_chip8_instance();
        chip8.v_registers[2] = 0x6A;
        chip8.v_registers[3] = 0x6A;
        chip8.program_counter = 0x200;

        set_initial_opcode_to(0x5230, &mut chip8.memory);

        chip8.emulate_cycle()?;

        assert_eq!(chip8.program_counter, 0x204);

        Ok(())
    }

    #[test]
    fn it_stores_the_least_significant_bit_of_vx_in_vf_and_shifts_vx_to_the_right_by_1(
    ) -> Result<(), Chip8Error> {
        let mut chip8 = get_chip8_instance();

        chip8.v_registers[6] = 0b00000011;

        set_initial_opcode_to(0x86A6, &mut chip8.memory);

        chip8.emulate_cycle()?;

        assert_eq!(chip8.v_registers[6], 0b00000001);
        assert_eq!(chip8.v_registers[15], 0b1);

        Ok(())
    }

    #[test]
    fn it_sets_vx_to_vy_minus_vx_vf_is_set_to_0_when_there_is_a_borrow() -> Result<(), Chip8Error> {
        let mut chip8 = get_chip8_instance();

        chip8.v_registers[4] = 0x20;
        chip8.v_registers[5] = 0x11;

        set_initial_opcode_to(0x8457, &mut chip8.memory);

        chip8.emulate_cycle()?;

        assert_eq!(chip8.v_registers[4], 0xF);
        assert_eq!(chip8.v_registers[15], 0);

        Ok(())
    }

    #[test]
    fn it_sets_vx_to_vy_minus_vx_vf_is_set_to_1_when_there_isnt_a_borrow() -> Result<(), Chip8Error>
    {
        let mut chip8 = get_chip8_instance();

        chip8.v_registers[4] = 0x11;
        chip8.v_registers[5] = 0x20;

        set_initial_opcode_to(0x8457, &mut chip8.memory);

        chip8.emulate_cycle()?;

        assert_eq!(chip8.v_registers[4], 0xF1);
        assert_eq!(chip8.v_registers[15], 1);

        Ok(())
    }

    #[test]
    fn it_sets_vf_to_the_value_of_vx_msb_shifts_vx_left_by_1() -> Result<(), Chip8Error> {
        let mut chip8 = get_chip8_instance();

        chip8.v_registers[1] = 0b10000000;

        set_initial_opcode_to(0x812E, &mut chip8.memory);

        chip8.emulate_cycle()?;

        assert_eq!(chip8.v_registers[15usize], 1);
        assert_eq!(chip8.v_registers[1], 0);

        Ok(())
    }

    #[test]
    fn it_skips_the_next_instruction_if_vx_not_equals_vy() -> Result<(), Chip8Error> {
        let mut chip8 = get_chip8_instance();

        chip8.v_registers[10] = 0x11;
        chip8.v_registers[11] = 0x20;

        set_initial_opcode_to(0x9AB0, &mut chip8.memory);

        chip8.emulate_cycle()?;

        assert_eq!(chip8.program_counter, 0x204);

        Ok(())
    }

    #[test]
    fn it_doesnt_skip_the_next_instruction_if_vx_equals_vy() -> Result<(), Chip8Error> {
        let mut chip8 = get_chip8_instance();

        chip8.v_registers[10] = 0x11;
        chip8.v_registers[11] = 0x11;

        set_initial_opcode_to(0x9AB0, &mut chip8.memory);

        chip8.emulate_cycle()?;

        assert_eq!(chip8.program_counter, 0x202);

        Ok(())
    }

    #[test]
    fn it_sets_the_index_register_value() -> Result<(), Chip8Error> {
        let mut chip8 = get_chip8_instance();

        set_initial_opcode_to(0xA111, &mut chip8.memory);

        chip8.emulate_cycle()?;

        assert_eq!(chip8.index_register, 0x111);

        Ok(())
    }

    #[test]
    fn it_sets_the_value_of_vx() -> Result<(), Chip8Error> {
        let mut chip8 = get_chip8_instance();
        chip8.v_registers[4] = 0xF;
        set_initial_opcode_to(0x6423, &mut chip8.memory);

        chip8.emulate_cycle()?;

        assert_eq!(chip8.v_registers[4], 0x23);

        Ok(())
    }

    #[test]
    fn it_adds_the_value_to_vx() -> Result<(), Chip8Error> {
        let mut chip8 = get_chip8_instance();
        chip8.v_registers[1] = 0x10;
        set_initial_opcode_to(0x7110, &mut chip8.memory);

        chip8.emulate_cycle()?;

        assert_eq!(chip8.v_registers[1], 0x20);

        Ok(())
    }

    #[test]
    fn it_sets_the_value_of_vx_to_vy() -> Result<(), Chip8Error> {
        let mut chip8 = get_chip8_instance();
        chip8.v_registers[1] = 0x10;
        chip8.v_registers[2] = 0x20;
        set_initial_opcode_to(0x8120, &mut chip8.memory);

        chip8.emulate_cycle()?;

        assert_eq!(chip8.v_registers[1], 0x20);

        Ok(())
    }

    #[test]
    fn it_sets_the_value_of_vx_to_vx_bitwise_or_vy() -> Result<(), Chip8Error> {
        let mut chip8 = get_chip8_instance();
        chip8.v_registers[6] = 0x10;
        chip8.v_registers[7] = 0x20;
        set_initial_opcode_to(0x8671, &mut chip8.memory);

        chip8.emulate_cycle()?;

        assert_eq!(chip8.v_registers[6], 0x30);

        Ok(())
    }

    #[test]
    fn it_sets_the_value_of_vx_to_vx_bitwise_and_vy() -> Result<(), Chip8Error> {
        let mut chip8 = get_chip8_instance();
        chip8.v_registers[8] = 0xFF;
        chip8.v_registers[9] = 0x10;
        set_initial_opcode_to(0x8892, &mut chip8.memory);

        chip8.emulate_cycle()?;

        assert_eq!(chip8.v_registers[8], 0x10);

        Ok(())
    }

    #[test]
    fn it_sets_the_value_of_vx_to_vx_bitwise_xor_vy() -> Result<(), Chip8Error> {
        let mut chip8 = get_chip8_instance();
        chip8.v_registers[7] = 0x72;
        chip8.v_registers[8] = 0x15;
        set_initial_opcode_to(0x8783, &mut chip8.memory);

        chip8.emulate_cycle()?;

        assert_eq!(chip8.v_registers[7], 0x67);

        Ok(())
    }

    #[test]
    fn it_adds_the_value_of_vy_to_vx_setting_vf_when_there_is_a_carry() -> Result<(), Chip8Error> {
        let mut chip8 = get_chip8_instance();
        chip8.v_registers[0] = 0xC8;
        chip8.v_registers[1] = 0x64;
        set_initial_opcode_to(0x8014, &mut chip8.memory);

        chip8.emulate_cycle()?;

        // Overflowing add of 200 + 100 = 44
        assert_eq!(chip8.v_registers[0], 0x2C);
        assert_eq!(chip8.v_registers[15usize], 1);

        Ok(())
    }

    #[test]
    fn it_subtracts_the_value_of_vy_of_vf_setting_vf_then_there_is_a_borrow(
    ) -> Result<(), Chip8Error> {
        let mut chip8 = get_chip8_instance();
        chip8.v_registers[0] = 0xD1;
        chip8.v_registers[1] = 0xD2;
        set_initial_opcode_to(0x8015, &mut chip8.memory);

        chip8.emulate_cycle()?;

        assert_eq!(chip8.v_registers[0], 0xFF);
        assert_eq!(chip8.v_registers[15usize], 1);

        Ok(())
    }

    #[test]
    fn it_jumps_to_the_address_nnn_plus_vx0() -> Result<(), Chip8Error> {
        let mut chip8 = get_chip8_instance();

        chip8.v_registers[0] = 0x1;
        set_initial_opcode_to(0xB100, &mut chip8.memory);

        chip8.emulate_cycle()?;

        assert_eq!(chip8.program_counter, 0x301);

        Ok(())
    }

    #[test]
    fn it_sets_vx_to_random_number_bitwise_and_nn() -> Result<(), Chip8Error> {
        let mut chip8 = get_chip8_instance();

        set_initial_opcode_to(0xC313, &mut chip8.memory);

        chip8.emulate_cycle()?;

        assert_eq!(chip8.v_registers[3], 0x1);

        Ok(())
    }

    //0xDXYN
    #[test]
    fn it_draws_the_correct_pixels() {
        // TBD
    }

    #[test]
    fn it_skips_instruction_if_key_press() -> Result<(), Chip8Error> {
        let mut chip8 = get_chip8_instance();
        chip8.v_registers[5] = 8;
        chip8.keyboard[8] = 1;
        set_initial_opcode_to(0xE59E, &mut chip8.memory);

        chip8.emulate_cycle()?;

        assert_eq!(chip8.program_counter, 0x204);

        Ok(())
    }

    #[test]
    fn it_skips_instruction_if_key_not_pressed() -> Result<(), Chip8Error> {
        let mut chip8 = get_chip8_instance();
        chip8.v_registers[3] = 6;
        chip8.keyboard[6] = 0;
        set_initial_opcode_to(0xE3A1, &mut chip8.memory);

        chip8.emulate_cycle()?;

        assert_eq!(chip8.program_counter, 0x204);

        Ok(())
    }

    #[test]
    fn it_waits_for_a_keypress_and_stores_it_in_vx() {
        // Todo
    }

    #[test]
    fn it_sets_vx_to_the_value_of_the_delay_timer() -> Result<(), Chip8Error> {
        let mut chip8 = get_chip8_instance();
        chip8.delay_timer = 40;
        set_initial_opcode_to(0xF307, &mut chip8.memory);

        chip8.emulate_cycle()?;

        assert_eq!(chip8.v_registers[3], 40);

        Ok(())
    }

    #[test]
    fn it_sets_the_delay_timer_to_vx() -> Result<(), Chip8Error> {
        let mut chip8 = get_chip8_instance();
        chip8.v_registers[5] = 100;
        set_initial_opcode_to(0xF515, &mut chip8.memory);

        chip8.emulate_cycle()?;

        assert_eq!(chip8.delay_timer, 99);

        Ok(())
    }

    #[test]
    fn it_sets_the_sound_timer_to_vx() -> Result<(), Chip8Error> {
        let mut chip8 = get_chip8_instance();
        chip8.v_registers[3] = 10;
        set_initial_opcode_to(0xF318, &mut chip8.memory);

        chip8.emulate_cycle()?;

        assert_eq!(chip8.sound_timer, 9);

        Ok(())
    }

    #[test]
    fn it_adds_vx_to_i() -> Result<(), Chip8Error> {
        let mut chip8 = get_chip8_instance();
        chip8.v_registers[8] = 0x10;
        chip8.index_register = 0x01;
        set_initial_opcode_to(0xF81E, &mut chip8.memory);

        chip8.emulate_cycle()?;

        assert_eq!(chip8.index_register, 0x11);

        Ok(())
    }

    #[test]
    fn it_sets_i_to_sprite_location_read_from_vx() -> Result<(), Chip8Error> {
        let mut chip8 = get_chip8_instance();
        chip8.v_registers[1] = 10;
        set_initial_opcode_to(0xF129, &mut chip8.memory);

        chip8.emulate_cycle()?;

        assert_eq!(chip8.index_register, 10);

        Ok(())
    }

    #[test]
    fn it_stores_bcd_of_vx_from_i() -> Result<(), Chip8Error> {
        let mut chip8 = get_chip8_instance();
        chip8.v_registers[9] = 123;
        chip8.index_register = 0x203;
        set_initial_opcode_to(0xF933, &mut chip8.memory);

        chip8.emulate_cycle()?;

        assert_eq!(chip8.memory[chip8.index_register as usize], 1);
        assert_eq!(chip8.memory[(chip8.index_register + 1) as usize], 2);
        assert_eq!(chip8.memory[(chip8.index_register + 2) as usize], 3);

        Ok(())
    }

    #[test]
    fn it_writes_from_v0_to_vx_starting_at_memory_address_i() -> Result<(), Chip8Error> {
        let mut chip8 = get_chip8_instance();
        let v_registers = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
        chip8.v_registers = v_registers;
        chip8.index_register = 0x204;
        set_initial_opcode_to(0xF355, &mut chip8.memory);

        chip8.emulate_cycle()?;

        assert_eq!(
            chip8.memory[chip8.index_register as usize..=chip8.index_register as usize + 3],
            [0, 1, 2, 3]
        );

        Ok(())
    }

    #[test]
    fn it_writes_to_v0_to_vx_starting_at_memory_address_i() -> Result<(), Chip8Error> {
        let mut chip8 = get_chip8_instance();
        chip8.index_register = 0x202;
        chip8.memory[chip8.index_register as usize] = 101;
        chip8.memory[chip8.index_register as usize + 1] = 102;
        chip8.memory[chip8.index_register as usize + 2] = 103;
        chip8.memory[chip8.index_register as usize + 3] = 104;
        set_initial_opcode_to(0xF365, &mut chip8.memory);

        chip8.emulate_cycle()?;

        assert_eq!(chip8.v_registers[0..=3], [101, 102, 103, 104]);

        Ok(())
    }
}

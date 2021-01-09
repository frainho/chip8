use std::{error::Error, io::prelude::*};

pub use audio::Audio;
use font_set::FONT_SET;
pub use keyboard::KeyboardEvents;
use types::{
    DelayTimer, Graphics, IndexRegister, Keyboard, Memory, Opcode, ProgramCounter, SoundTimer,
    Stack, StackPointer, VRegisters,
};

mod audio;
mod font_set;
mod keyboard;
mod types;

pub struct Chip8<'a> {
    delay_timer: DelayTimer,
    pub graphics: Graphics,
    index_register: IndexRegister,
    keyboard: Keyboard,
    memory: Memory,
    opcode: Opcode,
    program_counter: ProgramCounter,
    random_number_generator: Box<dyn Fn() -> u8>,
    sound_timer: SoundTimer,
    stack: Stack,
    stack_pointer: StackPointer,
    v_registers: VRegisters,
    audio: Box<dyn Audio + 'a>,
    keyboard_events: Box<dyn KeyboardEvents + 'a>,
}

impl<'a> Chip8<'a> {
    pub fn new<A, K>(
        random_number_generator: Box<dyn Fn() -> u8>,
        audio: Box<A>,
        keyboard_events: Box<K>,
    ) -> Chip8<'a>
    where
        A: 'a + Audio,
        K: 'a + KeyboardEvents,
    {
        Chip8 {
            delay_timer: 0,
            graphics: [0; 2048],
            index_register: 0,
            keyboard: [0; 16],
            memory: [0; 4096],
            opcode: 0,
            program_counter: 0x200,
            random_number_generator,
            sound_timer: 0,
            stack: [0; 16],
            stack_pointer: 0,
            v_registers: [0; 16],
            audio,
            keyboard_events,
        }
    }

    pub fn initialize(&mut self) {
        self.load_font_set();
    }

    pub fn load_program(&mut self, rom_data: Vec<u8>) -> Result<(), std::io::Error> {
        let mut program_memory = &mut self.memory[self.program_counter as usize..];
        program_memory.write_all(&rom_data)?;

        Ok(())
    }

    pub fn emulate_cycle(&mut self) -> Result<(), Box<dyn Error>> {
        self.fetch_opcode();
        match self.opcode {
            0x00E0 => {
                for i in 0..self.graphics.len() {
                    self.graphics[i] = 0;
                }
                self.program_counter += 2;
            }
            0x00EE => {
                self.stack_pointer -= 1;
                self.program_counter = self.stack[self.stack_pointer as usize];
                self.program_counter += 2;
            }
            0x1000..=0x1FFF => self.program_counter = self.opcode & 0x0FFF,
            0x2000..=0x2FFF => {
                self.stack[self.stack_pointer as usize] = self.program_counter;
                self.stack_pointer += 1;
                self.program_counter = self.opcode & 0x0FFF;
            }
            0x3000..=0x3FFF => {
                let v_index = ((self.opcode & 0x0F00) >> 8) as usize;
                let v_register_value = self.v_registers[v_index];
                let value = (self.opcode & 0x00FF) as u8;

                if v_register_value == value {
                    self.program_counter += 2;
                }
                self.program_counter += 2;
            }
            0x4000..=0x4FFF => {
                let v_index = ((self.opcode & 0x0F00) >> 8) as usize;
                let v_register_value = self.v_registers[v_index];
                let value = (self.opcode & 0x00FF) as u8;

                if v_register_value != value {
                    self.program_counter += 2;
                }

                self.program_counter += 2;
            }
            0x5000..=0x5FFF => {
                let x_index = ((self.opcode & 0x0F00) >> 8) as usize;
                let x_register_value = self.v_registers[x_index];
                let y_index = ((self.opcode & 0x00F0) >> 4) as usize;
                let y_register_value = self.v_registers[y_index];

                if x_register_value == y_register_value {
                    self.program_counter += 2;
                }
                self.program_counter += 2;
            }
            0x6000..=0x6FFF => {
                let v_register_index = ((self.opcode & 0x0F00) >> 8) as usize;
                let new_v_register_value = (self.opcode & 0x00FF) as u8;

                self.v_registers[v_register_index] = new_v_register_value;
                self.program_counter += 2;
            }
            0x7000..=0x7FFF => {
                let v_register_index = ((self.opcode & 0x0F00) >> 8) as usize;
                let value_to_add = (self.opcode & 0x00FF) as u8;

                let (sum, _) = self.v_registers[v_register_index].overflowing_add(value_to_add);
                self.v_registers[v_register_index] = sum;
                self.program_counter += 2;
            }
            0x8000..=0x8FFF => match self.opcode & 0x000F {
                0x0000 => {
                    let vx_index = ((self.opcode & 0x0F00) >> 8) as usize;
                    let vy_index = ((self.opcode & 0x00F0) >> 4) as usize;

                    self.v_registers[vx_index] = self.v_registers[vy_index];
                    self.program_counter += 2;
                }
                0x0001 => {
                    let vx_index = ((self.opcode & 0x0F00) >> 8) as usize;
                    let vy_index = ((self.opcode & 0x00F0) >> 4) as usize;

                    self.v_registers[vx_index] =
                        self.v_registers[vx_index] | self.v_registers[vy_index];
                    self.program_counter += 2;
                }
                0x0002 => {
                    let vx_index = ((self.opcode & 0x0F00) >> 8) as usize;
                    let vy_index = ((self.opcode & 0x00F0) >> 4) as usize;

                    self.v_registers[vx_index] =
                        self.v_registers[vx_index] & self.v_registers[vy_index];
                    self.program_counter += 2;
                }
                0x0003 => {
                    let vx_index = ((self.opcode & 0x0F00) >> 8) as usize;
                    let vy_index = ((self.opcode & 0x00F0) >> 4) as usize;

                    self.v_registers[vx_index] =
                        self.v_registers[vx_index] ^ self.v_registers[vy_index];
                    self.program_counter += 2;
                }
                0x0004 => {
                    let vy_index = ((self.opcode & 0x00F0) >> 4) as usize;
                    let vx_index = ((self.opcode & 0x0F00) >> 8) as usize;

                    let vy = self.v_registers[vy_index];
                    let vx = self.v_registers[vx_index];

                    let (result, overflowed) = vx.overflowing_add(vy);

                    if overflowed {
                        self.v_registers[15usize] = 1;
                    }

                    self.v_registers[vx_index] = result;
                    self.program_counter += 2;
                }
                0x0005 => {
                    let vy_index = ((self.opcode & 0x00F0) >> 4) as usize;
                    let vx_index = ((self.opcode & 0x0F00) >> 8) as usize;

                    let vy = self.v_registers[vy_index];
                    let vx = self.v_registers[vx_index];

                    let (result, overflowed) = vx.overflowing_sub(vy);

                    if overflowed {
                        self.v_registers[15usize] = 1;
                    }

                    self.v_registers[vx_index] = result;
                    self.program_counter += 2;
                }
                0x0006 => {
                    let vx_index = ((self.opcode & 0x0F00) >> 8) as usize;
                    let vx = self.v_registers[vx_index];
                    self.v_registers[15] = vx & 1;
                    self.v_registers[vx_index] >>= 1;
                    self.program_counter += 2;
                }
                0x0007 => {
                    let vy_index = ((self.opcode & 0x00F0) >> 4) as usize;
                    let vx_index = ((self.opcode & 0x0F00) >> 8) as usize;

                    let vy = self.v_registers[vy_index];
                    let vx = self.v_registers[vx_index];

                    let (result, overflowed) = vx.overflowing_sub(vy);

                    if overflowed {
                        self.v_registers[15] = 1;
                    } else {
                        self.v_registers[15] = 0;
                    }

                    self.v_registers[vx_index] = result;
                    self.program_counter += 2;
                }
                0x000E => {
                    let vx_index = ((self.opcode & 0x0F00) >> 8) as usize;
                    let vx_msb = self.v_registers[vx_index] >> 7;
                    self.v_registers[15usize] = vx_msb;
                    self.v_registers[vx_index] <<= 1;
                    self.program_counter += 2;
                }
                _ => panic!("Invalid opcode: {:x}", self.opcode),
            },
            0x9000..=0x9FFF => {
                let vy_index = ((self.opcode & 0x00F0) >> 4) as usize;
                let vx_index = ((self.opcode & 0x0F00) >> 8) as usize;
                let vy = self.v_registers[vy_index];
                let vx = self.v_registers[vx_index];

                if vx != vy {
                    self.program_counter += 2;
                }
                self.program_counter += 2;
            }
            0xA000..=0xAFFF => {
                self.index_register = self.opcode & 0x0FFF;
                self.program_counter += 2;
            }
            0xB000..=0xBFFF => {
                let value_to_add = self.opcode & 0x0FFF;
                let v0_value = self.v_registers[0] as u16;
                self.program_counter += value_to_add + v0_value;
            }
            0xC000..=0xCFFF => {
                let vx_index = ((self.opcode & 0x0F00) >> 8) as usize;
                let opcode_value = (self.opcode & 0x00FF) as u8;
                let random_number: u8 = (self.random_number_generator)();
                self.v_registers[vx_index] = random_number & opcode_value;
                self.program_counter += 2;
            }
            0xD000..=0xDFFF => {
                let vx_index = ((self.opcode & 0x0F00) >> 8) as usize;
                let vx = self.v_registers[vx_index] as usize;
                let vy_index = ((self.opcode & 0x00F0) >> 4) as usize;
                let vy = self.v_registers[vy_index] as usize;
                let n = self.opcode & 0x000F;
                let bytes_to_draw =
                    &self.memory[self.index_register as usize..(self.index_register + n) as usize];

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
                self.program_counter += 2;
            }
            0xE000..=0xEFFF => match self.opcode & 0x00FF {
                0x009E => {
                    let vx_index = ((self.opcode & 0x0F00) >> 8) as usize;
                    let vx_value = self.v_registers[vx_index];
                    if self.keyboard[vx_value as usize] == 1 {
                        self.program_counter += 2;
                    }
                    self.program_counter += 2;
                }
                0x00A1 => {
                    let vx_index = ((self.opcode & 0x0F00) >> 8) as usize;
                    let vx_value = self.v_registers[vx_index];
                    if self.keyboard[vx_value as usize] == 0 {
                        self.program_counter += 2;
                    }
                    self.program_counter += 2;
                }
                _ => panic!("Invalid opcode: {:x}", self.opcode),
            },
            0xF000..=0xFFFF => match self.opcode & 0x00FF {
                0x0007 => {
                    let vx_index = ((self.opcode & 0x0F00) >> 8) as usize;
                    self.v_registers[vx_index] = self.delay_timer;
                    self.program_counter += 2;
                }
                0x0015 => {
                    let vx_index = ((self.opcode & 0x0F00) >> 8) as usize;
                    self.delay_timer = self.v_registers[vx_index];
                    self.program_counter += 2;
                }
                0x0018 => {
                    let vx_index = ((self.opcode & 0x0F00) >> 8) as usize;
                    self.sound_timer = self.v_registers[vx_index];
                    self.program_counter += 2;
                }
                0x001E => {
                    let vx_index = ((self.opcode & 0x0F00) >> 8) as usize;
                    self.index_register += self.v_registers[vx_index] as u16;
                    self.program_counter += 2;
                }
                0x000A => {
                    let vx_index = ((self.opcode & 0x0F00) >> 8) as usize;
                    self.v_registers[vx_index] = self.keyboard_events.wait_on_key_event();
                    self.program_counter += 2;
                }
                0x0029 => {
                    let vx_index = ((self.opcode & 0x0F00) >> 8) as usize;
                    self.index_register = self.v_registers[vx_index] as u16;
                    self.program_counter += 2;
                }
                0x0033 => {
                    let vx_index = ((self.opcode & 0x0F00) >> 8) as usize;
                    let vx_value = self.v_registers[vx_index];

                    self.memory[self.index_register as usize] = vx_value / 100;
                    self.memory[self.index_register as usize + 1] = (vx_value / 10) % 10;
                    self.memory[self.index_register as usize + 2] = vx_value % 10;
                    self.program_counter += 2;
                }
                0x0055 => {
                    let vx_index = ((self.opcode & 0x0F00) >> 8) as usize;
                    let v_registers_to_copy = &self.v_registers[0..=vx_index];

                    for (index, v_register_value) in v_registers_to_copy.iter().enumerate() {
                        self.memory[self.index_register as usize + index] = *v_register_value;
                    }
                    self.program_counter += 2;
                }
                0x0065 => {
                    let vx_index = ((self.opcode & 0x0F00) >> 8) as usize;
                    let v_registers_to_write = &mut self.v_registers[0..=vx_index];

                    for (index, v_register_to_write) in v_registers_to_write.iter_mut().enumerate()
                    {
                        *v_register_to_write = self.memory[self.index_register as usize + index];
                    }

                    self.program_counter += 2;
                }
                _ => panic!("Invalid opcode: {:x}", self.opcode),
            },
            _ => panic!("Invalid opcode: {:x}", self.opcode),
        };

        self.keyboard_events
            .handle_keyboard_events(&mut self.keyboard)?;
        self.update_timers();
        Ok(())
    }

    fn load_font_set(&mut self) {
        for i in 0..80usize {
            self.memory[i] = FONT_SET[i];
        }
    }

    fn fetch_opcode(&mut self) {
        self.opcode = (self.memory[self.program_counter as usize] as u16) << 8;
        self.opcode = self.opcode | (self.memory[self.program_counter as usize + 1] as u16);
    }

    fn update_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            if self.sound_timer == 1 {
                self.audio.play();
            }
            self.sound_timer -= 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use super::*;

    pub fn set_initial_opcode_to(opcode: u16, memory: &mut Memory) {
        memory[0x200] = ((opcode & 0xFF00) >> 8) as u8;
        memory[0x201] = (opcode & 0x00FF) as u8;
    }

    pub fn test_random_number_generator() -> u8 {
        1
    }

    struct MockAudio;
    impl Audio for MockAudio {
        fn play(&self) -> () {
            ()
        }

        fn stop(&self) -> () {
            ()
        }
    }

    struct MockKeyboard;

    impl KeyboardEvents for MockKeyboard {
        fn wait_on_key_event(&self) -> u8 {
            1
        }

        fn handle_keyboard_events(&self, _keyboard: &mut Keyboard) -> Result<(), Box<dyn Error>> {
            Ok(())
        }
    }

    fn get_chip8_instance<'audio>() -> Chip8<'audio> {
        Chip8::new(
            Box::new(test_random_number_generator),
            Box::new(MockAudio),
            Box::new(MockKeyboard),
        )
    }

    #[test]
    fn it_sets_the_correct_default_values() {
        let chip8 = get_chip8_instance();

        assert_eq!(chip8.opcode, 0);
        assert_eq!(chip8.program_counter, 0x200);
        assert_eq!(chip8.index_register, 0);
        assert_eq!(chip8.stack_pointer, 0);
        assert_eq!(chip8.memory, [0; 4096]);
        assert_eq!(chip8.graphics, [0; 2048]);
        assert_eq!(chip8.v_registers, [0; 16]);
        assert_eq!(chip8.stack, [0; 16]);
        assert_eq!(chip8.delay_timer, 0);
        assert_eq!(chip8.sound_timer, 0);
    }

    #[test]
    fn it_loads_the_font_set_on_initialization() {
        let mut chip8 = get_chip8_instance();

        chip8.initialize();

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
    fn it_fetches_correct_opcode_when_emulating_the_first_cycle() {
        let mut chip8 = get_chip8_instance();
        chip8.memory[0x200] = 0x10;
        chip8.memory[0x201] = 0x20;

        chip8.emulate_cycle();

        assert_eq!(chip8.opcode, 4128);
    }

    #[test]
    fn it_correctly_counts_down_the_timers() {
        let mut chip8 = get_chip8_instance();
        set_initial_opcode_to(0x00E0, &mut chip8.memory);

        chip8.delay_timer = 1;
        chip8.sound_timer = 1;

        chip8.emulate_cycle();

        assert_eq!(chip8.delay_timer, 0);
        assert_eq!(chip8.sound_timer, 0);

        chip8.memory[0x202] = 0x00;
        chip8.memory[0x203] = 0xE0;

        chip8.emulate_cycle();

        assert_eq!(chip8.delay_timer, 0);
        assert_eq!(chip8.sound_timer, 0);
    }

    #[test]
    fn it_clears_the_display() {
        let mut chip8 = get_chip8_instance();
        chip8.graphics[1] = 69;
        chip8.graphics[2] = 98;
        set_initial_opcode_to(0x00E0, &mut chip8.memory);

        chip8.emulate_cycle();

        assert_eq!(chip8.graphics, [0u8; 2048]);
    }

    #[test]
    fn it_calls_the_subroutine_at_the_correct_address() {
        let mut chip8 = get_chip8_instance();
        set_initial_opcode_to(0x2010, &mut chip8.memory);

        chip8.emulate_cycle();

        assert_eq!(chip8.stack[0], 0x200);
        assert_eq!(chip8.stack_pointer, 1);
        assert_eq!(chip8.program_counter, 0x010);
    }

    #[test]
    fn it_returns_from_a_subroutine() {
        let mut chip8 = get_chip8_instance();

        chip8.stack[0] = 0x123;
        chip8.stack_pointer = 1;

        set_initial_opcode_to(0x00EE, &mut chip8.memory);

        chip8.emulate_cycle();

        assert_eq!(chip8.stack_pointer, 0);
        assert_eq!(chip8.program_counter, 0x125);
    }

    #[test]
    fn it_jumps_to_the_correct_address() {
        let mut chip8 = get_chip8_instance();

        set_initial_opcode_to(0x176C, &mut chip8.memory);

        chip8.emulate_cycle();

        assert_eq!(chip8.program_counter, 0x76C);
    }

    #[test]
    fn it_skips_the_next_instruction_if_vx_equals_nn() {
        let mut chip8 = get_chip8_instance();
        chip8.v_registers[2] = 0x6C;
        chip8.program_counter = 0x200;

        set_initial_opcode_to(0x326C, &mut chip8.memory);

        chip8.emulate_cycle();

        assert_eq!(chip8.program_counter, 0x204);
    }

    #[test]
    fn it_skips_the_next_instruction_if_vx_not_equals_nn() {
        let mut chip8 = get_chip8_instance();
        chip8.v_registers[2] = 0x6A;
        chip8.program_counter = 0x200;

        set_initial_opcode_to(0x426C, &mut chip8.memory);

        chip8.emulate_cycle();

        assert_eq!(chip8.program_counter, 0x204);
    }

    #[test]
    fn it_skips_the_next_instruction_if_vx_equals_vy() {
        let mut chip8 = get_chip8_instance();
        chip8.v_registers[2] = 0x6A;
        chip8.v_registers[3] = 0x6A;
        chip8.program_counter = 0x200;

        set_initial_opcode_to(0x5230, &mut chip8.memory);

        chip8.emulate_cycle();

        assert_eq!(chip8.program_counter, 0x204);
    }

    #[test]
    fn it_stores_the_least_significant_bit_of_vx_in_vf_and_shifts_vx_to_the_right_by_1() {
        let mut chip8 = get_chip8_instance();

        chip8.v_registers[6] = 0b00000011;

        set_initial_opcode_to(0x86A6, &mut chip8.memory);

        chip8.emulate_cycle();

        assert_eq!(chip8.v_registers[6], 0b00000001);
        assert_eq!(chip8.v_registers[15], 0b1);
    }

    #[test]
    fn it_sets_vx_to_vy_minus_vx_vf_is_set_to_0_when_there_is_a_borrow() {
        let mut chip8 = get_chip8_instance();

        chip8.v_registers[4] = 0x20;
        chip8.v_registers[5] = 0x11;

        set_initial_opcode_to(0x8457, &mut chip8.memory);

        chip8.emulate_cycle();

        assert_eq!(chip8.v_registers[4], 0xF);
        assert_eq!(chip8.v_registers[15], 0);
    }

    #[test]
    fn it_sets_vx_to_vy_minus_vx_vf_is_set_to_1_when_there_isnt_a_borrow() {
        let mut chip8 = get_chip8_instance();

        chip8.v_registers[4] = 0x11;
        chip8.v_registers[5] = 0x20;

        set_initial_opcode_to(0x8457, &mut chip8.memory);

        chip8.emulate_cycle();

        assert_eq!(chip8.v_registers[4], 0xF1);
        assert_eq!(chip8.v_registers[15], 1);
    }

    #[test]
    fn it_sets_vf_to_the_value_of_vx_msb_shifts_vx_left_by_1() {
        let mut chip8 = get_chip8_instance();

        chip8.v_registers[1] = 0b10000000;

        set_initial_opcode_to(0x812E, &mut chip8.memory);

        chip8.emulate_cycle();

        assert_eq!(chip8.v_registers[15usize], 1);
        assert_eq!(chip8.v_registers[1], 0);
    }

    #[test]
    fn it_skips_the_next_instruction_if_vx_not_equals_vy() {
        let mut chip8 = get_chip8_instance();

        chip8.v_registers[10] = 0x11;
        chip8.v_registers[11] = 0x20;

        set_initial_opcode_to(0x9AB0, &mut chip8.memory);

        chip8.emulate_cycle();

        assert_eq!(chip8.program_counter, 0x204);
    }

    #[test]
    fn it_doesnt_skip_the_next_instruction_if_vx_equals_vy() {
        let mut chip8 = get_chip8_instance();

        chip8.v_registers[10] = 0x11;
        chip8.v_registers[11] = 0x11;

        set_initial_opcode_to(0x9AB0, &mut chip8.memory);

        chip8.emulate_cycle();

        assert_eq!(chip8.program_counter, 0x202);
    }

    #[test]
    fn it_sets_the_index_register_value() {
        let mut chip8 = get_chip8_instance();

        set_initial_opcode_to(0xA111, &mut chip8.memory);

        chip8.emulate_cycle();

        assert_eq!(chip8.index_register, 0x111);
    }

    #[test]
    fn it_sets_the_value_of_vx() {
        let mut chip8 = get_chip8_instance();
        chip8.v_registers[4] = 0xF;
        set_initial_opcode_to(0x6423, &mut chip8.memory);

        chip8.emulate_cycle();

        assert_eq!(chip8.v_registers[4], 0x23);
    }

    #[test]
    fn it_adds_the_value_to_vx() {
        let mut chip8 = get_chip8_instance();
        chip8.v_registers[1] = 0x10;
        set_initial_opcode_to(0x7110, &mut chip8.memory);

        chip8.emulate_cycle();

        assert_eq!(chip8.v_registers[1], 0x20);
    }

    #[test]
    fn it_sets_the_value_of_vx_to_vy() {
        let mut chip8 = get_chip8_instance();
        chip8.v_registers[1] = 0x10;
        chip8.v_registers[2] = 0x20;
        set_initial_opcode_to(0x8120, &mut chip8.memory);

        chip8.emulate_cycle();

        assert_eq!(chip8.v_registers[1], 0x20);
    }

    #[test]
    fn it_sets_the_value_of_vx_to_vx_bitwise_or_vy() {
        let mut chip8 = get_chip8_instance();
        chip8.v_registers[6] = 0x10;
        chip8.v_registers[7] = 0x20;
        set_initial_opcode_to(0x8671, &mut chip8.memory);

        chip8.emulate_cycle();

        assert_eq!(chip8.v_registers[6], 0x30);
    }

    #[test]
    fn it_sets_the_value_of_vx_to_vx_bitwise_and_vy() {
        let mut chip8 = get_chip8_instance();
        chip8.v_registers[8] = 0xFF;
        chip8.v_registers[9] = 0x10;
        set_initial_opcode_to(0x8892, &mut chip8.memory);

        chip8.emulate_cycle();

        assert_eq!(chip8.v_registers[8], 0x10);
    }

    #[test]
    fn it_sets_the_value_of_vx_to_vx_bitwise_xor_vy() {
        let mut chip8 = get_chip8_instance();
        chip8.v_registers[7] = 0x72;
        chip8.v_registers[8] = 0x15;
        set_initial_opcode_to(0x8783, &mut chip8.memory);

        chip8.emulate_cycle();

        assert_eq!(chip8.v_registers[7], 0x67);
    }

    #[test]
    fn it_adds_the_value_of_vy_to_vx_setting_vf_when_there_is_a_carry() {
        let mut chip8 = get_chip8_instance();
        chip8.v_registers[0] = 0xC8;
        chip8.v_registers[1] = 0x64;
        set_initial_opcode_to(0x8014, &mut chip8.memory);

        chip8.emulate_cycle();

        // Overflowing add of 200 + 100 = 44
        assert_eq!(chip8.v_registers[0], 0x2C);
        assert_eq!(chip8.v_registers[15usize], 1);
    }

    #[test]
    fn it_subtracts_the_value_of_vy_of_vf_setting_vf_then_there_is_a_borrow() {
        let mut chip8 = get_chip8_instance();
        chip8.v_registers[0] = 0xD1;
        chip8.v_registers[1] = 0xD2;
        set_initial_opcode_to(0x8015, &mut chip8.memory);

        chip8.emulate_cycle();

        assert_eq!(chip8.v_registers[0], 0xFF);
        assert_eq!(chip8.v_registers[15usize], 1);
    }

    #[test]
    fn it_jumps_to_the_address_nnn_plus_vw0() {
        let mut chip8 = get_chip8_instance();

        chip8.v_registers[0] = 0x1;
        set_initial_opcode_to(0xB100, &mut chip8.memory);

        chip8.emulate_cycle();

        assert_eq!(chip8.program_counter, 0x301);
    }

    #[test]
    fn it_sets_vx_to_random_number_bitwise_and_nn() {
        let mut chip8 = get_chip8_instance();

        set_initial_opcode_to(0xC313, &mut chip8.memory);

        chip8.emulate_cycle();

        assert_eq!(chip8.v_registers[3], 0x1)
    }

    //0xDXYN
    #[test]
    fn it_draws_the_correct_pixels() {
        // TBD
    }

    #[test]
    fn it_skips_instruction_if_key_press() {
        let mut chip8 = get_chip8_instance();
        chip8.v_registers[5] = 8;
        chip8.keyboard[8] = 1;
        set_initial_opcode_to(0xE59E, &mut chip8.memory);

        chip8.emulate_cycle();

        assert_eq!(chip8.program_counter, 0x204);
    }

    #[test]
    fn it_skips_instruction_if_key_not_pressed() {
        let mut chip8 = get_chip8_instance();
        chip8.v_registers[3] = 6;
        chip8.keyboard[6] = 0;
        set_initial_opcode_to(0xE3A1, &mut chip8.memory);

        chip8.emulate_cycle();

        assert_eq!(chip8.program_counter, 0x204);
    }

    #[test]
    fn it_waits_for_a_keypress_and_stores_it_in_vx() {
        // Todo
    }

    #[test]
    fn it_sets_vx_to_the_value_of_the_delay_timer() {
        let mut chip8 = get_chip8_instance();
        chip8.delay_timer = 40;
        set_initial_opcode_to(0xF307, &mut chip8.memory);

        chip8.emulate_cycle();

        assert_eq!(chip8.v_registers[3], 40);
    }

    #[test]
    fn it_sets_the_delay_timer_to_vx() {
        let mut chip8 = get_chip8_instance();
        chip8.v_registers[5] = 100;
        set_initial_opcode_to(0xF515, &mut chip8.memory);

        chip8.emulate_cycle();

        assert_eq!(chip8.delay_timer, 99);
    }

    #[test]
    fn it_sets_the_sound_timer_to_vx() {
        let mut chip8 = get_chip8_instance();
        chip8.v_registers[3] = 10;
        set_initial_opcode_to(0xF318, &mut chip8.memory);

        chip8.emulate_cycle();

        assert_eq!(chip8.sound_timer, 9);
    }

    #[test]
    fn it_adds_vx_to_i() {
        let mut chip8 = get_chip8_instance();
        chip8.v_registers[8] = 0x10;
        chip8.index_register = 0x01;
        set_initial_opcode_to(0xF81E, &mut chip8.memory);

        chip8.emulate_cycle();

        assert_eq!(chip8.index_register, 0x11);
    }

    #[test]
    fn it_sets_i_to_sprite_location_read_from_vx() {
        let mut chip8 = get_chip8_instance();
        chip8.v_registers[1] = 10;
        set_initial_opcode_to(0xF129, &mut chip8.memory);

        chip8.emulate_cycle();

        assert_eq!(chip8.index_register, 10);
    }

    #[test]
    fn it_stores_bcd_of_vx_from_i() {
        let mut chip8 = get_chip8_instance();
        chip8.v_registers[9] = 123;
        chip8.index_register = 0x203;
        set_initial_opcode_to(0xF933, &mut chip8.memory);

        chip8.emulate_cycle();

        assert_eq!(chip8.memory[chip8.index_register as usize], 1);
        assert_eq!(chip8.memory[(chip8.index_register + 1) as usize], 2);
        assert_eq!(chip8.memory[(chip8.index_register + 2) as usize], 3);
    }

    #[test]
    fn it_writes_from_v0_to_vx_starting_at_memory_address_i() {
        let mut chip8 = get_chip8_instance();
        let v_registers = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
        chip8.v_registers = v_registers;
        chip8.index_register = 0x204;
        set_initial_opcode_to(0xF355, &mut chip8.memory);

        chip8.emulate_cycle();

        assert_eq!(
            chip8.memory[chip8.index_register as usize..=chip8.index_register as usize + 3],
            [0, 1, 2, 3]
        );
    }

    #[test]
    fn it_writes_to_v0_to_vx_starting_at_memory_address_i() {
        let mut chip8 = get_chip8_instance();
        chip8.index_register = 0x202;
        chip8.memory[chip8.index_register as usize] = 101;
        chip8.memory[chip8.index_register as usize + 1] = 102;
        chip8.memory[chip8.index_register as usize + 2] = 103;
        chip8.memory[chip8.index_register as usize + 3] = 104;
        set_initial_opcode_to(0xF365, &mut chip8.memory);

        chip8.emulate_cycle();

        assert_eq!(chip8.v_registers[0..=3], [101, 102, 103, 104]);
    }
}

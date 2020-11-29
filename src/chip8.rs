use std::{fs::File, io::prelude::*, io::BufReader, path::PathBuf};

use crate::{
    font_set::FONT_SET,
    types::{
        DelayTimer, Graphics, IndexRegister, Memory, Opcode, ProgramCounter, SoundTimer, Stack,
        StackPointer, VRegisters,
    },
};
pub struct Chip8 {
    opcode: Opcode,
    program_counter: ProgramCounter,
    index_register: IndexRegister,
    stack_pointer: StackPointer,
    memory: Memory,
    graphics: Graphics,
    v_registers: VRegisters,
    delay_timer: DelayTimer,
    sound_timer: SoundTimer,
    stack: Stack,
}

impl Chip8 {
    pub fn new() -> Chip8 {
        Chip8 {
            opcode: 0,
            program_counter: 0x200,
            index_register: 0,
            stack_pointer: 0,
            memory: [0; 4096],
            graphics: [0; 2048],
            v_registers: [0; 16],
            stack: [0; 16],
            delay_timer: 0,
            sound_timer: 0,
        }
    }

    pub fn initialize(&mut self) {
        self.load_font_set();
    }

    pub fn load_program(&mut self, rom_name: &str) -> Result<(), std::io::Error> {
        let mut file_path = PathBuf::from("./tmpdir");
        file_path.push(rom_name);
        let file = File::open(file_path)?;

        let mut reader = BufReader::new(file);
        let program_memory = &mut self.memory[self.program_counter as usize..];
        reader.read(program_memory)?;

        Ok(())
    }

    pub fn emulate_cycle(&mut self) {
        self.fetch_opcode();

        match self.opcode {
            0x00E0 => {
                // Clear the screen
            }
            0x00EE => {
                self.stack_pointer -= 1;
                self.program_counter = self.stack[self.stack_pointer as usize];
                self.stack[self.stack_pointer as usize] = 0;
            }
            0x0000..=0x0FFF => {
                // Calls machine code routine (RCA 1802 for COSMAC VIP) at address NNN. Not necessary for most ROMs.
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
                    self.program_counter += 4;
                } else {
                    self.program_counter += 2;
                }
            }
            0x4000..=0x4FFF => {
                let v_index = ((self.opcode & 0x0F00) >> 8) as usize;
                let v_register_value = self.v_registers[v_index];
                let value = (self.opcode & 0x00FF) as u8;

                if v_register_value != value {
                    self.program_counter += 4;
                } else {
                    self.program_counter += 2;
                }
            }
            0x5000..=0x5FFF => {
                let x_index = ((self.opcode & 0x0F00) >> 8) as usize;
                let x_register_value = self.v_registers[x_index];
                let y_index = ((self.opcode & 0x00F0) >> 4) as usize;
                let y_register_value = self.v_registers[y_index];

                if x_register_value == y_register_value {
                    self.program_counter += 4;
                } else {
                    self.program_counter += 2;
                }
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

                self.v_registers[v_register_index] += value_to_add;
                self.program_counter += 2;
            }
            0x8000..=0x8FFF => {
                match self.opcode & 0x000F {
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
                    }
                    0x000E => {
                        // 8XYE - Stores the most significant bit of VX in VF and then shifts VX to the left by 1.
                    }
                    _ => panic!("Invalid opcode"),
                }
            }
            0x9000..=0x9FFF => {
                let vy_index = ((self.opcode & 0x00F0) >> 4) as usize;
                let vx_index = ((self.opcode & 0x0F00) >> 8) as usize;
                let vy = self.v_registers[vy_index];
                let vx = self.v_registers[vx_index];

                if vx != vy {
                    self.program_counter += 2;
                }
            }
            0xA000..=0xAFFF => {
                let new_index_register_value = self.opcode & 0x0FFF;
                self.index_register = new_index_register_value;
                self.program_counter += 2;
            }
            0xB000..=0xBFFF => {
                // Jumps to the address NNN plus V0.
            }
            0xC000..=0xCFFF => {
                // Sets VX to the result of a bitwise and operation on a random number (Typically: 0 to 255) and NN.
            }
            0xD000..=0xDFFF => {
                // Draws a sprite at coordinate (VX, VY) that has a width of 8 pixels and a height of N+1 pixels. Each row of 8 pixels is read as bit-coded starting from memory location I; I value doesn’t change after the execution of this instruction. As described above, VF is set to 1 if any screen pixels are flipped from set to unset when the sprite is drawn, and to 0 if that doesn’t happen
            }
            0xE000..=0xEFFF => {
                // EX9E - Skips the next instruction if the key stored in VX is pressed. (Usually the next instruction is a jump to skip a code block)
                // EXA1 - Skips the next instruction if the key stored in VX isn't pressed. (Usually the next instruction is a jump to skip a code block)
            }
            0xF000..=0xFFFF => {
                // FX07 - Sets VX to the value of the delay timer.
                // FX0A - A key press is awaited, and then stored in VX. (Blocking Operation. All instruction halted until next key event)
                // FX15 - Sets the delay timer to VX.
                // FX18 - Sets the sound timer to VX.
                // FX1E - Adds VX to I. VF is not affected.
                // FX29 - Sets I to the location of the sprite for the character in VX. Characters 0-F (in hexadecimal) are represented by a 4x5 font.
                // FX33 - Stores the binary-coded decimal representation of VX, with the most significant of three digits at the address in I, the middle digit at I plus 1, and the least significant digit at I plus 2. (In other words, take the decimal representation of VX, place the hundreds digit in memory at location in I, the tens digit at location I+1, and the ones digit at location I+2.)
                // FX55 - Stores V0 to VX (including VX) in memory starting at address I. The offset from I is increased by 1 for each value written, but I itself is left unmodified.[d]
                // FX65 - Fills V0 to VX (including VX) with values from memory starting at address I. The offset from I is increased by 1 for each value written, but I itself is left unmodified.[d]
            }
        };

        // Update timers
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

    fn _update_timers() {
        /*
          Besides executing opcodes, the Chip 8 also has two timers you will need to implement. As mentioned above, both timers (delay timer and sound timer) count down to zero if they have been set to a value larger than zero. Since these timers count down at 60 Hz, you might want to implement something that slows down your emulation cycle (Execute 60 opcodes in one second).
        */
    }
}

#[cfg(test)]
mod test {
    use crate::test_utils::{set_initial_opcode_to, TestFile};

    use super::*;

    #[test]
    fn it_sets_the_correct_default_values() {
        let chip8 = Chip8::new();

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
        let mut chip8 = Chip8::new();

        chip8.initialize();

        assert_eq!(&chip8.memory[0..80], FONT_SET);
    }

    #[test]
    fn it_loads_the_program_to_memory() -> Result<(), std::io::Error> {
        let fake_data = b"fake_data";
        let _file = TestFile::create("IBM Logo.ch8", fake_data)?;
        let mut chip8 = Chip8::new();

        chip8.load_program("IBM Logo.ch8")?;

        assert_eq!(&chip8.memory[0x200..0x200 + fake_data.len()], fake_data);
        Ok(())
    }

    #[test]
    fn it_fetches_correct_opcode_when_emulating_the_first_cycle() {
        let mut chip8 = Chip8::new();
        chip8.memory[0x200] = 1;
        chip8.memory[0x201] = 2;

        chip8.emulate_cycle();

        // 258 = 1 << 8 | 2
        assert_eq!(chip8.opcode, 258);
    }

    #[test]
    fn it_calls_the_subroutine_at_the_correct_address() {
        let mut chip8 = Chip8::new();
        chip8.memory[0x200] = 0x20;
        chip8.memory[0x201] = 0x10;

        chip8.emulate_cycle();

        assert_eq!(chip8.stack[0], 0x200);
        assert_eq!(chip8.stack_pointer, 1);
        assert_eq!(chip8.program_counter, 0x010);
    }

    #[test]
    fn it_returns_from_a_subroutine() {
        let mut chip8 = Chip8::new();

        chip8.stack[0] = 0x123;
        chip8.stack_pointer = 1;

        set_initial_opcode_to(0x00EE, &mut chip8.memory);

        chip8.emulate_cycle();

        assert_eq!(chip8.stack[0], 0);
        assert_eq!(chip8.stack_pointer, 0);
        assert_eq!(chip8.program_counter, 0x123);
    }

    #[test]
    fn it_jumps_to_the_correct_address() {
        let mut chip8 = Chip8::new();

        set_initial_opcode_to(0x176C, &mut chip8.memory);

        chip8.emulate_cycle();

        assert_eq!(chip8.program_counter, 0x76C);
    }

    #[test]
    fn it_skips_the_next_instruction_if_vx_equals_nn() {
        let mut chip8 = Chip8::new();
        chip8.v_registers[2] = 0x6C;
        chip8.program_counter = 0x200;

        set_initial_opcode_to(0x326C, &mut chip8.memory);

        chip8.emulate_cycle();

        assert_eq!(chip8.program_counter, 0x204);
    }

    #[test]
    fn it_does_not_skip_the_next_instruction_if_vx_not_equal_nn() {
        let mut chip8 = Chip8::new();
        chip8.v_registers[2] = 0x6C;
        chip8.program_counter = 0x200;

        set_initial_opcode_to(0x326B, &mut chip8.memory);

        chip8.emulate_cycle();

        assert_eq!(chip8.program_counter, 0x202);
    }

    #[test]
    fn it_skips_the_next_instruction_if_vx_not_equals_nn() {
        let mut chip8 = Chip8::new();
        chip8.v_registers[2] = 0x6A;
        chip8.program_counter = 0x200;

        set_initial_opcode_to(0x426C, &mut chip8.memory);

        chip8.emulate_cycle();

        assert_eq!(chip8.program_counter, 0x204);
    }

    #[test]
    fn it_does_not_skip_the_next_instruction_if_vx_equal_nn() {
        let mut chip8 = Chip8::new();
        chip8.v_registers[2] = 0x6C;
        chip8.program_counter = 0x200;

        set_initial_opcode_to(0x426C, &mut chip8.memory);

        chip8.emulate_cycle();

        assert_eq!(chip8.program_counter, 0x202);
    }

    #[test]
    fn it_skips_the_next_instruction_if_vx_equals_vy() {
        let mut chip8 = Chip8::new();
        chip8.v_registers[2] = 0x6A;
        chip8.v_registers[3] = 0x6A;
        chip8.program_counter = 0x200;

        set_initial_opcode_to(0x5230, &mut chip8.memory);

        chip8.emulate_cycle();

        assert_eq!(chip8.program_counter, 0x204);
    }

    #[test]
    fn it_does_not_skip_the_next_instruction_if_vx_not_equal_vy() {
        let mut chip8 = Chip8::new();
        chip8.v_registers[2] = 0x6C;
        chip8.v_registers[5] = 0x57;
        chip8.program_counter = 0x200;

        set_initial_opcode_to(0x5250, &mut chip8.memory);

        chip8.emulate_cycle();

        assert_eq!(chip8.program_counter, 0x202);
    }

    #[test]
    fn it_stores_the_least_significant_bit_of_vx_in_vf_and_shifts_vx_to_the_right_by_1() {
        let mut chip8 = Chip8::new();

        chip8.v_registers[6] = 0b00000011;

        set_initial_opcode_to(0x86A6, &mut chip8.memory);

        chip8.emulate_cycle();

        assert_eq!(chip8.v_registers[6], 0b00000001);
        assert_eq!(chip8.v_registers[15], 0b1);
        assert_eq!(chip8.program_counter, 0x202);
    }

    #[test]
    fn it_sets_vx_to_vy_minus_vx_vf_is_set_to_0_when_there_is_a_borrow() {
        let mut chip8 = Chip8::new();

        chip8.v_registers[4] = 0x20;
        chip8.v_registers[5] = 0x11;

        set_initial_opcode_to(0x8457, &mut chip8.memory);

        chip8.emulate_cycle();

        assert_eq!(chip8.v_registers[4], 0xF);
        assert_eq!(chip8.v_registers[15], 0);
    }

    #[test]
    fn it_sets_vx_to_vy_minus_vx_vf_is_set_to_1_when_there_isnt_a_borrow() {
        let mut chip8 = Chip8::new();

        chip8.v_registers[4] = 0x11;
        chip8.v_registers[5] = 0x20;

        set_initial_opcode_to(0x8457, &mut chip8.memory);

        chip8.emulate_cycle();

        assert_eq!(chip8.v_registers[4], 0xF1);
        assert_eq!(chip8.v_registers[15], 1);
    }

    #[test]
    fn it_skips_the_next_instruction_if_vx_not_equals_vy() {
        let mut chip8 = Chip8::new();

        chip8.v_registers[10] = 0x11;
        chip8.v_registers[11] = 0x20;

        set_initial_opcode_to(0x9AB0, &mut chip8.memory);

        chip8.emulate_cycle();

        assert_eq!(chip8.program_counter, 0x202);
    }

    #[test]
    fn it_doesnt_skip_the_next_instruction_if_vx_equals_vy() {
        let mut chip8 = Chip8::new();

        chip8.v_registers[10] = 0x11;
        chip8.v_registers[11] = 0x11;

        set_initial_opcode_to(0x9AB0, &mut chip8.memory);

        chip8.emulate_cycle();

        assert_eq!(chip8.program_counter, 0x200);
    }

    #[test]
    fn it_sets_the_index_register_value() {
        let mut chip8 = Chip8::new();

        set_initial_opcode_to(0xA111, &mut chip8.memory);

        chip8.emulate_cycle();

        assert_eq!(chip8.index_register, 0x111);
        assert_eq!(chip8.program_counter, 0x202);
    }

    #[test]
    fn it_sets_the_value_of_vx() {
        let mut chip8 = Chip8::new();
        chip8.v_registers[4] = 0xF;
        set_initial_opcode_to(0x6423, &mut chip8.memory);

        chip8.emulate_cycle();

        assert_eq!(chip8.v_registers[4], 0x23);
        assert_eq!(chip8.program_counter, 0x202);
    }

    #[test]
    fn it_adds_the_value_to_vx() {
        let mut chip8 = Chip8::new();
        chip8.v_registers[1] = 0x10;
        set_initial_opcode_to(0x7110, &mut chip8.memory);

        chip8.emulate_cycle();

        assert_eq!(chip8.v_registers[1], 0x20);
        assert_eq!(chip8.program_counter, 0x202);
    }

    #[test]
    fn it_sets_the_value_of_vx_to_vy() {
        let mut chip8 = Chip8::new();
        chip8.v_registers[1] = 0x10;
        chip8.v_registers[2] = 0x20;
        set_initial_opcode_to(0x8120, &mut chip8.memory);

        chip8.emulate_cycle();

        assert_eq!(chip8.v_registers[1], 0x20);
        assert_eq!(chip8.program_counter, 0x202);
    }

    #[test]
    fn it_sets_the_value_of_vx_to_vx_bitwise_or_vy() {
        let mut chip8 = Chip8::new();
        chip8.v_registers[6] = 0x10;
        chip8.v_registers[7] = 0x20;
        set_initial_opcode_to(0x8671, &mut chip8.memory);

        chip8.emulate_cycle();

        assert_eq!(chip8.v_registers[6], 0x30);
        assert_eq!(chip8.program_counter, 0x202);
    }

    #[test]
    fn it_sets_the_value_of_vx_to_vx_bitwise_and_vy() {
        let mut chip8 = Chip8::new();
        chip8.v_registers[8] = 0xFF;
        chip8.v_registers[9] = 0x10;
        set_initial_opcode_to(0x8892, &mut chip8.memory);

        chip8.emulate_cycle();

        assert_eq!(chip8.v_registers[8], 0x10);
        assert_eq!(chip8.program_counter, 0x202);
    }

    #[test]
    fn it_sets_the_value_of_vx_to_vx_bitwise_xor_vy() {
        let mut chip8 = Chip8::new();
        chip8.v_registers[7] = 0x72;
        chip8.v_registers[8] = 0x15;
        set_initial_opcode_to(0x8783, &mut chip8.memory);

        chip8.emulate_cycle();

        assert_eq!(chip8.v_registers[7], 0x67);
        assert_eq!(chip8.program_counter, 0x202);
    }

    #[test]
    fn it_adds_the_value_of_vy_to_vx_setting_vf_when_there_is_a_carry() {
        let mut chip8 = Chip8::new();
        chip8.v_registers[0] = 0xC8;
        chip8.v_registers[1] = 0x64;
        set_initial_opcode_to(0x8014, &mut chip8.memory);

        chip8.emulate_cycle();

        // Overflowing add of 200 + 100 = 44
        assert_eq!(chip8.v_registers[0], 0x2C);
        assert_eq!(chip8.v_registers[15usize], 1);
        assert_eq!(chip8.program_counter, 0x202);
    }

    #[test]
    fn it_subtracts_the_value_of_vy_of_vf_setting_vf_then_there_is_a_borrow() {
        let mut chip8 = Chip8::new();
        chip8.v_registers[0] = 0xD1;
        chip8.v_registers[1] = 0xD2;
        set_initial_opcode_to(0x8015, &mut chip8.memory);

        chip8.emulate_cycle();

        assert_eq!(chip8.v_registers[0], 0xFF);
        assert_eq!(chip8.v_registers[15usize], 1);
        assert_eq!(chip8.program_counter, 0x202);
    }
}

use crate::chip8::Chip8Sys;
use crate::chip8error::Chip8Error;
use getrandom;

impl Chip8Sys {
    // This will run the next command in program_counter is pointing to in Chip8Sys.memory
    pub fn run(&mut self) -> Result<(), Chip8Error> {
        // check to see if we're waiting for a key press
        if self.check_waiting() {
            return Ok(());
        }
        // Delay timer
        if self.delay_timer > 0 {
            // Only decrement delay time if it's been 6 cycles to match original slow clock of
            // chip-8
            if self.dt_cycle_ct % 6 == 0 {
                self.delay_timer -= 1;
            }
            self.dt_cycle_ct += 1;
        }
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
            if self.sound_timer == 0 {
                self.is_playing_sound = false;
            }
        }
        // fetch section
        let instruction = self.memory[self.program_counter as usize];

        let a: u8 = (0xF0 & instruction) >> 0x4;
        let b: u8 = 0x0F & instruction;

        let instruction = self.memory[(self.program_counter + 1) as usize];

        let c: u8 = (0xF0 & instruction) >> 0x4;
        let d: u8 = 0x0F & instruction;
        // Once I've read the instruction increment the PC
        self.program_counter += 2;
        // Prints debug what instruction values I'm sending in
        /*
        // println!("a: {:x}", a);
        // println!("b: {:x}", b);
        // println!("c: {:x}", c);
        // println!("d: {:x}", d);
        // println!("PC inc: {:x}", self.program_counter);
        // */
        // Implement the Instructions for the Chip-8
        match a {
            0x0 => {
                // println!("Hit 0x0");
                match instruction {
                    // Clear display
                    0x00E0 => self.frame_buffer = [0x00; 256],
                    // Return from Subroutine
                    0x00EE => {
                        self.program_counter = self.stack[self.stack_pointer as usize];
                        // I don't think this is necessary but I can't pop on an array in rust.
                        self.stack[self.stack_pointer as usize] = 0;
                        self.stack_pointer -= 1;
                    }
                    // SYS addr
                    _ => (), // self.program_counter = Chip8Sys::nnn(b, c, d),
                }
            }
            0x1 => {
                // println!("Hit 0x1 - Jump");
                self.program_counter = Chip8Sys::nnn(b, c, d);
            }
            0x2 => {
                // println!("Hit 0x2 - Call addr");
                self.stack_pointer += 1;
                self.stack[self.stack_pointer as usize] = self.program_counter;
                self.program_counter = Chip8Sys::nnn(b, c, d);
            }
            0x3 => {
                // println!("Hit 0x3 - Skip if vX is NN");
                if self.register[b as usize] == Chip8Sys::nn(c, d) {
                    self.program_counter += 2;
                }
            }
            0x4 => {
                // println!("Hit 0x4 - Skip if vX is not equal to NN");
                if self.register[b as usize] != Chip8Sys::nn(c, d) {
                    self.program_counter += 2;
                }
            }
            0x5 => {
                // println!("Hit 0x5 - Skip if reg[X] == reg[Y]");
                if self.register[b as usize] == self.register[c as usize] {
                    self.program_counter += 2;
                }
            }
            0x6 => {
                // println!("Hit 0x6 - Load VX with NN");
                self.register[b as usize] = Chip8Sys::nn(c, d);
                // println!("register[{:02X}] = {:02X}", b, self.register[b as usize]);
            }
            0x7 => {
                // println!("Hit 0x7 - Add NN to reg[X]");
                let nn = Chip8Sys::nn(c, d);
                let reg_val = self.register[b as usize];
                let result: u16 = reg_val as u16 + nn as u16;
                self.register[b as usize] = (result & 0xFF) as u8;
                self.register[0xF] = ((result & 0b1_0000_0000) >> 8) as u8;
            }
            0x8 => match d {
                0 => {
                    // println!("Hit 0x8XY0 - Set reg[X] to reg[Y]");
                    self.register[b as usize] = self.register[c as usize];
                }
                1 => {
                    // println!("Hit 0x8XY1 - Set reg[X] to reg[X] OR reg[Y]");
                    self.register[b as usize] =
                        self.register[b as usize] | self.register[c as usize];
                    if self.is_register_f_reset() {
                        self.register[0xF] = 0;
                    }
                }
                2 => {
                    // println!("Hit 0x8XY2 - Set reg[X] to reg[X] AND reg[Y]");
                    self.register[b as usize] =
                        self.register[b as usize] & self.register[c as usize];
                    if self.is_register_f_reset() {
                        self.register[0xF] = 0;
                    }
                }
                3 => {
                    // println!("Hit 0x8XY3 - Set reg[X] to reg[X] XOR reg[Y]");
                    self.register[b as usize] =
                        self.register[b as usize] ^ self.register[c as usize];
                    if self.is_register_f_reset() {
                        self.register[0xF] = 0;
                    }
                }
                4 => {
                    // println!("Hit 0x8XY4 - Set reg[X] to reg[X] PLUS reg[Y]");
                    let reg_x = self.register[b as usize];
                    let reg_y = self.register[c as usize];
                    let result: u16 = reg_x as u16 + reg_y as u16;
                    // result before masking and saving it.
                    self.register[b as usize] = (result & 0xFF) as u8;
                    // set the carry bit
                    self.register[0xF] = ((result & 0b1_0000_0000) >> 8) as u8;
                    // println!("V{:X}: {:02X}, V{:X}: {:02X}, res: {:08b}, VF: {:02X}",b, self.register[b as usize],c, self.register[c as usize],result,self.register[0xF]);
                }
                5 => {
                    // println!("Hit 0x8XY5 - Set reg[X] to reg[X] MINUS reg[Y]");
                    // VF should = NOT borrow
                    // figure out if we need to deal with an overflow case
                    if self.register[b as usize] < self.register[c as usize] {
                        // calculate the two's compliment of reg[x]
                        let two_comp = (!self.register[c as usize]) as u16 + 1;
                        self.register[b as usize] =
                            ((self.register[b as usize] as u16 + two_comp as u16) & 0xFF) as u8;
                        self.register[0xF] = 0;
                    } else {
                        // otherwise we can just do it normal and set VF
                        let overflow: i16 =
                            self.register[b as usize] as i16 - self.register[c as usize] as i16;
                        self.register[b as usize] = (overflow & 0xFF) as u8;
                        self.register[0xF] = 1;
                    }
                }
                6 => {
                    // println!("Hit 0x8X_6 - Set reg[X] to reg[X] / 2 (SHR)");
                    let overflow;
                    if self.is_mod_vx_in_place() {
                        overflow = self.register[b as usize] & 0x1;
                        self.register[b as usize] >>= 1;
                    } else {
                        overflow = self.register[c as usize] & 0x1;
                        self.register[b as usize] = self.register[c as usize] >> 1;
                    }
                    // handle the overflow when shifting
                    self.register[0xF] = overflow;
                }
                7 => {
                    // println!("Hit 0x8XY7 - Set reg[X] to reg[Y] MINUS reg[X]");
                    // VF should = NOT borrow
                    // figure out if we need to deal with an overflow case
                    if self.register[b as usize] > self.register[c as usize] {
                        // calculate the two's compliment of reg[x]
                        let two_comp = (!self.register[b as usize]) as u16 + 1;
                        self.register[b as usize] =
                            ((self.register[c as usize] as u16 + two_comp as u16) & 0xFF) as u8;
                        self.register[0xF] = 0;
                    } else {
                        // otherwise we can just do it normal and set VF
                        let overflow: i16 =
                            self.register[c as usize] as i16 - self.register[b as usize] as i16;
                        self.register[b as usize] = (overflow & 0xFF) as u8;
                        self.register[0xF] = 1;
                    }
                }
                0xE => {
                    // println!("Hit 0x8X_E - Set reg[X] to reg[Y] * 2 (SHL)");
                    let overflow;
                    if self.is_mod_vx_in_place() {
                        overflow = (self.register[b as usize] & 0b1000) >> 3;
                        self.register[b as usize] <<= 1;
                    } else {
                        overflow = (self.register[c as usize] & 0b1000) >> 3;
                        self.register[b as usize] = self.register[c as usize] << 1;
                    }
                    // handle overflow for multiplication
                    self.register[0xF] = overflow;
                }
                _ => return Err(Chip8Error::Invalid0x8XYN(d)),
            },
            0x9 => {
                // println!("Hit 0x9 - Skip if X != Y");
                // if register b != register c then increment pc by 2
                if self.register[b as usize] != self.register[c as usize] {
                    self.program_counter += 2;
                }
            }
            0xA => {
                // println!("Hit 0xA - Load register I");
                self.register_i = Chip8Sys::nnn(b, c, d);
                // println!("reg I = {:02X}", self.register_i);
            }
            0xB => {
                // 0xBNNN
                self.program_counter = self.register[0] as u16 + Chip8Sys::nnn(b, c, d);
            }
            0xC => {
                // println!("Hit 0xCXNN - Set Vx to Random bite then AND with NN");
                let rng = match getrandom::u32() {
                    Ok(r) => r,
                    Err(e) => return Err(Chip8Error::IssueGeneratingRandomNum(e)),
                };
                self.register[b as usize] = ((rng & 0xFF) as u8) & Chip8Sys::nn(c, d);
            }
            0xD => {
                // println!("Hit 0xD - Draw");
                self.draw(b, c, d);
            }
            0xE => {
                // println!("Hit 0xE - key press");
                match Chip8Sys::nn(c, d) {
                    // Skip if Key reg[x] is pressed
                    0x9E => {
                        // if a value greater than 0xF somehow winds up in here return error
                        if self.register[b as usize] as usize > self.keys.len() {
                            // 0xEX9E - register X should be a value less than 0xF"
                            return Err(Chip8Error::InvalidRegisterX(b));
                        }
                        // self.register[b] has the value of the key
                        // self.keys stores if the key is pressed
                        if self.keys[self.register[b as usize] as usize] {
                            self.program_counter += 2;
                        }
                    }
                    0xA1 => {
                        // Skip if key reg[x] is not pressed
                        if self.register[b as usize] as usize > self.keys.len() {
                            // 0xEXA1 - register X should be a value less than 0xF
                            return Err(Chip8Error::InvalidRegisterX(b));
                        }
                        // self.register[b] has the value of the key
                        // self.keys stores if the key is pressed
                        if !self.keys[self.register[b as usize] as usize] {
                            self.program_counter += 2;
                        }
                    }
                    _ => return Err(Chip8Error::Invalid0xENNN(c, d)),
                }
            }

            0xF => {
                match Chip8Sys::nn(c, d) {
                    0x07 => {
                        // println!(" - Load reg[x] with delay timer");
                        self.register[b as usize] = self.delay_timer;
                    }
                    0x0A => {
                        // println!(" - Wait for key press");
                        match self.wait(b) {
                            Ok(k) => k,
                            Err(e) => return Err(e),
                        }
                    }
                    0x15 => {
                        // // println!(" - Set Delay Timer with Reg[x]'s value");
                        self.delay_timer = self.register[b as usize];
                        self.dt_cycle_ct = 0;
                    }
                    0x18 => {
                        // println!(" - Set Sound Timer with Reg[x]'s value");
                        self.sound_timer = self.register[b as usize];
                        self.is_playing_sound = true;
                    }
                    0x1E => {
                        // println!(" - Set I to I + Reg[x]");
                        self.register_i =
                            (self.register_i + self.register[b as usize] as u16) & 0xFFFF;
                    }
                    0x29 => {
                        // println!(" - Set I to location of sprite for digit Reg[x]");
                        self.register_i = 0x050 + b as u16 * 5;
                    }
                    0x33 => {
                        // println!(" - store the 100s, 10s, and 1s place of reg[x] into memory location I, I+1, and I+2 respectively");
                        let value = self.register[b as usize];
                        let places = (
                            (value as f32 / 100.).floor() as u8,
                            (value as f32 / 10.).floor() as u8,
                            (value as f32 / 1.).floor() as u8,
                        );
                        self.memory[self.register_i as usize] = places.0;
                        self.memory[self.register_i as usize + 1] = places.1 - places.0 * 10;
                        self.memory[self.register_i as usize + 2] = places.2 - places.1 * 10;
                    }
                    0x55 => {
                        // println!(" - store registers reg[0] to reg[x] to memory starting at the location stored in register I");
                        for count in 0..=b {
                            self.memory[(self.register_i + count as u16) as usize] =
                                self.register[count as usize];
                        }
                        if self.is_inc_index() {
                            self.register_i = self.register_i + b as u16 + 1;
                        }
                    }
                    0x65 => {
                        // println!(" - read register reg[0] to reg[x] out of memory starting at the location stored in register I");
                        for count in 0..=b {
                            self.register[count as usize] =
                                self.memory[(self.register_i + count as u16) as usize];
                        }
                        if self.is_inc_index() {
                            self.register_i = self.register_i + b as u16 + 1;
                        }
                    }
                    _ => return Err(Chip8Error::Invalid0xFNNN(c, d)),
                }
            }
            _ => return Err(Chip8Error::InvalidFirstByte(a)),
        }
        Ok(())
    }
    // Helper function to handle the Draw command logic 0xDXYN
    fn draw(&mut self, x: u8, y: u8, n: u8) {
        // println!("Drawing {:02X} {:02X} {:02X}", x, y, n);
        // get the x and y location out of the x and y registers
        let x_loc;
        let mut y_loc;
        //if self.is_wrap_draw() {
        x_loc = self.register[x as usize] % 64;
        y_loc = self.register[y as usize] % 32;
        //} else {
        //    x_loc = self.register[x as usize] & 0b0111_1111;
        //    y_loc = self.register[y as usize] & 0b0011_1111;
        // }
        // println!("at {}, {}", x_loc, y_loc);
        // pull the sprite's location in memory out using register I as the starting location
        let mut sprite_location = self.register_i;
        for _ in 0..n {
            // get the sprite's pixels from memory
            let sprite_pxs = self.memory[sprite_location as usize];
            // calculate the u8 (chunk) of the frame_buffer we'll be updating
            let fb_chunk_index = (y_loc * 8) + (x_loc as f32 / 8.).floor() as u8;
            // calculate the overflow (next) chunk of the frame frame_buffer
            let fb_chunk_index_next;
            let is_edge_x = (x_loc as f32 / 8.).floor() == 7.;
            if is_edge_x {
                // In the case of drawing at the edge we want to wrap around on the same row
                fb_chunk_index_next = fb_chunk_index - 7;
            } else {
                fb_chunk_index_next = fb_chunk_index + 1;
            }
            // Calculate the offset based on x's location
            let offset = x_loc % 8;
            let fb_chunk_index_original = self.frame_buffer[fb_chunk_index as usize];
            let fb_chunk_index_next_original = self.frame_buffer[fb_chunk_index_next as usize];
            // Draw the bits using xor
            self.frame_buffer[fb_chunk_index as usize] ^= sprite_pxs >> offset;
            // Update the flag if fb was 1 and became 0
            let flag_for_index = !(self.frame_buffer[fb_chunk_index as usize]
                & fb_chunk_index_original)
                & fb_chunk_index_original;
            // println!("flag_for_index: ");
            // println!("original: {:08b}", fb_chunk_index_original);
            // println!("new     : {:08b}", self.frame_buffer[fb_chunk_index as usize]);
            // println!("flag    : {:08b}", flag_for_index);
            // default to false so we don't mess with the flag during clipping
            let mut flag_for_next = 0;
            // Only do this if we're wrapping or we're clipping but not at the edge
            // println!("clipping {}, is_edge_x {}",!self.is_wrap_draw(), !is_edge_x);
            if self.is_wrap_draw() | (!self.is_wrap_draw() & !is_edge_x) {
                self.frame_buffer[fb_chunk_index_next as usize] ^=
                    (((sprite_pxs as u16) << (8 - offset)) & 0xFF) as u8;
                flag_for_next = !(self.frame_buffer[fb_chunk_index_next as usize]
                    & fb_chunk_index_next_original)
                    & fb_chunk_index_next_original;
            }
            // println!("flag_for_next: ");
            // println!("original: {:08b}", fb_chunk_index_next_original);
            // println!("new     : {:08b}", self.frame_buffer[fb_chunk_index_next as usize]);
            // println!("flag    : {:08b}", flag_for_next);
            if (flag_for_index != 0) | (flag_for_next != 0) {
                // println!("VF SET: index: {}, next: {}", flag_for_index, flag_for_next);
                self.register[0xF] = 1;
            }
            // increment Y
            y_loc += 1;

            // If we just drew on the last line of the screen start drawing at the top
            if y_loc == 32 {
                // if we're clipping just stop drawing
                if !self.is_wrap_draw() {
                    break;
                }
                y_loc = 0;
            }
            sprite_location += 1;
        }
        /*
           for byte in self.frame_buffer.iter() {
           // println!("{:08b}",byte);
           }
        // */
    }
    // helper function to get the last 3 nibbles of a command
    // commands coming in as 0x?NNN will use this
    fn nnn(b: u8, c: u8, d: u8) -> u16 {
        (b as u16) << 8 | (c << 4 | d) as u16
    }
    // helper function to get the last 2 nibbles of a command
    // commands coming in as 0x??NN will use this
    fn nn(c: u8, d: u8) -> u8 {
        c << 4 | d
    }
}

#[cfg(test)]
pub mod test {

    use super::*;

    #[test]
    // Test that the nibbles going to nn() build a byte of NN
    fn test_nn() {
        assert_eq!(0x45, Chip8Sys::nn(0x4, 0x5));
    }
    #[test]
    // Test that the nibbles going to nnn() build a byte of NNN
    fn test_nnn() {
        assert_eq!(0x456, Chip8Sys::nnn(0x4, 0x5, 0x6));
    }

    // NOTE: Section where I test all the Chip-8 instructions

    #[test]
    // Tests clear screen; 0x00E0
    fn test_clear_screen() {
        let mut chip8 = single_instruction_chip_8(0x00E0);
        chip8.frame_buffer = [0xAA; 256];
        let _ = chip8.run().unwrap();
        assert_eq!([0x00; 256], chip8.frame_buffer);
    }

    #[test]
    // Tests Jump to memory location NNN; 0x1NNN
    fn test_jump() {
        let mut chip8 = single_instruction_chip_8(0x1556);
        let _ = chip8.run().unwrap();
        assert_eq!(chip8.program_counter, 0x556);
    }

    #[test]
    // Tests Return from Subroutine; 0x00EE
    fn test_return_from_subroutine() {
        let stk_ptr = 5;
        let mut chip8 = single_instruction_chip_8(0x00EE);
        chip8.stack_pointer = stk_ptr;
        chip8.stack = [0xFF; 16];
        let _ = chip8.run().unwrap();
        // want to make sure we cleared the old stack pointer's location
        // to simulate poping something off the stack
        assert!(
            chip8.stack[stk_ptr as usize] == 0x00,
            "Chip-8 stack value should have been cleared."
        );
        // want to make sure the stack_pointer is decremented by 1
        assert!(
            chip8.stack_pointer == stk_ptr - 1,
            "Chip-8 stack pointer should have been decremented."
        );
    }

    #[test]
    // Tests Call Address; 0x2NNN
    fn test_call_address() {
        let addr = 0xF11;
        let stk_ptr = 3;
        let mut chip8 = single_instruction_chip_8(0x2000 | addr);
        chip8.stack_pointer = stk_ptr;
        let _ = chip8.run().unwrap();
        // stack pointer should be incremented by 1
        assert!(
            chip8.stack_pointer == (stk_ptr + 1),
            "Chip-8 stack pointer should have been incremented by one."
        );
        // the pc should be stored on the stack at that location
        // since this is a brand new program the pc starts at 0x200
        // and pc is incremented by 2 during fetch section so 0x202
        // println!("{:02X?}", chip8.stack);
        assert!(
            chip8.stack[chip8.stack_pointer as usize] == 0x202,
            "Chip-8 stack should have stored program counter."
        );
        // program counter is then set to F11
        assert!(
            chip8.program_counter == 0xF11,
            "Chip-8 program counter should have been set to NNN: {:03X}.",
            addr
        );
    }

    #[test]
    // Tests Skip if reg X == NN equal works; 0x3XNN
    fn test_skip_regx_eq_eq() {
        let reg_x = 3;
        let nn: u8 = 0xAA;
        let mut chip8 = single_instruction_chip_8(0x3000 | reg_x << 8 | nn as u16);
        chip8.register[reg_x as usize] = nn;
        let _ = chip8.run().unwrap();
        // program counter should be incremented by 2 if equal
        // program counter has already been updated by 2 from the fetch section
        assert_eq!(chip8.program_counter, 0x204);
    }

    #[test]
    // Tests Skip if reg X == NN not equal works; 0x3XNN
    fn test_skip_regx_eq_ne() {
        let reg_x = 3;
        let nn: u8 = 0xAA;
        let mut chip8 = single_instruction_chip_8(0x3000 | reg_x << 8 | nn as u16);
        chip8.register[reg_x as usize] = !nn;
        let _ = chip8.run().unwrap();
        // program counter should NOT be incremented since register X is not equal
        // program counter has already been updated by 2 from the fetch section
        assert_eq!(chip8.program_counter, 0x202);
    }

    #[test]
    // Tests Skip if reg X != NN equal works; 0x4XNN
    fn test_skip_regx_ne_eq() {
        let reg_x = 3;
        let nn: u8 = 0xAA;
        let mut chip8 = single_instruction_chip_8(0x4000 | reg_x << 8 | nn as u16);
        chip8.register[reg_x as usize] = !nn;
        let _ = chip8.run().unwrap();
        // program counter should be incremented by 2 if not equal
        // program counter has already been updated by 2 from the fetch section
        assert_eq!(chip8.program_counter, 0x204);
    }

    #[test]
    // Tests Skip if reg X != NN not equal works; 0x4XNN
    fn test_skip_regx_ne_ne() {
        let reg_x = 3;
        let nn: u8 = 0xAA;
        let mut chip8 = single_instruction_chip_8(0x4000 | reg_x << 8 | nn as u16);
        chip8.register[reg_x as usize] = nn;
        let _ = chip8.run().unwrap();
        // program counter should NOT be incremented since register X is equal
        // program counter has already been updated by 2 from the fetch section
        assert_eq!(chip8.program_counter, 0x202);
    }

    #[test]
    // Tests Skip if reg X == reg Y; 0x5XY_
    fn test_skip_regx_eq_regy_eq() {
        let reg_x = 3;
        let reg_y = 2;
        let reg_val = 0xAA;
        let mut chip8 = single_instruction_chip_8(0x4000 | reg_x << 8 | reg_y << 4);
        chip8.register[reg_x as usize] = reg_val;
        chip8.register[reg_y as usize] = reg_val;
        let _ = chip8.run().unwrap();
        // program counter should be incremented by 2 because reg[x] = reg[y]
        // program counter has already been updated by 2 from the fetch section
        assert_eq!(chip8.program_counter, 0x204);
    }

    #[test]
    // Tests Skip if reg X != NN not equal works; 0x5XY_
    fn test_skip_regx_eq_regy_ne() {
        let reg_x = 3;
        let reg_y = 2;
        let reg_val: u8 = 0xAA;
        let mut chip8 = single_instruction_chip_8(0x4000 | reg_x << 8 | reg_y << 4);
        chip8.register[reg_x as usize] = reg_val;
        chip8.register[reg_y as usize] = !reg_val;
        // println!("{:02X} != {:02X}", reg_val, !reg_val);
        let _ = chip8.run().unwrap();
        // program counter should NOT be incremented since reg[x] != reg[y]
        // program counter has already been updated by 2 from the fetch section
        assert_ne!(
            chip8.program_counter, 0x202,
            "Chip-8 program counter should not have been incremented."
        );
    }

    #[test]
    // Tests Load Register X with NN; 0x6XNN
    fn test_load_register() {
        // set register 0xA to be 0x88
        let mut chip8 = single_instruction_chip_8(0x6A88);
        let _ = chip8.run().unwrap();
        assert_eq!(0x88, chip8.register[0xA]);
    }

    #[test]
    // Tests Add Value NN to Register X; 0x7XNN
    fn test_add_register() {
        // add 0x0A to Register A
        let mut chip8 = single_instruction_chip_8(0x7A0B);
        // directly access the register for testing purposes
        // println!("sum: {:02X}", 0x04 + 0x0B);
        chip8.register[0xA] = 0x04;
        let _ = chip8.run().unwrap();
        // 0x0B + 0x04 = 0x10
        assert_eq!(0x0F, chip8.register[0xA]);
    }

    #[test]
    // Tests Add Value NN to Register X with Overflow; 0x7XNN
    fn test_add_register_overflow() {
        let reg_x = 0xA;
        let val = 0xFF;
        let mut chip8 = single_instruction_chip_8(0x7000 | reg_x << 8 | 1);
        chip8.register[reg_x as usize] = val;
        let _ = chip8.run().unwrap();
        assert_eq!(
            chip8.register[reg_x as usize], 0,
            "Chip-8 0x7XNN should have added 1 to 0xFF making it 0."
        );
        assert_eq!(
            chip8.register[0xF], 1,
            "Chip-8 0x7XNN should have set the overflow bit when adding 1 to 0xFF."
        );
    }

    #[test]
    #[should_panic]
    // Tests that Chip8Sys::run() panics if you send an invalid N value for 0x8XYN
    fn test_invalid_0x8xyn_instruction_panics() {
        let mut chip8 = single_instruction_chip_8(0x8A0B);
        let _ = chip8.run().unwrap();
    }
    #[test]
    // Tests set reg[X] to reg[Y]; 0x8XY0
    fn test_set_regx_to_regy() {
        let reg_x = 4;
        let reg_y = 0xA;
        let test_val = 0x55;
        let mut chip8 = single_instruction_chip_8(0x8000 | reg_x << 8 | reg_y << 4);
        chip8.register[reg_y as usize] = test_val;
        let _ = chip8.run().unwrap();
        assert_eq!(chip8.register[reg_x as usize], test_val);
    }

    #[test]
    // Tests set reg[X] to reg[X] OR reg[Y]; 0x8XY1
    fn test_or_regx_to_regy() {
        let reg_x = 4;
        let reg_y = 0xA;
        let test_val = 0x55;
        let mut chip8 = single_instruction_chip_8(0x8000 | reg_x << 8 | reg_y << 4 | 1);
        chip8.register[reg_y as usize] = test_val;
        chip8.register[reg_x as usize] = !test_val;
        let _ = chip8.run().unwrap();
        assert_eq!(
            chip8.register[reg_x as usize],
            test_val | !test_val, // should be 0xFF
            "Chip-8 0x8XY1 should have set reg x to reg x OR reg y."
        );
    }

    #[test]
    // Tests set reg[x] to reg[x] AND reg[y]; 0x8XY2
    fn test_and_regx_regy() {
        let reg_x = 4;
        let reg_y = 0xA;
        let mut chip8 = single_instruction_chip_8(0x8000 | reg_x << 8 | reg_y << 4 | 2);
        chip8.register[reg_y as usize] = 0x0F;
        chip8.register[reg_x as usize] = 0xA5;
        let _ = chip8.run().unwrap();
        assert_eq!(
            // 0x0F & 0xA5 == 0x05
            chip8.register[reg_x as usize],
            0x05,
            "Chip-8 0x8XY1 should have set reg x to reg x AND reg y."
        );
    }
    #[test]
    // Tests set reg[x] to reg[x] XOR reg[y]; 0x8XY3
    fn test_xor_regx_regy() {
        let reg_x = 2;
        let reg_y = 1;
        let mut chip8 = single_instruction_chip_8(0x8000 | reg_x << 8 | reg_y << 4 | 3);
        chip8.register[reg_x as usize] = 0xFA;
        chip8.register[reg_y as usize] = 0xAF;
        let _ = chip8.run().unwrap();
        assert_eq!(
            chip8.register[reg_x as usize], 0x55,
            "Chip-8 0x8XY3 should have set reg x to reg x XOR reg y"
        );
    }

    #[test]
    // Tests set reg[x] to reg[x] PLUS reg[y]; 0x8XY4
    fn test_add_regx_regy() {
        let reg_x = 0;
        let reg_y = 0xE;
        let mut chip8 = single_instruction_chip_8(0x8000 | reg_x << 8 | reg_y << 4 | 4);
        chip8.register[reg_x as usize] = 0x01;
        chip8.register[reg_y as usize] = 0x09;
        let _ = chip8.run().unwrap();
        assert_eq!(
            chip8.register[reg_x as usize], 0x0A,
            "Chip-8 0x8XY4 should have set reg x to reg x PLUS reg y"
        );
    }
    #[test]
    // Tests reg x + reg y sets the overflow; 0x8XY4
    fn test_add_regx_regy_overflow() {
        let reg_x = 0;
        let reg_y = 0xE;
        let mut chip8 = single_instruction_chip_8(0x8000 | reg_x << 8 | reg_y << 4 | 4);
        chip8.register[reg_x as usize] = 0xFF;
        chip8.register[reg_y as usize] = 0x01;
        let _ = chip8.run().unwrap();
        assert_eq!(
            chip8.register[reg_x as usize], 0x00,
            "Chip-8 0x8XY4 should have set reg x to 0 after adding 1 to 0xFF."
        );
        assert_eq!(
            chip8.register[0xf], 0x1,
            "Chip-8 0x8XY4 should have set the overflow bit when adding 1 to 0xFF."
        );
    }
    #[test]
    // Tests that you can use VF as a register and have it overwritten
    fn test_add_regx_regy_vf_overwrite() {
        let reg_x = 0xF;
        let reg_y = 0xE;
        let mut chip8 = single_instruction_chip_8(0x8000 | reg_x << 8 | reg_y << 4 | 4);
        chip8.register[reg_x as usize] = 0xFF;
        chip8.register[reg_y as usize] = 0x0F;
        let _ = chip8.run().unwrap();
        assert_eq!(
            chip8.register[reg_x as usize], 0x01,
            "Chip-8 0x8XY4 should have set reg x to 1 due to overwrite after adding 0xF to 0xFF."
        );
    }
    #[test]
    // Tests set reg[x] to reg[x] MINUS reg[y]; 0x8XY5
    fn test_sub_regx_regy() {
        let reg_x = 0xE;
        let reg_y = 0xD;
        let mut chip8 = single_instruction_chip_8(0x8000 | reg_x << 8 | reg_y << 4 | 5);
        chip8.register[reg_x as usize] = 0x0F;
        chip8.register[reg_y as usize] = 0x04;
        let _ = chip8.run().unwrap();
        assert_eq!(
            chip8.register[reg_x as usize], 0x0B,
            "Chip-8 0x8XY5 should have set reg x to reg x PLUS reg y"
        );
        // also need to make sure the carry bit was set since this should still be a
        // positive number (yes that's feels backwards but VF = NOT borrow according to docs)
        assert_eq!(
            chip8.register[0xF], 1,
            "Chip-8 register F should not have been set."
        );
    }

    #[test]
    // Tests set reg[x] to reg[x] MINUS reg[y] with carry bit set; 0x8XY5
    fn test_sub_regx_regy_with_overflow() {
        let reg_x = 0xE;
        let reg_y = 0xD;
        let mut chip8 = single_instruction_chip_8(0x8000 | reg_x << 8 | reg_y << 4 | 5);
        chip8.register[reg_x as usize] = 0x04;
        chip8.register[reg_y as usize] = 0x08;
        let _ = chip8.run().unwrap();
        assert_eq!(
            chip8.register[reg_x as usize],
            // this is -4 in binary if you take the MSB as a sign bit
            0b1111_1100,
            "Chip-8 0x8XY5 should have set reg x to reg x PLUS reg y"
        );
        // also need to make sure the carry bit was set since this should still be a
        // positive number (yes that's feels backwards but VF = NOT borrow according to docs)
        assert_eq!(
            chip8.register[0xF], 0,
            "Chip-8 register F should not have been set."
        );
    }
    #[test]
    // Test that reg x MINUS reg y will overwite the reg x if it's set to reg 0xF
    fn test_sub_regx_regy_overwrite() {
        let reg_x = 0xF;
        let reg_y = 0xD;
        let mut chip8 = single_instruction_chip_8(0x8000 | reg_x << 8 | reg_y << 4 | 5);
        chip8.register[reg_x as usize] = 0x08;
        chip8.register[reg_y as usize] = 0x04;
        let _ = chip8.run().unwrap();
        // also need to make sure the carry bit was set since this should still be a
        // positive number (yes that's feels backwards but VF = NOT borrow according to docs)
        assert_eq!(
            chip8.register[0xF], 1,
            "Chip-8 0x8XY5 register F should have been set to 1 and overwritten reg x - reg y value."
        );
    }
    #[test]
    // Tests set reg[x] to reg[x] divide by 2 without carry; 0x8X_6
    // Assumes reg x is reg y shifted
    fn test_div2_regx_regy() {
        let reg_x = 0x2;
        let reg_y = 0xA;
        let val = 0xC;
        let mut chip8 = single_instruction_chip_8(0x8000 | reg_x << 8 | reg_y << 4 | 6);
        chip8.register[reg_y as usize] = val; // 12 in dec
        let _ = chip8.run().unwrap();
        assert_eq!(
            chip8.register[reg_x as usize],
            (val as f32 / 2.) as u8,
            "Chip-8 0x8X_6 did not divide register x by 2"
        );
        // check the carry which shouldn't change because it's even
        assert_eq!(
            chip8.register[0xF], 0,
            "Chip-8 0x8X_6 reg F bit should not be set for even division"
        );
    }

    #[test]
    // Tests set reg[x] to reg[x] divide by 2 with carry; 0x8X_6
    // Assumes reg x is reg y shifted
    fn test_div2_regx_odd_nums_regy() {
        let reg_x = 0x2;
        let reg_y = 0xA;
        let val = 0xD; // 13 in dec;
        let mut chip8 = single_instruction_chip_8(0x8000 | reg_x << 8 | reg_y << 4 | 6);
        chip8.register[reg_y as usize] = val;
        let _ = chip8.run().unwrap();
        assert_eq!(
            chip8.register[reg_x as usize],
            (val as f32 / 2.).floor() as u8,
            "Chip-8 0x8X_6 register x should be divided by two and rounded down"
        );
        // check the carry which shouldn't change because it's even
        assert_eq!(
            chip8.register[0xF], 1,
            "Chip-8 0x8X_6 reg F bit should be set for odd number division"
        );
    }
    #[test]
    // Tests set reg[x] to reg[x] divide by 2 without carry; 0x8X_6
    // Assumes reg x is reg y shifted
    fn test_div2_regx() {
        let reg_x = 0x2;
        let mut chip8 = single_instruction_chip_8_custom_quirks(
            0x8000 | reg_x << 8 | 6,
            false,
            false,
            false,
            true,
        );
        chip8.register[reg_x as usize] = 0xC; // 12 in dec
        let _ = chip8.run().unwrap();
        assert_eq!(
            chip8.register[reg_x as usize], 0x6,
            "Chip-8 0x8X_6 did not divide register x by 2"
        );
        // check the carry which shouldn't change because it's even
        assert_eq!(
            chip8.register[0xF], 0,
            "Chip-8 0x8X_6 reg F bit should not be set for even division"
        );
    }

    #[test]
    // Tests set reg[x] to reg[x] divide by 2 with carry; 0x8X_6
    // Assumes reg x is reg y shifted
    fn test_div2_regx_odd_nums() {
        let reg_x = 0x2;
        let mut chip8 = single_instruction_chip_8_custom_quirks(
            0x8000 | reg_x << 8 | 6,
            false,
            false,
            false,
            true,
        );
        chip8.register[reg_x as usize] = 0xD; // 13 in dec
        let _ = chip8.run().unwrap();
        assert_eq!(
            chip8.register[reg_x as usize], 0x6,
            "Chip-8 0x8X_6 register x should be divided by two and rounded down"
        );
        // check the carry which shouldn't change because it's even
        assert_eq!(
            chip8.register[0xF], 1,
            "Chip-8 0x8X_6 reg F bit should be set for odd number division"
        );
    }
    #[test]
    // Tests set reg[x] to reg[y] MINUS reg[x]; 0x8XY7
    fn test_subn_regx_regy() {
        let reg_x = 0xE;
        let reg_y = 0xD;
        let mut chip8 = single_instruction_chip_8(0x8000 | reg_x << 8 | reg_y << 4 | 7);
        chip8.register[reg_x as usize] = 0x04;
        chip8.register[reg_y as usize] = 0x0F;
        let _ = chip8.run().unwrap();
        assert_eq!(
            chip8.register[reg_x as usize], 0x0B,
            "Chip-8 0x8XY5 should have set reg x to reg x PLUS reg y"
        );
        // also need to make sure the carry bit was set since this should still be a
        // positive number (yes that's feels backwards but VF = NOT borrow according to docs)
        assert_eq!(
            chip8.register[0xF], 1,
            "Chip-8 register F should not have been set."
        );
    }

    #[test]
    // Tests set reg[x] to reg[y] MINUS reg[x]; 0x8XY7
    fn test_subn_regx_regy_with_overflow() {
        let reg_x = 0xE;
        let reg_y = 0xD;
        let mut chip8 = single_instruction_chip_8(0x8000 | reg_x << 8 | reg_y << 4 | 7);
        chip8.register[reg_x as usize] = 0x08;
        chip8.register[reg_y as usize] = 0x04;
        let _ = chip8.run().unwrap();
        assert_eq!(
            chip8.register[reg_x as usize],
            // this is -4 in raw binary if you use the MSB as a sign bit
            0b1111_1100,
            "Chip-8 0x8XY5 should have set reg x to reg x PLUS reg y"
        );
        // also need to make sure the carry bit was set since this should still be a
        // positive number (yes that's feels backwards but VF = NOT borrow according to docs)
        assert_eq!(
            chip8.register[0xF], 0,
            "Chip-8 register F should not have been set."
        );
    }

    #[test]
    // Tests set reg[x] to reg[x] multiplied by 2; 0x8X_E
    // Assumin that VX is modded in place
    fn test_2x_regx() {
        let reg_x = 0x4;
        let mut chip8 = single_instruction_chip_8_custom_quirks(
            0x8000 | reg_x << 8 | 0xE,
            false,
            false,
            false,
            true,
        );
        chip8.register[reg_x as usize] = 0x4;
        let _ = chip8.run().unwrap();
        assert_eq!(
            chip8.register[reg_x as usize], 0x8,
            "Chip-8 0x8X_E should have multiplied register x by 2."
        );
        // test the carry
        assert_eq!(
            chip8.register[0xF], 0,
            "Chip-8 0x8X_E should not have set the reg F bit."
        )
    }
    #[test]
    // Tests set reg[x] to reg[x] multiplied by 2; 0x8X_E
    // Assumin that VX is modded in place
    fn test_2x_regx_overflow() {
        let reg_x = 0x4;
        let val = 0xAA;
        let mut chip8 = single_instruction_chip_8_custom_quirks(
            0x8000 | reg_x << 8 | 0xE,
            true,
            true,
            false,
            true,
        );
        chip8.register[reg_x as usize] = val;
        let _ = chip8.run().unwrap();
        assert_eq!(
            chip8.register[reg_x as usize],
            // mulitply and mask off the overflow bits so it matches chip8's out
            ((val as u16 * 2) & 0xFF) as u8,
            "Chip-8 0x8X_E should have multiplied register y by 2 as stored it in x."
        );
        // test the carry
        assert_eq!(
            chip8.register[0xF], 1,
            "Chip-8 0x8X_E should have set the reg F bit to denote overflow."
        )
    }

    #[test]
    // Tests set reg[x] to reg[x] multiplied by 2; 0x8X_E
    // Assuming that VX is set to shifted VY
    fn test_2x_regx_regy() {
        let reg_x = 0x4;
        let reg_y = 0x05;
        let mut chip8 = single_instruction_chip_8(0x8000 | reg_x << 8 | reg_y << 4 | 0xE);
        chip8.register[reg_y as usize] = 0x4;
        let _ = chip8.run().unwrap();
        assert_eq!(
            chip8.register[reg_x as usize], 0x8,
            "Chip-8 0x8X_E should have multiplied register x by 2."
        );
        // test the carry
        assert_eq!(
            chip8.register[0xF], 0,
            "Chip-8 0x8X_E should not have set the reg F bit."
        )
    }
    #[test]
    // Tests set reg[x] to reg[y] multiplied by 2; 0x8XYE
    // Assuming that VX is set to shifted VY
    fn test_2x_regx_overflow_regy() {
        let reg_x = 0x4;
        let reg_y = 0x05;
        let val = 0xAA;
        let mut chip8 = single_instruction_chip_8(0x8000 | reg_x << 8 | reg_y << 4 | 0xE);
        chip8.register[reg_y as usize] = val;
        let _ = chip8.run().unwrap();
        assert_eq!(
            chip8.register[reg_x as usize],
            // mulitply and mask off the overflow bits so it matches chip8's out
            ((val as u16 * 2) & 0xFF) as u8,
            "Chip-8 0x8X_E should have multiplied register x by 2."
        );
        // test the carry
        assert_eq!(
            chip8.register[0xF], 1,
            "Chip-8 0x8X_E should have set the reg F bit to denote overflow."
        )
    }

    #[test]
    // Tests SKip next instruction if reg[X] != reg[Y]; 0x9XY0
    fn test_skip_if_ne_skip() {
        let reg_x = 0x1;
        let reg_y = 0x4;
        let mut chip8 = single_instruction_chip_8(0x9000 | reg_x << 8 | reg_y << 4);
        chip8.register[reg_x as usize] = 0x6;
        chip8.register[reg_y as usize] = chip8.register[reg_x as usize] + 0x2;
        let _ = chip8.run().unwrap();
        assert_eq!(
            chip8.program_counter, 0x204,
            "Chip-8 0x9XY0 should have incremented the program counter"
        );
    }
    #[test]
    // Tests Skip next instruction if reg[X] != reg[Y]; 0x9XY0
    // Testing the negative case (where it doesn't skip forward bc x=y)
    fn test_skip_if_ne_no_skip() {
        let reg_x = 0x1;
        let reg_y = 0x4;
        let mut chip8 = single_instruction_chip_8(0x9000 | reg_x << 8 | reg_y << 4);
        chip8.register[reg_x as usize] = 0x6;
        chip8.register[reg_y as usize] = chip8.register[reg_x as usize];
        let _ = chip8.run().unwrap();
        assert_eq!(
            chip8.program_counter, 0x202,
            "Chip-8 0x9XY0 should not have incremented the program counter"
        );
    }
    #[test]
    // Tests Setting Index Register I to NNN; 0xANNN
    fn test_set_register_i() {
        // Set register I to 0x9A9
        let mut chip8 = single_instruction_chip_8(0xA9A9);
        let _ = chip8.run().unwrap();
        assert_eq!(chip8.register_i, 0x9A9);
    }
    #[test]
    // Tests jump to location offset from reg 0; 0xBNNN
    fn test_jump_offset_reg0() {
        let nnn = 0x213;
        let v0 = 0x50;
        let mut chip8 = single_instruction_chip_8(0xB000 | nnn);
        chip8.register[0] = v0;
        let _ = chip8.run().unwrap();
        assert_eq!(
            chip8.program_counter,
            v0 as u16 + nnn,
            "Chip-8 0xBNNN should have set PC to reg[0] + NNN. reg[0]: {:X}. NNN: {:X}",
            chip8.register[0],
            nnn
        );
    }
    #[test]
    // Tests that a random number stores to reg[x]
    // might fail in the 1/(256^2) chance that you get 0 randomly generated twice
    fn test_random_storage() {
        let nn = 0xFF;
        let reg_x = 0x4;
        let mut chip8 = single_instruction_chip_8(0xC000 | reg_x << 8 | nn);
        let _ = chip8.run().unwrap();
        if chip8.register[reg_x as usize] == 0 {
            // in the unlikely case we get 0 the first time re run it.
            chip8.program_counter = 0x200;
            let _ = chip8.run().unwrap();
        }
        assert_ne!(
            chip8.register[reg_x as usize], 0,
            "Chip-8 0xCXNN should have set register X with a random number."
        );
    }

    #[test]
    // Tests that the random number generator will AND with NN; 0xCXNN
    fn test_random_and_0() {
        let nn = 0x00;
        let reg_x = 0x4;
        let mut chip8 = single_instruction_chip_8(0xC000 | reg_x << 8 | nn);
        let _ = chip8.run().unwrap();
        if chip8.register[reg_x as usize] == 0 {
            // in the unlikely case we get 0 the first time re run it.
            chip8.program_counter = 0x200;
            let _ = chip8.run().unwrap();
        }
        assert_eq!(
            chip8.register[reg_x as usize], 0,
            "Chip-8 0xCXNN should have set register X to reg[X] & 0 (which is 0)."
        );
    }

    #[test]
    // Tests Draw sprite in reg_i that's N pixels tall
    // in the frame_buffer at location stored in register X, and register Y;
    // 0xDXYN
    fn test_draw() {
        let mut chip8 = single_instruction_chip_8(0xD125);
        // load register 1 with X location
        chip8.register[0x1] = 0xF;
        // load register 2 with Y location
        chip8.register[0x2] = 0x8;
        // set register I to reference the sprite for 0 in memory 0x050
        chip8.register_i = 0x050;
        let _ = chip8.run().unwrap();
        // make the expected frame empty
        let mut expected_frame_buffer = [0; 256];
        // manually load the 0 sprite into the right spots
        expected_frame_buffer[65] = 0b0000_0001;
        expected_frame_buffer[66] = 0b1110_0000;
        expected_frame_buffer[73] = 0b0000_0001;
        expected_frame_buffer[74] = 0b0010_0000;
        expected_frame_buffer[81] = 0b0000_0001;
        expected_frame_buffer[82] = 0b0010_0000;
        expected_frame_buffer[89] = 0b0000_0001;
        expected_frame_buffer[90] = 0b0010_0000;
        expected_frame_buffer[97] = 0b0000_0001;
        expected_frame_buffer[98] = 0b1110_0000;

        assert_eq!(chip8.frame_buffer, expected_frame_buffer);
    }
    #[test]
    #[should_panic]
    // Tests that if you sent the incorrect NN value for 0xEXNN Chip8Sys::run() panics
    fn test_invalid_0xe_instruction_panics() {
        let mut chip8 = single_instruction_chip_8(0xE000 | 0xFF);
        let _ = chip8.run().unwrap();
    }

    #[test]
    #[should_panic]
    // Tests that if you pass an out of bounds key address for 0xEX9E Chip8Sys::run() panics
    fn test_out_of_bound_key_panics_skip_pressed() {
        let reg_x = 0x1;
        let mut chip8 = single_instruction_chip_8(0xE000 | reg_x << 8 | 0x9E);
        chip8.register[reg_x as usize] = 0xF0;
        let _ = chip8.run().unwrap();
    }
    #[test]
    // Tests Skip if key with value of reg[x] is pressed; 0xEX9E
    fn test_skip_if_key_pressed() {
        let key = 0xB;
        let reg_x = 0x3;
        let mut chip8 = single_instruction_chip_8(0xE000 | reg_x << 8 | 0x9E);
        chip8.register[reg_x as usize] = 0xB;
        chip8.keys[key] = true;
        let _ = chip8.run().unwrap();
        assert_eq!(
            chip8.program_counter, 0x204,
            "Chip-8 0xEX9E should have incremented program counter on key press."
        );
    }

    #[test]
    // Tests Skip if key with value of reg[x] is pressed; 0xEX9E
    fn test_no_skip_if_key_pressed() {
        let key = 0xA;
        let reg_x = 0x3;
        let mut chip8 = single_instruction_chip_8(0xE000 | reg_x << 8 | 0x9E);
        chip8.register[reg_x as usize] = key - 1;
        chip8.keys[key as usize] = true;
        let _ = chip8.run().unwrap();
        assert_eq!(
            chip8.program_counter, 0x202,
            "Chip-8 0xEX9E should have incremented program counter on key press."
        );
    }

    #[test]
    #[should_panic]
    // Tests that if you pass an out of bounds key address for 0xEXA1 Chip8Sys::run() panics
    fn test_out_of_bound_key_panics_skip_not_pressed() {
        let reg_x = 0x1;
        let mut chip8 = single_instruction_chip_8(0xE000 | reg_x << 8 | 0xA1);
        chip8.register[reg_x as usize] = 0xF0;
        let _ = chip8.run().unwrap();
    }
    #[test]
    // Tests Skip if key with value of reg[x] is pressed; 0xEXA1
    fn test_skip_if_key_not_pressed() {
        let key = 0xB;
        let reg_x = 0x3;
        let mut chip8 = single_instruction_chip_8(0xE000 | reg_x << 8 | 0xA1);
        chip8.register[reg_x as usize] = key;
        // set all the keys to pressed
        chip8.keys = [true; 16];
        // unpress the test key
        chip8.keys[key as usize] = false;
        let _ = chip8.run().unwrap();
        assert_eq!(
            chip8.program_counter, 0x204,
            "Chip-8 0xEXA1 should have incremented program counter on key not pressed."
        );
    }

    #[test]
    // Tests Skip if key with value of reg[x] is pressed; 0xEXA1
    fn test_no_skip_if_key_not_pressed() {
        let key = 0xA;
        let reg_x = 0x3;
        let mut chip8 = single_instruction_chip_8(0xE000 | reg_x << 8 | 0xA1);
        chip8.register[reg_x as usize] = key;
        chip8.keys[key as usize] = true;
        let _ = chip8.run().unwrap();
        assert_eq!(
            chip8.program_counter, 0x202,
            "Chip-8 0xEXA1 should not have incremented program counter on key not pressed."
        );
    }

    #[test]
    #[should_panic]
    // Tests that Chip8Sys::run() panics if you send an invalid NN value for 0xFXNN
    fn test_invalid_0xf_instruction_panics() {
        let mut chip8 = single_instruction_chip_8(0xF0FF);
        let _ = chip8.run().unwrap();
    }

    #[test]
    // Tests load vx with delay timer value; 0xFX07
    fn test_load_x_with_delay_timer() {
        let reg_x = 0xA;
        let mut chip8 = single_instruction_chip_8(0xF000 | reg_x << 8 | 07);
        chip8.delay_timer = 50;
        let _ = chip8.run().unwrap();
        assert_eq!(
            chip8.register[reg_x as usize], chip8.delay_timer,
            "Chip-8 0xFX07 should have loaded dealy timer's current value into register[x]"
        );
    }

    #[test]
    // Tests that chip8 doesn't increment program counter while waiting for key press; 0xFX0A
    fn test_wait_for_key_press_pause_function() {
        let mut chip8 = single_instruction_chip_8(0xF000 | 0x0A);
        for _ in 0..4 {
            let _ = chip8.run().unwrap();
            if chip8.program_counter != 0x202 {
                panic!("Chip-8 0xFX0A should not have incremented the program counter");
            }
        }
    }

    #[test]
    // Tests that if a key is pressed it is stored into register[x]; 0xFX0A
    fn test_wait_for_key_press_store_key() {
        let reg_x = 0xA;
        let pressed_key = 0x8;
        let mut chip8 = single_instruction_chip_8(0xF000 | reg_x << 8 | 0x0A);
        let _ = chip8.run().unwrap();
        assert_eq!(
            chip8.register[reg_x as usize], 0,
            "Chip-8 0xFX0A register[x] should be 0"
        );
        chip8.keys[pressed_key as usize] = true;
        let _ = chip8.run().unwrap();
        assert_eq!(
            chip8.register[reg_x as usize], pressed_key,
            "Chip-8 0xFX0A pressed key should have been stored in register[x]."
        );
    }

    #[test]
    // Test that the highest number key pressed is stored
    fn test_wait_for_key_press_low_key_select() {
        let reg_x = 0x2;
        let pressed_key = 0x8;
        let mut chip8 = single_instruction_chip_8(0xF000 | reg_x << 8 | 0x0A);
        let _ = chip8.run().unwrap();
        assert_eq!(
            chip8.register[reg_x as usize], 0,
            "Chip-8 0xFX0A register[x] should be 0"
        );
        chip8.keys[pressed_key as usize] = true;
        chip8.keys[(pressed_key + 2) as usize] = true;
        let _ = chip8.run().unwrap();
        assert_eq!(
            chip8.register[reg_x as usize], pressed_key,
            "Chip-8 0xFX0A the lower of the two pressed key should have been stored in register[x]."
        );
    }

    #[test]
    // Tests that the next instruction is immediately acted on after a key is pressed.
    // I could do this by paying attention to the program counter but this guarantees that we're
    // actually hitting the match and acting on it.
    fn test_wait_for_key_press_immediate_next_action() {
        let reg_x = 0xA;
        let fill_screen = [0xAA; 256];
        let pressed_key = 0x8;
        let mut chip8 = single_instruction_chip_8(0xF000 | reg_x << 8 | 0x0A);
        // set the second instruction to clear screen
        chip8.memory[0x202] = 0x00;
        chip8.memory[0x203] = 0xE0;
        // fill the screen so that I can test if it got cleared
        chip8.frame_buffer = fill_screen;
        let _ = chip8.run().unwrap();
        assert_eq!(
            chip8.register[reg_x as usize], 0,
            "Chip-8 0xFX0A register[x] should be 0"
        );
        let _ = chip8.run().unwrap();
        // Nothing changed so the frame should still be filled
        assert_eq!(
            chip8.frame_buffer, fill_screen,
            "Chip-8 0xFX0A should not have moved to the clear screen instruction."
        );
        chip8.keys[pressed_key as usize] = true;
        let _ = chip8.run().unwrap();
        assert_eq!(
            chip8.frame_buffer, [0; 256],
            "Chip-8 0xFX0A pressed key should have moved to the next instruction and cleared the screen."
        );
    }

    #[test]
    // Tests the setting of delay timer with reg[x]'s value; 0xFX15
    fn test_load_dealy_timer_reg_x() {
        let reg_x = 0xC;
        let value = 0xAA;
        let mut chip8 = single_instruction_chip_8(0xF000 | reg_x << 8 | 0x15);
        chip8.register[reg_x as usize] = value;
        let _ = chip8.run().unwrap();
        assert_eq!(
            chip8.delay_timer, value,
            "Chip-8 0xFX15 should have loaded delay timer with register X's value"
        );
    }

    #[test]
    // Tests the setting of delay timer with reg[x]'s value; 0xFX18
    fn test_load_sound_timer_reg_x() {
        let reg_x = 0xC;
        let value = 0xAA;
        let mut chip8 = single_instruction_chip_8(0xF000 | reg_x << 8 | 0x18);
        chip8.register[reg_x as usize] = value;
        let _ = chip8.run().unwrap();
        assert_eq!(
            chip8.sound_timer, value,
            "Chip-8 0xFX15 should have loaded sound timer with register X's value"
        );
    }
    #[test]
    // Tests the setting of delay timer with reg[x]'s value; 0xF15
    fn test_set_i_to_i_plus_x() {
        let reg_x = 0xC;
        let i_val = 0xAA;
        let x_val = 0xAA;
        let mut chip8 = single_instruction_chip_8(0xF000 | reg_x << 8 | 0x1E);
        chip8.register[reg_x as usize] = x_val;
        chip8.register_i = i_val;
        let _ = chip8.run().unwrap();
        assert_eq!(
            chip8.register_i,
            i_val + x_val as u16,
            "Chip-8 0xFX1E should have added register X to register I"
        );
    }
    #[test]
    // Tests that I is set to X's coresponding sprite location; 0xFX29
    fn test_set_i_to_sprite() {
        // these hardcoded locations are in the docs and the initialization of chip8
        let sprite_locs: [u16; 16] = [
            0x50, 0x55, 0x5A, 0x5F, 0x64, 0x69, 0x6E, 0x73, 0x78, 0x7D, 0x82, 0x87, 0x8C, 0x91,
            0x96, 0x9B,
        ];
        for (count, loc) in sprite_locs.iter().enumerate() {
            let mut chip8 = single_instruction_chip_8(0xF000 | (count as u16) << 8 | 0x29);
            let _ = chip8.run().unwrap();
            assert_eq!(
                &chip8.register_i, loc,
                "Chip-8 0xFX29 should have set register I to sprite {:02X}'s location",
                count
            );
        }
    }

    #[test]
    // Tests the 100s, 10s, and 1s place storage; 0xFX33;
    fn test_bcd_storage() {
        let value = 123;
        let reg_x = 0x4;
        let mem_loc = 0x500;
        let mut chip8 = single_instruction_chip_8(0xF000 | reg_x << 8 | 0x33);
        chip8.register[reg_x as usize] = value;
        chip8.register_i = mem_loc;
        let _ = chip8.run().unwrap();
        assert_eq!(
            chip8.memory[mem_loc as usize], 1,
            "Chip-8 0xFX33 should have stored hundreds place into memory location stored in I"
        );
        assert_eq!(
            chip8.memory[(mem_loc + 1) as usize],
            2,
            "Chip-8 0xFX33 should have stored tens place into memory location stored in I + 1"
        );
        assert_eq!(
            chip8.memory[(mem_loc + 2) as usize],
            3,
            "Chip-8 0xFX33 should have stored ones place into memory location stored in I + 2"
        );
    }

    #[test]
    // Tests the storage of register 0 to register X into memory starting at register I
    fn test_load_x_to_memory() {
        let reg_x: u8 = 0xA;
        let val: u8 = 0xAA;
        let mem = 0x500;
        let mut chip8 = single_instruction_chip_8(0xF000 | (reg_x as u16) << 8 | 0x55);
        // load up some values for register 0 to X
        for count in 0..=reg_x {
            chip8.register[count as usize] = (val + count) & 0xFF;
        }
        chip8.register_i = mem;
        let _ = chip8.run().unwrap();
        for count in 0..=reg_x {
            assert_eq!(
                chip8.memory[(mem + count as u16) as usize],
                (val + count) & 0xFF,
                "Chip-8 0xFX55 should have set register {:02X} to {:02X}",
                count,
                val + count
            );
        }
    }

    #[test]
    // Tests the read of register 0 to register X into memory starting at register I
    fn test_read_x_to_memory() {
        let reg_x: u8 = 0xA;
        let val: u8 = 0xAA;
        let mem: u16 = 0x500;
        let mut chip8 = single_instruction_chip_8(0xF000 | (reg_x as u16) << 8 | 0x65);
        // load up some values for register 0 to X
        for count in 0..=reg_x {
            chip8.memory[(mem + count as u16) as usize] = (val + count) & 0xFF;
        }
        chip8.register_i = mem as u16;
        let _ = chip8.run().unwrap();
        for count in 0..=reg_x {
            assert_eq!(
                chip8.register[count as usize],
                (val + count) & 0xFF,
                "Chip-8 0xFX65 should have set register {:02X} to {:02X}",
                count,
                val + count
            );
        }
    }

    // NOTE: This is the format I find myself using for these tests
    // with other code sprinkled in
    #[test]
    // Tests TEMPLATE
    fn test_chip8_command() {
        let mut chip8 = single_instruction_chip_8(0x0000);
        let _ = chip8.run().unwrap();
        assert_eq!(1, 1);
    }

    // NOTE: Helper functions for testing
    // Helper function to build a Chip8Sys easily with 1 instruction at 200
    // Uses the default chip-8 quirks
    pub fn single_instruction_chip_8(instruction: u16) -> Chip8Sys {
        let mut chip8 = Chip8Sys::new_chip_8();
        chip8.memory[0x200] = ((instruction & 0xFF00) >> 8) as u8;
        chip8.memory[0x201] = (instruction & 0xFF) as u8;
        chip8
    }
    // NOTE: Helper functions for testing
    // Helper function to build a Chip8Sys easily with 1 instruction at 200
    // Uses custom quirks
    pub fn single_instruction_chip_8_custom_quirks(
        instruction: u16,
        is_inc_index: bool,
        is_vf_reset: bool,
        is_wrap: bool,
        is_shift_in_place: bool,
    ) -> Chip8Sys {
        let mut chip8 =
            Chip8Sys::new_set_quirks(is_inc_index, is_vf_reset, is_wrap, is_shift_in_place);
        chip8.memory[0x200] = ((instruction & 0xFF00) >> 8) as u8;
        chip8.memory[0x201] = (instruction & 0xFF) as u8;
        chip8
    }
}

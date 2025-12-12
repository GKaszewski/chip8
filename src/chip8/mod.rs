use rand::Rng;

#[derive(Clone, Copy, Debug)]
pub struct Quirks {
    pub shift_vy: bool, // If true, 8xy6/8xyE set Vx = Vy shift. If false, Vx = Vx shift.
}

impl Default for Quirks {
    fn default() -> Self {
        Self { shift_vy: false }
    }
}

pub struct Chip8 {
    memory: [u8; 4096],     // 4K memory
    v: [u8; 16],            // 16 8-bit registers
    pc: u16,                // program counter
    i: u16,                 // index register
    stack: [u16; 16],       // stack
    sp: usize,              // stack pointer
    timer_delay: u8,        // delay timer
    timer_sound: u8,        // sound timer
    display: [u8; 64 * 32], // display
    fontset: [u8; 80],      // fontset
    quirks: Quirks,         // Configurable quirks
}

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

impl Chip8 {
    pub fn new(quirks: Quirks) -> Self {
        let mut chip8 = Chip8 {
            memory: [0; 4096],
            v: [0; 16],
            pc: 0x200,
            i: 0,
            stack: [0; 16],
            sp: 0,
            timer_delay: 0,
            timer_sound: 0,
            display: [0; 64 * 32],
            fontset: FONT_SET,
            quirks,
        };

        chip8.initialize_memory();

        chip8
    }

    fn initialize_memory(&mut self) {
        // Load fontset into memory
        for i in 0..80 {
            self.memory[i] = self.fontset[i];
        }
    }

    pub fn load_rom(&mut self, data: &[u8]) {
        for (i, &byte) in data.iter().enumerate() {
            if i + 0x200 < self.memory.len() {
                self.memory[i + 0x200] = byte;
            }
        }
    }

    pub fn get_display(&self) -> &[u8] {
        &self.display
    }

    pub fn get_v(&self) -> &[u8] {
        &self.v
    }

    #[cfg(test)]
    pub fn get_pc(&self) -> u16 {
        self.pc
    }

    #[cfg(test)]
    pub fn get_sp(&self) -> usize {
        self.sp
    }

    #[cfg(test)]
    pub fn get_stack(&self) -> &[u16] {
        &self.stack
    }

    #[cfg(test)]
    pub fn set_v(&mut self, index: usize, value: u8) {
        self.v[index] = value;
    }

    #[cfg(test)]
    pub fn get_v_at(&self, index: usize) -> u8 {
        self.v[index]
    }

    pub fn get_timer_sound(&self) -> u8 {
        self.timer_sound
    }

    pub fn tick(&mut self, keypad: [u8; 16]) {
        let opcode = self.fetch_opcode();
        self.execute_opcode(opcode, keypad);
    }

    pub fn update_timers(&mut self) {
        if self.timer_delay > 0 {
            self.timer_delay -= 1;
        }

        if self.timer_sound > 0 {
            self.timer_sound -= 1;
        }
    }

    fn fetch_opcode(&mut self) -> u16 {
        let opcode = (self.memory[self.pc as usize] as u16) << 8
            | (self.memory[(self.pc + 1) as usize] as u16);
        self.pc += 2;
        opcode
    }

    fn execute_opcode(&mut self, opcode: u16, keypad: [u8; 16]) {
        let first_nibble = ((opcode & 0xF000) >> 12) as u8;
        let second_nibble = ((opcode & 0x0F00) >> 8) as u8;
        let third_nibble = ((opcode & 0x00F0) >> 4) as u8;
        let fourth_nibble = (opcode & 0x000F) as u8;
        let nnn = opcode & 0x0FFF;
        let kk = (opcode & 0x00FF) as u8;

        // Helper indices
        let x = second_nibble as usize;
        let y = third_nibble as usize;

        match first_nibble {
            0 => {
                if second_nibble == 0 {
                    if fourth_nibble == 0xE {
                        // 00EE - return from subroutine
                        if self.sp > 0 {
                            self.sp -= 1;
                            self.pc = self.stack[self.sp];
                        }
                    } else {
                        // clear screen
                        // For idiomatic Rust, we can use fill(0) if available or just iter mut
                        for pixel in self.display.iter_mut() {
                            *pixel = 0;
                        }
                    }
                }
            }
            1 => {
                // 1nnn - jump to address nnn
                self.pc = nnn;
            }
            2 => {
                // 2nnn - call subroutine at nnn
                if self.sp < self.stack.len() {
                    self.stack[self.sp] = self.pc;
                    self.sp += 1;
                    self.pc = nnn;
                }
            }
            3 => {
                // 3xkk - skip next instruction if Vx = kk
                if self.v[x] == kk {
                    self.pc += 2;
                }
            }
            4 => {
                // 4xkk - skip next instruction if Vx != kk
                if self.v[x] != kk {
                    self.pc += 2;
                }
            }
            5 => {
                // 5xy0 - skip next instruction if Vx = Vy
                if self.v[x] == self.v[y] {
                    self.pc += 2;
                }
            }
            6 => {
                // 6xkk - set Vx = kk
                self.v[x] = kk;
            }
            7 => {
                // 7xkk - set Vx = Vx + kk
                let (val, _) = self.v[x].overflowing_add(kk);
                self.v[x] = val;
            }
            8 => {
                match fourth_nibble {
                    0 => {
                        // 8xy0 - set Vx = Vy
                        self.v[x] = self.v[y];
                    }
                    1 => {
                        // 8xy1 - set Vx = Vx OR Vy
                        self.v[x] |= self.v[y];
                    }
                    2 => {
                        // 8xy2 - set Vx = Vx AND Vy
                        self.v[x] &= self.v[y];
                    }
                    3 => {
                        // 8xy3 - set Vx = Vx XOR Vy
                        self.v[x] ^= self.v[y];
                    }
                    4 => {
                        // 8xy4 - set Vx = Vx + Vy, set VF = carry
                        let (result, overflow) = self.v[x].overflowing_add(self.v[y]);
                        self.v[0xF] = if overflow { 1 } else { 0 };
                        self.v[x] = result;
                    }
                    5 => {
                        // 8xy5 - set Vx = Vx - Vy, set VF = NOT borrow (if Vx > Vy, then VF=1)
                        self.v[0xF] = if self.v[x] > self.v[y] { 1 } else { 0 };
                        self.v[x] = self.v[x].wrapping_sub(self.v[y]);
                    }
                    6 => {
                        // 8xy6 - SHIFTR
                        // Quirk: if shift_vy is true, Vx = Vy then shift. Else Vx = Vx then shift.
                        if self.quirks.shift_vy {
                            self.v[x] = self.v[y];
                        }
                        self.v[0xF] = self.v[x] & 1;
                        self.v[x] >>= 1;
                    }
                    7 => {
                        // 8xy7 - set Vx = Vy - Vx, set VF = NOT borrow
                        self.v[0xF] = if self.v[y] > self.v[x] { 1 } else { 0 };
                        self.v[x] = self.v[y].wrapping_sub(self.v[x]);
                    }
                    0xE => {
                        // 8xyE - SHIFTL
                        // Quirk: if shift_vy is true, Vx = Vy then shift. Else Vx = Vx then shift.
                        if self.quirks.shift_vy {
                            self.v[x] = self.v[y];
                        }
                        self.v[0xF] = (self.v[x] & 0x80) >> 7;
                        self.v[x] <<= 1;
                    }
                    _ => println!("Unknown opcode: {:X}", opcode),
                }
            }
            9 => {
                // 9xy0 - skip next instruction if Vx != Vy
                if self.v[x] != self.v[y] {
                    self.pc += 2;
                }
            }
            0xA => {
                // Annn - set I = nnn
                self.i = nnn;
            }
            0xB => {
                // Bnnn - jump to location nnn + V0
                self.pc = nnn + self.v[0] as u16;
            }
            0xC => {
                // Cxkk - set Vx = random byte AND kk
                let mut rng = rand::rng();
                let random: u8 = rng.random();
                self.v[x] = random & kk;
            }
            0xD => {
                // Dxyn - display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision
                let vx = self.v[x] as usize;
                let vy = self.v[y] as usize;
                let height = fourth_nibble as usize;
                let mut collision: u8 = 0;

                for yline in 0..height {
                    let pixel = self.memory[self.i as usize + yline];
                    for xline in 0..8 {
                        if (pixel & (0x80 >> xline)) != 0 {
                            let idx = (vx + xline + ((vy + yline) * 64)) as usize;
                            // wrapping behavior is sometimes expected but let's stick to clipping or simple check
                            // Standard Chip-8 usually wraps. But here let's stick to boundary check as before but simpler.
                            let display_len = self.display.len();
                            // Simple clipping to avoid panic
                            let actual_idx = idx % display_len;

                            if self.display[actual_idx] == 1 {
                                collision = 1;
                            }
                            self.display[actual_idx] ^= 1;
                        }
                    }
                }

                self.v[0xF] = collision;
            }
            0xE => {
                match kk {
                    0x9E => {
                        // Ex9E - skip next instruction if key with the value of Vx is pressed
                        if keypad[self.v[x] as usize] != 0 {
                            self.pc += 2;
                        }
                    }
                    0xA1 => {
                        // ExA1 - skip next instruction if key with the value of Vx is not pressed
                        if keypad[self.v[x] as usize] == 0 {
                            self.pc += 2;
                        }
                    }
                    _ => println!("Unknown opcode: {:X}", opcode),
                }
            }
            0xF => {
                match kk {
                    0x07 => {
                        // Fx07 - set Vx = delay timer value
                        self.v[x] = self.timer_delay;
                    }
                    0x0A => {
                        // Fx0A - wait for a key press, store the value of the key in Vx
                        let mut key_pressed = false;
                        for i in 0..keypad.len() {
                            if keypad[i] != 0 {
                                self.v[x] = i as u8;
                                key_pressed = true;
                            }
                        }
                        if !key_pressed {
                            self.pc -= 2; // Loop instruction
                        }
                    }
                    0x15 => {
                        // Fx15 - set delay timer = Vx
                        self.timer_delay = self.v[x];
                    }
                    0x18 => {
                        // Fx18 - set sound timer = Vx
                        self.timer_sound = self.v[x];
                    }
                    0x1E => {
                        // Fx1E - set I = I + Vx
                        self.i += self.v[x] as u16;
                    }
                    0x29 => {
                        // Fx29 - set I = location of sprite for digit Vx
                        self.i = self.v[x] as u16 * 0x5;
                    }
                    0x33 => {
                        // Fx33 - store BCD representation of Vx in memory locations I, I+1, and I+2
                        let num = self.v[x];
                        self.memory[self.i as usize] = num / 100;
                        self.memory[(self.i + 1) as usize] = (num % 100) / 10;
                        self.memory[(self.i + 2) as usize] = num % 10;
                    }
                    0x55 => {
                        // Fx55 - store registers V0 through Vx in memory starting at location I
                        for i in 0..=x {
                            self.memory[self.i as usize + i] = self.v[i];
                        }
                    }
                    0x65 => {
                        // Fx65 - read registers V0 through Vx from memory starting at location I
                        for i in 0..=x {
                            self.v[i] = self.memory[self.i as usize + i];
                        }
                    }
                    _ => println!("Unknown opcode: {:X}", opcode),
                }
            }
            _ => println!("Unknown opcode: {:X}", opcode),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state() {
        let chip8 = Chip8::new(Quirks::default());
        assert_eq!(chip8.get_pc(), 0x200);
        assert_eq!(chip8.get_sp(), 0);
    }

    #[test]
    fn test_jump_1nnn() {
        let mut chip8 = Chip8::new(Quirks::default());
        chip8.execute_opcode(0x1300, [0; 16]); // Jump to 0x300
        assert_eq!(chip8.get_pc(), 0x300);
    }

    #[test]
    fn test_call_and_return() {
        let mut chip8 = Chip8::new(Quirks::default());
        let start_pc = 0x200;

        // 2300 - Call 0x300
        chip8.execute_opcode(0x2300, [0; 16]);

        assert_eq!(chip8.get_pc(), 0x300);
        assert_eq!(chip8.get_sp(), 1);
        assert_eq!(chip8.get_stack()[0], start_pc);

        // 00EE - Return
        chip8.execute_opcode(0x00EE, [0; 16]);

        assert_eq!(chip8.get_pc(), start_pc);
        assert_eq!(chip8.get_sp(), 0);
    }

    #[test]
    fn test_nested_calls() {
        let mut chip8 = Chip8::new(Quirks::default());

        // Call 0x300
        chip8.execute_opcode(0x2300, [0; 16]);
        assert_eq!(chip8.get_pc(), 0x300);
        assert_eq!(chip8.get_sp(), 1);
        assert_eq!(chip8.get_stack()[0], 0x200);

        // Call 0x400 from 0x300
        chip8.execute_opcode(0x2400, [0; 16]);
        assert_eq!(chip8.get_pc(), 0x400);
        assert_eq!(chip8.get_sp(), 2);
        assert_eq!(chip8.get_stack()[1], 0x300);

        // Return to 0x300
        chip8.execute_opcode(0x00EE, [0; 16]);
        assert_eq!(chip8.get_pc(), 0x300);
        assert_eq!(chip8.get_sp(), 1);

        // Return to 0x200
        chip8.execute_opcode(0x00EE, [0; 16]);
        assert_eq!(chip8.get_pc(), 0x200);
        assert_eq!(chip8.get_sp(), 0);
    }

    #[test]
    fn test_skip_3xkk_equal() {
        let mut chip8 = Chip8::new(Quirks::default());
        chip8.set_v(0, 0x55);

        // 3055 - Skip next if V0 == 0x55
        // Case: Equal
        chip8.execute_opcode(0x3055, [0; 16]);
        assert_eq!(chip8.get_pc(), 0x200 + 2); // It skips (adds 2)
    }

    #[test]
    fn test_skip_3xkk_not_equal() {
        let mut chip8 = Chip8::new(Quirks::default());
        chip8.set_v(0, 0x55);

        // 30AA - Skip next if V0 == 0xAA (False)
        chip8.execute_opcode(0x30AA, [0; 16]);
        assert_eq!(chip8.get_pc(), 0x200); // No skip
    }

    #[test]
    fn test_add_7xkk() {
        let mut chip8 = Chip8::new(Quirks::default());
        chip8.set_v(0, 0x10);

        // 7005 - V0 += 0x05
        chip8.execute_opcode(0x7005, [0; 16]);
        assert_eq!(chip8.get_v_at(0), 0x15);

        // Overflow test
        chip8.set_v(0, 0xFF);
        // 7001 - V0 += 1
        chip8.execute_opcode(0x7001, [0; 16]);
        assert_eq!(chip8.get_v_at(0), 0x00);
    }

    #[test]
    fn test_add_register_8xy4() {
        let mut chip8 = Chip8::new(Quirks::default());
        chip8.set_v(0, 0x10);
        chip8.set_v(1, 0x20);

        // 8014 - V0 = V0 + V1
        chip8.execute_opcode(0x8014, [0; 16]);
        assert_eq!(chip8.get_v_at(0), 0x30);
        assert_eq!(chip8.get_v_at(0xF), 0); // No carry

        // Carry test
        chip8.set_v(0, 0xFF);
        chip8.set_v(1, 0x01);
        chip8.execute_opcode(0x8014, [0; 16]); // V0 += V1
        assert_eq!(chip8.get_v_at(0), 0x00);
        assert_eq!(chip8.get_v_at(0xF), 1); // Carry
    }

    #[test]
    fn test_timers() {
        let mut chip8 = Chip8::new(Quirks::default());
        chip8.timer_delay = 2;
        chip8.timer_sound = 2;

        chip8.update_timers();
        assert_eq!(chip8.timer_delay, 1);
        assert_eq!(chip8.timer_sound, 1);

        chip8.update_timers();
        assert_eq!(chip8.timer_delay, 0);
        assert_eq!(chip8.timer_sound, 0);

        chip8.update_timers();
        assert_eq!(chip8.timer_delay, 0); // Stops at 0
    }
}

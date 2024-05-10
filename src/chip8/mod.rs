use rand::Rng;
use raylib::prelude::*;

pub struct Chip8 {
    pub memory: [u8; 4096], // 4K memory
    pub v: [u8; 16], // 16 8-bit registers
    pub pc: u16, // program counter
    pub i: u16, // index register
    pub stack: [u16; 16], // stack
    pub timer_delay: u8, // delay timer
    pub timer_sound: u8, // sound timer
    pub display: [u8; 64 * 32], // display
    pub fontset: [u8; 80], // fontset
    pub keypad: [u8; 16], // keypad
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
    0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];

pub fn initialize () -> Chip8 {
    let mut chip8 = Chip8 {
        memory: [0; 4096],
        v: [0; 16],
        pc: 0x200,
        i: 0,
        stack: [0; 16],
        timer_delay: 0,
        timer_sound: 0,
        display: [0; 64 * 32],
        fontset: FONT_SET,
        keypad: [0; 16],
    };

    initialize_memory(&mut chip8);

    return chip8;
}

pub fn initialize_memory (chip8: &mut Chip8) {
    // Load fontset into memory
    for i in 0..80 {
        chip8.memory[i] = chip8.fontset[i];
    }

    for i in 81..4096 {
        chip8.memory[i] = 0;
    }
}

pub fn fetch_opcode(chip8: &mut Chip8) -> u16 {
    let opcode = (chip8.memory[chip8.pc as usize] as u16) << 8 | (chip8.memory[(chip8.pc + 1) as usize] as u16);
    chip8.pc += 2;
    return opcode;
}

pub fn execute_opcode(opcode: u16, chip8: &mut Chip8) {
    let first_nibble = ((opcode & 0xF000) >> 12) as u8;
    let second_nibble = ((opcode & 0x0F00) >> 8) as u8;
    let third_nibble = ((opcode & 0x00F0) >> 4) as u8;
    let fourth_nibble = (opcode & 0x000F) as u8;
    let nnn = opcode & 0x0FFF;
    let kk = (opcode & 0x00FF) as u8;

    match first_nibble  {
        0 => {
            if second_nibble == 0 {
                if fourth_nibble == 0xE {
                    // 00EE - return from subroutine
                    chip8.pc = chip8.stack[chip8.stack.len() - 1];
                } else {
                    // clear screen
                    for i in 0..chip8.display.len() {
                        chip8.display[i] = 0;
                    }
                } 
            }
        },
        1 => {
            // 1nnn - jump to address nnn
            chip8.pc = nnn;
        },
        2 => {
            // 2nnn - call subroutine at nnn
            chip8.stack[chip8.stack.len() - 1] = chip8.pc;
            chip8.pc = nnn;
        },
        3 => {
            // 3xkk - skip next instruction if Vx = kk
            if chip8.v[second_nibble as usize] == kk {
                chip8.pc += 2;
            }
        },
        4 => {
            // 4xkk - skip next instruction if Vx != kk
            if chip8.v[second_nibble as usize] != kk {
                chip8.pc += 2;
            }
        },
        5 => {
            // 5xy0 - skip next instruction if Vx = Vy
            if chip8.v[second_nibble as usize] == chip8.v[third_nibble as usize] {
                chip8.pc += 2;
            }
        },
        6 => {
            // 6xkk - set Vx = kk
            chip8.v[second_nibble as usize] = kk;
        },
        7 => {
            // 7xkk - set Vx = Vx + kk
            let vx = chip8.v[second_nibble as usize] as u16;
            let val = kk as u16;
            let sum = vx + val;
            chip8.v[second_nibble as usize] = sum as u8;

        },
        8 => {
            match fourth_nibble {
                0 => {
                    // 8xy0 - set Vx = Vy
                    chip8.v[second_nibble as usize] = chip8.v[third_nibble as usize];
                },
                1 => {
                    // 8xy1 - set Vx = Vx OR Vy
                    chip8.v[second_nibble as usize] |= chip8.v[third_nibble as usize];
                },
                2 => {
                    // 8xy2 - set Vx = Vx AND Vy
                    chip8.v[second_nibble as usize] &= chip8.v[third_nibble as usize];
                },
                3 => {
                    // 8xy3 - set Vx = Vx XOR Vy
                    chip8.v[second_nibble as usize] ^= chip8.v[third_nibble as usize];
                },
                4 => {
                    // 8xy4 - set Vx = Vx + Vy, set VF = carry
                    let vx = chip8.v[second_nibble as usize] as u16;
                    let vy = chip8.v[third_nibble as usize] as u16;
                    let result = vx + vy;
                    chip8.v[0x0F] = if result > 255 { 1 } else { 0 };
                    chip8.v[second_nibble as usize] = result as u8;
                },
                5 => {
                    // 8xy5 - set Vx = Vx - Vy, set VF = NOT borrow
                    chip8.v[second_nibble as usize] = chip8.v[second_nibble as usize].wrapping_sub(chip8.v[third_nibble as usize]);
                    chip8.v[0x0F] = if chip8.v[second_nibble as usize] > chip8.v[third_nibble as usize] { 1 } else { 0 };
                },
                6 => {
                    // 8xy6 - set Vx = Vx SHR 1
                    chip8.v[0x0F] = chip8.v[second_nibble as usize] & 1;
                    chip8.v[second_nibble as usize] >>= 1;
                },
                7 => {
                    // 8xy7 - set Vx = Vy - Vx, set VF = NOT borrow
                    chip8.v[0x0F] = if chip8.v[third_nibble as usize] > chip8.v[second_nibble as usize] { 1 } else { 0 };
                    chip8.v[second_nibble as usize] = chip8.v[third_nibble as usize].wrapping_sub(chip8.v[second_nibble as usize]);
                },
                0xE => {
                    // 8xyE - set Vx = Vx SHL 1
                    chip8.v[0x0F] = (chip8.v[second_nibble as usize] & 0b10000000) >> 7;
                    chip8.v[second_nibble as usize] <<= 1;
                },
                _ => println!("Unknown opcode: {:X}", opcode),
            }
        },
        9 => {
            // 9xy0 - skip next instruction if Vx != Vy
            if chip8.v[second_nibble as usize] != chip8.v[third_nibble as usize] {
                chip8.pc += 2;
            }
        },
        0xA => {
            // Annn - set I = nnn
            chip8.i = nnn;
        },
        0xB => {
            // Bnnn - jump to location nnn + V0
            chip8.pc = nnn + chip8.v[0] as u16;
        },
        0xC => {
            // Cxkk - set Vx = random byte AND kk
            let mut rng = rand::thread_rng();
            let random: u8 = rng.gen_range(0..255);
            chip8.v[second_nibble as usize] = random & kk;
        },
        0xD => {
            // Dxyn - display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision
            let x = chip8.v[second_nibble as usize] as usize;
            let y = chip8.v[third_nibble as usize] as usize;
            let height = fourth_nibble as usize;
            let mut collision: u8 = 0;
            for yline in 0..height {
                let pixel = chip8.memory[chip8.i as usize + yline];
                for xline in 0..8 {
                    if (pixel & (0x80 >> xline)) != 0 {
                        let index = (x + xline + ((y + yline) * 64)) as usize;
                        // check for out of bounds
                        if index < chip8.display.len() {
                            if chip8.display[index] == 1 {
                                collision = 1;
                            }
                            chip8.display[index] ^= 1;
                        } else {
                            println!("Out of bounds: {}", index);
                        }
                    }
                }
            }

            chip8.v[0xF] = collision;
        },
        0xE => {
            match kk {
                0x9E => {
                    // Ex9E - skip next instruction if key with the value of Vx is pressed
                    if chip8.keypad[chip8.v[second_nibble as usize] as usize] != 0 {
                        //println!("Key pressed: {}", chip8.keypad[chip8.v[second_nibble as usize] as usize]);
                        chip8.pc += 2;
                    }
                },
                0xA1 => {
                    // ExA1 - skip next instruction if key with the value of Vx is not pressed
                    if chip8.keypad[chip8.v[second_nibble as usize] as usize] == 0 {
                        //println!("Key not pressed: {}", chip8.keypad[chip8.v[second_nibble as usize] as usize]);
                        chip8.pc += 2;
                    }
                },
                _ => println!("Unknown opcode: {:X}", opcode),
            }
        },
        0xF => {
            match kk {
                0x07 => {
                    // Fx07 - set Vx = delay timer value
                    chip8.v[second_nibble as usize] = chip8.timer_delay;
                },
                0x0A => {
                    // Fx0A - wait for a key press, store the value of the key in Vx
                    let mut key_pressed = false;
                    for i in 0..chip8.keypad.len() {
                        if chip8.keypad[i] != 0 {
                            chip8.v[second_nibble as usize] = i as u8;
                            key_pressed = true;
                            //println!("Key pressed: {} - {}", i, chip8.keypad[chip8.v[second_nibble as usize] as usize]);
                        }
                    }
                    if !key_pressed {
                        //println!("Waiting for key press...");
                        chip8.pc -= 2;
                    }
                },
                0x15 => {
                    // Fx15 - set delay timer = Vx
                    chip8.timer_delay = chip8.v[second_nibble as usize];
                },
                0x18 => {
                    // Fx18 - set sound timer = Vx
                    chip8.timer_sound = chip8.v[second_nibble as usize];
                },
                0x1E => {
                    // Fx1E - set I = I + Vx
                    chip8.i += chip8.v[second_nibble as usize] as u16;
                },
                0x29 => {
                    // Fx29 - set I = location of sprite for digit Vx
                    chip8.i = chip8.v[second_nibble as usize] as u16 * 0x5;
                },
                0x33 => {
                    // Fx33 - store BCD representation of Vx in memory locations I, I+1, and I+2
                    let num = chip8.v[second_nibble as usize];
                    chip8.memory[chip8.i as usize] = num / 100;
                    chip8.memory[(chip8.i + 1) as usize] = (num % 100) / 10;
                    chip8.memory[(chip8.i + 2) as usize] = num % 10;
                },
                0x55 => {
                    // Fx55 - store registers V0 through Vx in memory starting at location I
                    for i in 0..second_nibble + 1 {
                        chip8.memory[chip8.i as usize + i as usize] = chip8.v[i as usize];
                    }
                },
                0x65 => {
                    // Fx65 - read registers V0 through Vx from memory starting at location I
                    for i in 0..second_nibble + 1 {
                        chip8.v[i as usize] = chip8.memory[chip8.i as usize + i as usize];
                    }
                },
                _ => println!("Unknown opcode: {:X}", opcode),
            }
        }
       _ => println!("Unknown opcode: {:X}", opcode),
    }
}

pub fn handle_keypads (chip8: & mut Chip8, rl_context: &RaylibHandle) {
    match rl_context.is_key_down(KeyboardKey::KEY_ONE) {
        true => chip8.keypad[0x1] = 1,
        false => chip8.keypad[0x1] = 0,
    }
    match rl_context.is_key_down(KeyboardKey::KEY_TWO) {
        true => chip8.keypad[0x2] = 1,
        false => chip8.keypad[0x2] = 0,
    }
    match rl_context.is_key_down(KeyboardKey::KEY_THREE) {
        true => chip8.keypad[0x3] = 1,
        false => chip8.keypad[0x3] = 0,
    }
    match rl_context.is_key_down(KeyboardKey::KEY_C) {
        true => chip8.keypad[0xC] = 1,
        false => chip8.keypad[0xC] = 0,
    }
    match rl_context.is_key_down(KeyboardKey::KEY_FOUR) {
        true => chip8.keypad[0x4] = 1,
        false => chip8.keypad[0x4] = 0,
    }
    match rl_context.is_key_down(KeyboardKey::KEY_FIVE) {
        true => chip8.keypad[0x5] = 1,
        false => chip8.keypad[0x5] = 0,
    }
    match rl_context.is_key_down(KeyboardKey::KEY_SIX) {
        true => chip8.keypad[0x6] = 1,
        false => chip8.keypad[0x6] = 0,
    }
    match rl_context.is_key_down(KeyboardKey::KEY_D) {
        true => chip8.keypad[0xD] = 1,
        false => chip8.keypad[0xD] = 0,
    }
    match rl_context.is_key_down(KeyboardKey::KEY_SEVEN) {
        true => chip8.keypad[0x7] = 1,
        false => chip8.keypad[0x7] = 0,
    }
    match rl_context.is_key_down(KeyboardKey::KEY_EIGHT) {
        true => chip8.keypad[0x8] = 1,
        false => chip8.keypad[0x8] = 0,
    }
    match rl_context.is_key_down(KeyboardKey::KEY_NINE) {
        true => chip8.keypad[0x9] = 1,
        false => chip8.keypad[0x9] = 0,
    }
    match rl_context.is_key_down(KeyboardKey::KEY_E) {
        true => chip8.keypad[0xE] = 1,
        false => chip8.keypad[0xE] = 0,
    }
    match rl_context.is_key_down(KeyboardKey::KEY_A) {
        true => chip8.keypad[0xA] = 1,
        false => chip8.keypad[0xA] = 0,
    }
    match rl_context.is_key_down(KeyboardKey::KEY_ZERO) {
        true => chip8.keypad[0x0] = 1,
        false => chip8.keypad[0x0] = 0,
    }
    match rl_context.is_key_down(KeyboardKey::KEY_B) {
        true => chip8.keypad[0xB] = 1,
        false => chip8.keypad[0xB] = 0,
    }
    match rl_context.is_key_down(KeyboardKey::KEY_F) {
        true => chip8.keypad[0xF] = 1,
        false => chip8.keypad[0xF] = 0,
    }

}

pub fn play_beep ( chip8: & mut Chip8) {
    if chip8.timer_sound > 0 {
        println!("BEEP");
    }
}
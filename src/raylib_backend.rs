use crate::platform::{DebugInfo, Platform, UiActions};
use raylib::prelude::*;

pub struct RaylibBackend {
    rl: RaylibHandle,
    thread: RaylibThread,
    colors: [Color; 19],
    current_color_index: usize,
}

const KEY_MAP: [(KeyboardKey, usize); 16] = [
    (KeyboardKey::KEY_ONE, 0x1),
    (KeyboardKey::KEY_TWO, 0x2),
    (KeyboardKey::KEY_THREE, 0x3),
    (KeyboardKey::KEY_C, 0xC),
    (KeyboardKey::KEY_FOUR, 0x4),
    (KeyboardKey::KEY_FIVE, 0x5),
    (KeyboardKey::KEY_SIX, 0x6),
    (KeyboardKey::KEY_D, 0xD),
    (KeyboardKey::KEY_SEVEN, 0x7),
    (KeyboardKey::KEY_EIGHT, 0x8),
    (KeyboardKey::KEY_NINE, 0x9),
    (KeyboardKey::KEY_E, 0xE),
    (KeyboardKey::KEY_A, 0xA),
    (KeyboardKey::KEY_ZERO, 0x0),
    (KeyboardKey::KEY_B, 0xB),
    (KeyboardKey::KEY_F, 0xF),
];

impl RaylibBackend {
    pub fn new(width: i32, height: i32, title: &str) -> Self {
        let (rl, thread) = raylib::init().size(width, height).title(title).build();

        let colors = [
            Color::RED,
            Color::BLUE,
            Color::GREEN,
            Color::YELLOW,
            Color::ORANGE,
            Color::PURPLE,
            Color::PINK,
            Color::GOLD,
            Color::LIME,
            Color::MAROON,
            Color::DARKBLUE,
            Color::DARKGREEN,
            Color::DARKPURPLE,
            Color::DARKGRAY,
            Color::GRAY,
            Color::BLACK,
            Color::WHITE,
            Color::RAYWHITE,
            Color::MAGENTA,
        ];

        RaylibBackend {
            rl,
            thread,
            colors,
            current_color_index: 0,
        }
    }

    fn handle_draw_emulator(
        d: &mut RaylibDrawHandle,
        pixels: &[u8],
        pixel_size: usize,
        pixel_color: Color,
    ) {
        for y in 0..32 {
            for x in 0..64 {
                if pixels[(y * 64) + x] == 1 {
                    d.draw_rectangle(
                        (x * pixel_size).try_into().unwrap(),
                        (y * pixel_size).try_into().unwrap(),
                        pixel_size as i32,
                        pixel_size as i32,
                        pixel_color,
                    );
                }
            }
        }
    }

    fn handle_draw_debug(d: &mut RaylibDrawHandle, debug_info: &DebugInfo, screen_width: i32) {
        if debug_info.draw_cycles_info {
            d.draw_text(
                &format!("Cycles per second: {}", debug_info.cycles_per_second),
                10,
                10,
                20,
                Color::WHITE,
            );
            d.draw_text(
                &format!("Total cycles: {}", debug_info.total_cycles),
                10,
                30,
                20,
                Color::WHITE,
            );
        }

        if debug_info.draw_registers_info {
            for i in 0..16 {
                d.draw_text(
                    &format!("V{}: {}", i, debug_info.registers[i]),
                    screen_width - 80,
                    (10 + (i * 20)).try_into().unwrap(),
                    20,
                    Color::WHITE,
                );
            }
        }
    }
}

impl Platform for RaylibBackend {
    fn should_close(&self) -> bool {
        self.rl.window_should_close()
    }

    fn process_input(&mut self) -> ([u8; 16], UiActions) {
        let mut keys = [0u8; 16];
        for (key, idx) in KEY_MAP {
            keys[idx] = if self.rl.is_key_down(key) { 1 } else { 0 };
        }

        // Handle color cycling (UI feature internal to backend)
        if self.rl.is_key_pressed(KeyboardKey::KEY_LEFT_BRACKET) {
            if self.current_color_index > 0 {
                self.current_color_index -= 1;
            } else {
                self.current_color_index = self.colors.len() - 1;
            }
        }

        if self.rl.is_key_pressed(KeyboardKey::KEY_RIGHT_BRACKET) {
            self.current_color_index += 1;
            if self.current_color_index >= self.colors.len() {
                self.current_color_index = 0;
            }
        }

        // Detect UI Actions
        let ui_actions = UiActions {
            toggle_debug_cycles: self.rl.is_key_pressed(KeyboardKey::KEY_F1),
            toggle_debug_registers: self.rl.is_key_pressed(KeyboardKey::KEY_F2),
            toggle_emulator: self.rl.is_key_pressed(KeyboardKey::KEY_F3),
            increase_speed: self.rl.is_key_pressed(KeyboardKey::KEY_PAGE_UP),
            decrease_speed: self.rl.is_key_pressed(KeyboardKey::KEY_PAGE_DOWN),
        };

        (keys, ui_actions)
    }

    fn render(&mut self, pixels: &[u8], pixel_size: usize, debug_info: Option<DebugInfo>) {
        let screen_width = self.rl.get_screen_width();
        let mut d = self.rl.begin_drawing(&self.thread);
        d.clear_background(Color::BLACK);

        Self::handle_draw_emulator(
            &mut d,
            pixels,
            pixel_size,
            self.colors[self.current_color_index],
        );

        if let Some(info) = debug_info {
            Self::handle_draw_debug(&mut d, &info, screen_width);
        }
    }

    fn play_beep(&mut self) {
        println!("BEEP");
    }

    fn get_screen_width(&self) -> i32 {
        self.rl.get_screen_width()
    }
}

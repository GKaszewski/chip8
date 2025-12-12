use chip8::*;
use clap::Parser;
use std::time::Duration;

pub mod chip8;
pub mod platform;
pub mod raylib_backend;

use platform::{DebugInfo, Platform};
use raylib_backend::RaylibBackend;

#[derive(Parser, Debug)]
#[clap(
    name = "Chip8 emulator",
    version = "0.2.0",
    author = "Gabriel Kaszewski"
)]
struct ChipCliArgs {
    #[clap(short, long, help = "Path to the ROM file")]
    rom: String,
    #[clap(
        short = 'c',
        long = "tcps",
        default_value = "500",
        help = "Target cycles per second"
    )]
    target_cycles_per_second: u32,
    #[clap(
        long = "shift-quirk",
        help = "Enable classic shift behavior (Vx = Vy >> 1)"
    )]
    shift_quirk: bool,
    #[clap(short, long, default_value = "20", help = "Pixel size")]
    pixel_size: usize,
}

struct EmulatorState {
    chip8: Chip8,
    platform: RaylibBackend,
    pixel_size: usize,
    target_cycles_per_second: u32,
    draw_debug_cycles_info: bool,
    draw_debug_registers_info: bool,
    draw_emulator: bool,
}

impl EmulatorState {
    fn new(args: ChipCliArgs) -> Self {
        let quirks = chip8::Quirks {
            shift_vy: args.shift_quirk,
        };
        let mut chip8 = Chip8::new(quirks);

        let rom_data = std::fs::read(&args.rom).expect("Failed to read ROM file");
        chip8.load_rom(&rom_data);

        let platform = RaylibBackend::new(1280, 720, "Chip8");

        EmulatorState {
            chip8,
            platform,
            pixel_size: args.pixel_size,
            target_cycles_per_second: args.target_cycles_per_second,
            draw_debug_cycles_info: false,
            draw_debug_registers_info: false,
            draw_emulator: true,
        }
    }

    fn run(&mut self) {
        let mut cycles: u64 = 0;
        let mut total_cycles: u64 = 0;
        let mut cycles_per_second: u64 = 0;
        let mut last_cycle_time = std::time::Instant::now();
        let mut last_timer_time = std::time::Instant::now();
        let timer_duration = Duration::from_nanos(1_000_000_000 / 60);

        while !self.platform.should_close() {
            let sleep_duration = Duration::from_millis(1000 / self.target_cycles_per_second as u64);

            // Stats update
            if last_cycle_time.elapsed().as_secs() >= 1 {
                cycles_per_second = cycles / last_cycle_time.elapsed().as_secs();
                total_cycles += cycles;
                cycles = 0;
                last_cycle_time = std::time::Instant::now();
            }

            // Input Handling
            let (keys, ui_actions) = self.platform.process_input();
            self.handle_ui_actions(ui_actions);

            self.chip8.tick(keys);

            self.render(cycles_per_second, total_cycles);
            self.update_audio();
            self.update_timers(timer_duration, &mut last_timer_time);

            cycles += 1;
            std::thread::sleep(sleep_duration);
        }
    }

    fn handle_ui_actions(&mut self, actions: platform::UiActions) {
        if actions.toggle_debug_cycles {
            self.draw_debug_cycles_info = !self.draw_debug_cycles_info;
        }
        if actions.toggle_debug_registers {
            self.draw_debug_registers_info = !self.draw_debug_registers_info;
        }
        if actions.toggle_emulator {
            self.draw_emulator = !self.draw_emulator;
        }
        if actions.increase_speed {
            self.target_cycles_per_second += 100;
            if self.target_cycles_per_second > 10000 {
                self.target_cycles_per_second = 10000;
            }
        }
        if actions.decrease_speed {
            if self.target_cycles_per_second > 100 {
                self.target_cycles_per_second -= 100;
            } else {
                self.target_cycles_per_second = 10;
            }
        }
    }

    fn render(&mut self, cycles_per_second: u64, total_cycles: u64) {
        let debug_info = if self.draw_debug_cycles_info || self.draw_debug_registers_info {
            Some(DebugInfo {
                draw_cycles_info: self.draw_debug_cycles_info,
                draw_registers_info: self.draw_debug_registers_info,
                cycles_per_second,
                total_cycles,
                registers: {
                    let mut regs = [0u8; 16];
                    for (i, v) in self.chip8.get_v().iter().enumerate().take(16) {
                        regs[i] = *v;
                    }
                    regs
                },
            })
        } else {
            None
        };

        if self.draw_emulator {
            self.platform
                .render(self.chip8.get_display(), self.pixel_size, debug_info);
        } else {
            let empty = [0u8; 64 * 32];
            self.platform.render(&empty, self.pixel_size, debug_info);
        }
    }

    fn update_audio(&mut self) {
        if self.chip8.get_timer_sound() > 0 {
            self.platform.play_beep();
        }
    }

    fn update_timers(
        &mut self,
        timer_duration: Duration,
        last_timer_time: &mut std::time::Instant,
    ) {
        if last_timer_time.elapsed() >= timer_duration {
            self.chip8.update_timers();
            *last_timer_time = std::time::Instant::now();
        }
    }
}

fn main() {
    let args = ChipCliArgs::parse();
    let mut app = EmulatorState::new(args);
    app.run();
}

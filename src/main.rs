use chip8::*;
use clap::Parser;
use raylib::prelude::*;

pub mod chip8;

fn load_rom_from_file(chip8: &mut Chip8, filename: &str) {
    let buffer = std::fs::read(filename).expect("Unable to read file");
    for i in 0..buffer.len() {
        chip8.memory[i + 0x200] = buffer[i];
    }
}

#[derive(Parser, Debug)]
#[clap(
    name = "Chip8 emulator",
    version = "0.1.0",
    author = "Gabriel Kaszewski"
)]
struct ChipCliArgs {
    #[clap(short, long, help = "Path to the ROM file")]
    rom: String,
    #[clap(
        short = 'c',
        long = "tcps",
        default_value = "1000",
        help = "Target cycles per second"
    )]
    target_cycles_per_second: u32,
    #[clap(short, long, default_value = "20", help = "Pixel size")]
    pixel_size: usize,
}

fn handle_draw_debug(
    d: &mut RaylibDrawHandle,
    chip8: &Chip8,
    draw_cycles_info: bool,
    draw_registers: bool,
    screen_width: i32,
    cycles_per_second: u64,
    total_cycles: u64,
) {
    if draw_cycles_info {
        d.draw_text(
            &format!("Cycles per second: {}", cycles_per_second),
            10,
            10,
            20,
            Color::WHITE,
        );
        d.draw_text(
            &format!("Total cycles: {}", total_cycles),
            10,
            30,
            20,
            Color::WHITE,
        );
    }

    if draw_registers {
        for i in 0..16 {
            d.draw_text(
                &format!("V{}: {}", i, chip8.v[i]),
                screen_width - 80,
                (10 + (i * 20)).try_into().unwrap(),
                20,
                Color::WHITE,
            );
        }
    }
}

fn handle_draw_emulator(d: &mut RaylibDrawHandle, chip8: &Chip8, pixel_size: usize, pixel_color: Color) {
    for y in 0..32 {
        for x in 0..64 {
            if chip8.display[(y * 64) + x] == 1 {
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

fn main() {
    let args: ChipCliArgs = ChipCliArgs::parse();

    // Initialize the Chip8
    let mut chip8 = initialize();
    // Load ROM into memory
    let filename = args.rom;
    load_rom_from_file(&mut chip8, &filename);

    let (mut rl, thread) = raylib::init()
        .size(1280, 720)
        .title("Chip8")
        //.vsync()
        .build();

    let pixel_size = args.pixel_size;
    let mut cycles = 0;
    let mut total_cycles = 0;
    let mut cycles_per_second = 0;
    let mut last_time = std::time::Instant::now();

    let mut draw_debug_cycles_info = false;
    let mut draw_debug_registers_info = true;
    let mut draw_emulator = true;

    let original_target_cycles_per_second = args.target_cycles_per_second;
    let mut target_cycles_per_second = args.target_cycles_per_second;
    let mut sleep_duration =
        std::time::Duration::from_millis(1000 / target_cycles_per_second as u64);

    let screen_width = rl.get_screen_width();

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
    let mut current_color_index = 0;

    // loop
    while !rl.window_should_close() {
        // calculate cycles per second and fps
        let elapsed = last_time.elapsed();
        if elapsed.as_secs() >= 1 {
            cycles_per_second = cycles / elapsed.as_secs();
            total_cycles += cycles;
            cycles = 0;
            last_time = std::time::Instant::now();
        }

        if rl.is_key_pressed(KeyboardKey::KEY_F1) {
            draw_debug_cycles_info = !draw_debug_cycles_info;
        }

        if rl.is_key_pressed(KeyboardKey::KEY_F2) {
            draw_debug_registers_info = !draw_debug_registers_info;
        }

        if rl.is_key_pressed(KeyboardKey::KEY_F3) {
            draw_emulator = !draw_emulator;
        }

        if rl.is_key_pressed(KeyboardKey::KEY_COMMA) {
            target_cycles_per_second -= 100;
            if target_cycles_per_second < 100 {
                target_cycles_per_second = 100;
            }
            sleep_duration =
                std::time::Duration::from_millis(1000 / target_cycles_per_second as u64);
        }

        if rl.is_key_pressed(KeyboardKey::KEY_PERIOD) {
            target_cycles_per_second = original_target_cycles_per_second;
            sleep_duration =
                std::time::Duration::from_millis(1000 / target_cycles_per_second as u64);
        }

        if rl.is_key_pressed(KeyboardKey::KEY_SLASH) {
            target_cycles_per_second += 100;
            sleep_duration =
                std::time::Duration::from_millis(1000 / target_cycles_per_second as u64);
        }

        if rl.is_key_pressed(KeyboardKey::KEY_LEFT_BRACKET) {
            current_color_index -= 1;
            if current_color_index < 0 {
                current_color_index = colors.len() as i32 - 1;
            }
        }

        if rl.is_key_pressed(KeyboardKey::KEY_RIGHT_BRACKET) {
            current_color_index += 1;
            if current_color_index >= colors.len() as i32 {
                current_color_index = 0;
            }
        }

        // emulate cycle
        handle_keypads(&mut chip8, &rl);
        let opcode = fetch_opcode(&mut chip8);
        execute_opcode(opcode, &mut chip8);
        //render
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);
        if draw_emulator {
            handle_draw_emulator(&mut d, &chip8, pixel_size, colors[current_color_index as usize]);
        }

        handle_draw_debug(
            &mut d,
            &chip8,
            draw_debug_cycles_info,
            draw_debug_registers_info,
            screen_width,
            cycles_per_second,
            total_cycles,
        );

        play_beep(&mut chip8);

        if chip8.timer_delay > 0 {
            chip8.timer_delay -= 1;
        }

        if chip8.timer_sound > 0 {
            chip8.timer_sound -= 1;
        }

        cycles += 1;
        std::thread::sleep(sleep_duration);
    }
}

pub trait Platform {
    fn should_close(&self) -> bool;
    fn process_input(&mut self) -> ([u8; 16], UiActions);
    /// Debug info to be optionally rendered
    fn render(&mut self, pixels: &[u8], pixel_size: usize, debug_info: Option<DebugInfo>);
    fn play_beep(&mut self);
    fn get_screen_width(&self) -> i32;
}

pub struct DebugInfo {
    pub draw_cycles_info: bool,
    pub draw_registers_info: bool,
    pub cycles_per_second: u64,
    pub total_cycles: u64,
    pub registers: [u8; 16],
}

#[derive(Default)]
pub struct UiActions {
    pub toggle_debug_cycles: bool,
    pub toggle_debug_registers: bool,
    pub toggle_emulator: bool,
    pub increase_speed: bool,
    pub decrease_speed: bool,
}

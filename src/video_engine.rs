use minifb::{Key, Window};

pub const BACK_COLOR: u32 = 0x0;
pub const FRONT_COLOR: u32 = 0xFFFFFFFF;

pub struct VideoEngine {
    video_ram: Vec<u32>,
    window: Window,
}

impl VideoEngine {
    pub fn new(window: Window) -> Self {
        VideoEngine {
            video_ram: vec![BACK_COLOR; 32 * 64],
            window: window,
        }
    }

    pub fn set_pixel_to_1(&mut self, vx: usize, vy: usize) -> bool {
        println!("TODO: XOR PIXEL WITH 1 @ {},{}", vx, vy);
        let displacement = vx + (vy * 64);
        let current_value = self.video_ram[displacement];
        match current_value {
            BACK_COLOR => {
                self.video_ram[displacement] = FRONT_COLOR;
                false
            }
            FRONT_COLOR => {
                self.video_ram[displacement] = BACK_COLOR;
                true
            }
            _ => {
                panic!("Color {} not supported", current_value);
            }
        }
    }
    pub fn set_pixel_to_0(&mut self, vx: usize, vy: usize) {
        println!("TODO: XOR PIXEL WITH 0 @ {},{}", vx, vy);
        let displacement = vx + (vy * 64);
        let current_value = self.video_ram[displacement];
        match current_value {
            BACK_COLOR => {
                self.video_ram[displacement] = BACK_COLOR;
            }
            FRONT_COLOR => {
                self.video_ram[displacement] = FRONT_COLOR;
            }
            _ => {
                panic!("Color {} not supported", current_value);
            }
        }
    }

    pub fn cls(&mut self) {
        for i in 0..self.video_ram.len() {
            self.video_ram[i] = BACK_COLOR
        }
    }

    pub fn wait_for_key_input(&mut self) -> u8 {
        while !self.window.is_key_down(Key::Up) {
            self.draw()
        }
        0x1
    }

    pub fn vram(&self) -> &Vec<u32> {
        &self.video_ram
    }

    pub fn draw(&mut self) {
        self.window.update_with_buffer(&self.video_ram);
    }

    pub fn is_running(&self) -> bool {
        self.window.is_open() && !self.window.is_key_down(Key::Escape)
    }
}

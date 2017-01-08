use minifb::{Key, Window};

pub const BACK_COLOR: u32 = 0x0;
pub const FRONT_COLOR: u32 = 0xFFFFFFFF;

pub struct VideoEngine {
    video_ram: Vec<u32>,
    window: Window,
}

impl VideoEngine {
    pub fn new(window: Window) -> Self {
        let mut res = VideoEngine {
            video_ram: vec![BACK_COLOR; 32 * 64],
            window: window,
        };
        res.draw();
        res
    }

    pub fn set_pixel_to_1(&mut self, vx: usize, vy: usize) -> bool {
        let (vx, vy) = Self::wrap(vx, vy);
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
        let (vx, vy) = Self::wrap(vx, vy);
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

    fn wrap(x: usize, y: usize) -> (usize, usize) {
        (x % 64, y % 32)
    }

    pub fn cls(&mut self) {
        for i in 0..self.video_ram.len() {
            self.video_ram[i] = BACK_COLOR
        }
    }

    pub fn get_current_key_input(&self) -> Option<u8> {
        let mut result = None;
        match self.window.get_keys() {
            None => {}
            Some(keys) => {
                for key in keys {
                    match self.decode(key) {
                        None => {}
                        Some(code) => result = Some(code),
                    }
                }
            }
        }
        result
    }

    pub fn wait_for_key_input(&self) -> u8 {
        let mut result = None;
        loop {
            result = self.get_current_key_input();
            match result {
                None => {},
                Some(..) => {break}
            }
        }
        result.unwrap()
    }

    fn decode(&self, key: Key) -> Option<u8> {
        match key {
            Key::NumPad1 => Some(0x1),
            Key::NumPad2 => Some(0x2),
            Key::NumPad3 => Some(0x3),
            Key::NumPad4 => Some(0xc),
            Key::Q => Some(0x4),
            Key::W => Some(0x5),
            Key::E => Some(0x6),
            Key::R => Some(0xd),
            Key::A => Some(0x7),
            Key::S => Some(0x8),
            Key::D => Some(0x9),
            Key::F => Some(0xe),
            Key::Z => Some(0xa),
            Key::X => Some(0x0),
            Key::C => Some(0xb),
            Key::V => Some(0xf),
            _ => None,
        }
    }

    pub fn vram(&self) -> &Vec<u32> {
        &self.video_ram
    }

    pub fn draw(&mut self) {
        self.window.update_with_buffer(&self.video_ram);
        self.window.update();
    }

    pub fn is_running(&self) -> bool {
        self.window.is_open() && !self.window.is_key_down(Key::Escape)
    }
}

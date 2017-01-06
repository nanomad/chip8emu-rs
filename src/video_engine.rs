const BACK_COLOR: u32  = 0x0;
const FRONT_COLOR: u32 = 0xFFFFFFFF;

pub struct VideoEngine {
    video_ram: Vec<u32>,
}

impl VideoEngine {
    pub fn new() -> Self {
        VideoEngine {
            video_ram: vec![BACK_COLOR; 32 * 64],
        }
    }

    pub fn set_pixel_to_1(&mut self, vx: usize, vy: usize) -> bool {
        println!("TODO: XOR PIXEL @ {},{}", vx, vy);
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

    pub fn buffer(&self) -> &Vec<u32> {
        &self.video_ram
    }
}

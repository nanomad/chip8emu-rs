use std::collections::HashMap;
use std::convert::{From, TryFrom};
use video_engine::VideoEngine;

#[derive(Eq, PartialEq, Debug, Clone, Copy, Hash)]
pub enum Key {
    Key0,
    Key1,
    Key2,
    Key3,
    Key4,
    Key5,
    Key6,
    Key7,
    Key8,
    Key9,
    KeyA,
    KeyB,
    KeyC,
    KeyD,
    KeyE,
    KeyF,
}

impl TryFrom<u8> for Key {
    type Err = String;
    fn try_from(val: u8) -> Result<Self, Self::Err> {
        match val {
            0 => Ok(Key::Key0),
            1 => Ok(Key::Key1),
            2 => Ok(Key::Key2),
            3 => Ok(Key::Key3),
            4 => Ok(Key::Key4),
            5 => Ok(Key::Key5),
            6 => Ok(Key::Key6),
            7 => Ok(Key::Key7),
            8 => Ok(Key::Key8),
            9 => Ok(Key::Key9),
            0xA => Ok(Key::KeyA),
            0xB => Ok(Key::KeyB),
            0xC => Ok(Key::KeyC),
            0xD => Ok(Key::KeyD),
            0xE => Ok(Key::KeyE),
            0xF => Ok(Key::KeyF),
            _ => Err(format!("Invalid key code {}", val)),
        }
    }
}


impl From<Key> for u8 {
    fn from(val: Key) -> u8 {
        match val {
            Key::Key0 => 0,
            Key::Key1 => 1,
            Key::Key2 => 2,
            Key::Key3 => 3,
            Key::Key4 => 4,
            Key::Key5 => 5,
            Key::Key6 => 6,
            Key::Key7 => 7,
            Key::Key8 => 8,
            Key::Key9 => 9,
            Key::KeyA => 0xA,
            Key::KeyB => 0xB,
            Key::KeyC => 0xC,
            Key::KeyD => 0xD,
            Key::KeyE => 0xE,
            Key::KeyF => 0xF,
        }
    }
}

pub struct Keypad {
    key_states: HashMap<Key, bool>,
}

pub struct Peripherals {
    pub keypad: Keypad,
    pub video_engine: VideoEngine,
}

impl Peripherals {
    pub fn new() -> Self {
        Peripherals {
            keypad: Keypad::new(),
            video_engine: VideoEngine::new(),
        }
    }
}


impl Keypad {
    pub fn new() -> Self {
        Keypad { key_states: HashMap::new() }
    }

    pub fn set_button_state(&mut self, key: Key, is_down: bool) {
        self.key_states.insert(key, is_down);

    }

    pub fn get_current_key_input(&self) -> Option<u8> {
        for (key, state) in &self.key_states {
            if *state {
                return Some(*key as u8);
            } else {
                continue;
            }
        }
        None
    }
}

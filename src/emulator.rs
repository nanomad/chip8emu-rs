
use minifb::{WindowOptions, Scale, Window, Key, KeyRepeat};
use std::io;
use std::io::prelude::*;
use std::io::stdin;
use std::sync::mpsc::{channel, Receiver};
use std::thread;
use std::thread::JoinHandle;
use std::time;

use chip8::Chip8;
use debugger::debugger::Debugger;
use peripherals;
use peripherals::Peripherals;

#[derive(PartialEq, Eq)]
enum Mode {
    Running,
    Debugging,
}

pub struct Emulator {
    chip8: Chip8,
    window: Window,
    peripherals: Peripherals,

    mode: Mode,
    stdin_receiver: Receiver<String>,
    _stdin_thread: JoinHandle<()>,
}

impl Emulator {
    pub fn new(rom: &Vec<u8>) -> Self {
        let window_options = WindowOptions {
            borderless: false,
            title: true,
            resize: false,
            scale: Scale::X16,
        };

        let (stdin_sender, stdin_receiver) = channel();
        let stdin_thread = thread::spawn(move || loop {
            stdin_sender.send(read_stdin()).unwrap();
        });

        Emulator {
            chip8: Chip8::new(&rom),
            window: Window::new("RUST Chip8 Emulator", 64, 32, window_options).unwrap(),
            peripherals: Peripherals::new(),

            mode: Mode::Running,
            stdin_receiver: stdin_receiver,
            _stdin_thread: stdin_thread,
        }
    }

    pub fn run(&mut self) {
        let mut debugger = Debugger::new();
        while self.window.is_open() && !self.window.is_key_down(Key::Escape) && !debugger.is_exit() {
            {
                match self.mode {
                    Mode::Running => {
                        let mut start_debugger = false;

                        self.chip8.step(&mut self.peripherals);
                        if debugger.must_break(&self.chip8) {
                            start_debugger = true;
                        }

                        if start_debugger {
                            self.mode = Mode::Debugging;
                        }
                    }
                    Mode::Debugging => {
                        print!("[0x{:2x}]> ", self.chip8.pc());
                        io::stdout().flush().expect("Could not flush stdout");
                        while debugger.manage_cli(&mut self.stdin_receiver,
                                                  &mut self.chip8,
                                                  &mut self.peripherals) {
                            self.window.update();
                        }
                        self.mode = Mode::Running
                    }
                }
            }

            self.window.update_with_buffer(self.peripherals.video_engine.vram());

            if let Mode::Running = self.mode {
                self.update_keys();
                if self.window.is_key_pressed(Key::F12, KeyRepeat::No) {
                    self.mode = Mode::Debugging;
                }
            }
            thread::sleep(time::Duration::from_millis(3));
        }
    }
    fn update_keys(&mut self) {
        self.update_key(peripherals::Key::Key0, Key::X);
        self.update_key(peripherals::Key::Key1, Key::NumPad1);
        self.update_key(peripherals::Key::Key2, Key::NumPad2);
        self.update_key(peripherals::Key::Key3, Key::NumPad3);
        self.update_key(peripherals::Key::Key4, Key::Q);
        self.update_key(peripherals::Key::Key5, Key::W);
        self.update_key(peripherals::Key::Key6, Key::E);
        self.update_key(peripherals::Key::Key7, Key::A);
        self.update_key(peripherals::Key::Key8, Key::S);
        self.update_key(peripherals::Key::Key9, Key::D);
        self.update_key(peripherals::Key::KeyA, Key::Z);
        self.update_key(peripherals::Key::KeyB, Key::C);
        self.update_key(peripherals::Key::KeyC, Key::NumPad4);
        self.update_key(peripherals::Key::KeyD, Key::R);
        self.update_key(peripherals::Key::KeyE, Key::F);
        self.update_key(peripherals::Key::KeyF, Key::V);
    }

    fn update_key(&mut self, target_key: peripherals::Key, mapped_key: Key) {
        let ref mut keypad = self.peripherals.keypad;
        keypad.set_button_state(target_key, self.window.is_key_down(mapped_key));
    }
}

fn read_stdin() -> String {
    let mut input = String::new();
    stdin().read_line(&mut input).unwrap();
    input.trim().into()
}

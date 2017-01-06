#[derive(Debug)]
pub enum Command {
    Goto { loc: usize },
    Dump,
    VideoRamDump,
    Disasm { count: usize },
    Break { loc: usize },
    Step,
    Run,
    Repeat,
    Quit,
}

impl From<String> for Command {
    fn from(text: String) -> Self {
        let tokens: Vec<&str> = text.trim().split(" ").collect();
        if !tokens.is_empty() {
            match tokens[0] {
                "goto" | "g" => {
                    Command::Goto {
                        loc: usize::from_str_radix(tokens[1], 16).expect("Cannot parse command"),
                    }
                }
                "disasm" | "d" => Command::Disasm { count: 1 },
                "vdump" | "vx" => Command::VideoRamDump,
                "dump" | "x" => Command::Dump,
                "break" | "b" => {
                    Command::Break {
                        loc: usize::from_str_radix(tokens[1], 16).expect("Cannot parse command"),
                    }
                }
                "step" | "s" | "." => Command::Step,
                "run" | "r" => Command::Run,
                "quit" | "q" => Command::Quit,
                _ => Command::Repeat,
            }
        } else {
            Command::Repeat
        }
    }
}

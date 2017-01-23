use std::convert::TryFrom;

#[derive(Debug, Copy, Clone)]
pub enum Command {
    Goto { loc: usize },
    Dump { count: usize },
    VideoRamDump,
    RegDump,
    StackDump,
    Disasm { count: usize },
    Break { loc: usize },
    Step,
    Run,
    Repeat,
    Quit,
}

impl TryFrom<String> for Command {
    type Err = String;
    fn try_from(text: String) -> Result<Self, Self::Err> {
        let tokens: Vec<&str> = text.trim().split(' ').collect();
        if !tokens.is_empty() {
            match tokens[0] {
                "goto" | "g" => {
                    Ok(Command::Goto {
                        loc: usize::from_str_radix(tokens[1], 16).expect("Cannot parse command"),
                    })
                }
                "disasm" | "d" => Ok(Command::Disasm { count: 1 }),
                "vdump" | "vx" => Ok(Command::VideoRamDump),
                "rdump" | "rx" => Ok(Command::RegDump),
                "sdump" | "sx" => Ok(Command::StackDump),
                "dump" | "x" => {
                    let count = if tokens.len() > 1 {
                        usize::from_str_radix(tokens[1], 10).unwrap_or(0)
                    } else {
                        1
                    };
                    Ok(Command::Dump { count: count })
                }
                "break" | "b" => {
                    Ok(Command::Break {
                        loc: usize::from_str_radix(tokens[1], 16).expect("Cannot parse command"),
                    })
                }
                "step" | "s" | "." => Ok(Command::Step),
                "run" | "r" => Ok(Command::Run),
                "quit" | "q" => Ok(Command::Quit),
                "" => Ok(Command::Repeat),
                _ => Err(format!("Unsupported command {}", text)),
            }
        } else {
            Ok(Command::Repeat)
        }
    }
}

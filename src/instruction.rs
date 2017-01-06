use std::convert::TryFrom;
use std::fmt;

pub enum Instruction {
    Jmp { addr: usize },
    Skeq { vr: usize, k: u8 },
    Mov { vr: usize, k: u8 },
    Add { vr: usize, k: u8 },
    Mvi { k: u16 },
    Sprite { rx: usize, ry: usize, s: usize },
    Adi { vr: usize },
    Str { vr: usize },
    Ldr { vr: usize },
}

impl TryFrom<u16> for Instruction {
    type Err = String;
    fn try_from(opcode: u16) -> Result<Self, Self::Err> {
        match opcode & 0xF000 {
            0x1000 => Ok(Instruction::Jmp { addr: (opcode as usize & 0x0FFF) - 2 }),
            0x3000 => {
                let k = (opcode & 0x00FF) as u8;
                let vr = (opcode & 0xF00) >> 8;
                Ok(Instruction::Skeq {
                    vr: (vr as usize),
                    k: k,
                })
            }
            0x6000 => {
                let k = (opcode & 0x00FF) as u8;
                let vr = (opcode & 0xF00) >> 8;
                Ok(Instruction::Mov {
                    vr: (vr as usize),
                    k: k,
                })
            }
            0x7000 => {
                let k = (opcode & 0x00FF) as u8;
                let vr = (opcode & 0xF00) >> 8;
                Ok(Instruction::Add {
                    vr: (vr as usize),
                    k: k,
                })
            }
            0xA000 => Ok(Instruction::Mvi { k: opcode & 0x0FFF }),
            0xD000 => {
                let s = (opcode & 0x000F) as usize;
                let ry = ((opcode & 0x00F0) >> 4) as usize;
                let rx = ((opcode & 0x0F00) >> 8) as usize;
                Ok(Instruction::Sprite {
                    rx: rx,
                    ry: ry,
                    s: s,
                })
            }
            0xF000 => {
                match opcode & 0xF0FF {
                    0xF01E => {
                        let reg_end = (opcode & 0x0F00) >> 8;
                        Ok(Instruction::Adi { vr: reg_end as usize })
                    },
                    0xF055 => {
                        let reg_end = (opcode & 0x0F00) >> 8;
                        Ok(Instruction::Str { vr: reg_end as usize })
                    }
                    0xF065 => {
                        let reg_end = (opcode & 0x0F00) >> 8;
                        Ok(Instruction::Ldr { vr: reg_end as usize })
                    }
                    _ => {
                        Err(format!("Opcode 0x{:x} not yet implemented (in 0xF000 branch)",
                                    opcode))
                    }
                }
            }
            _ => Err(format!("Opcode 0x{:x} not yet implemented", opcode)),
        }
    }
}

impl fmt::Debug for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Instruction::Jmp { addr } => write!(f, "jmp    0x{:x}", addr+2),
            &Instruction::Skeq { vr, k } => write!(f, "skeq   v{}, 0x{:x}", vr, k),
            &Instruction::Mov { vr, k } => write!(f, "mov    v{}, 0x{:x}", vr, k),
            &Instruction::Add { vr, k } => write!(f, "add    v{}, 0x{:x}", vr, k),
            &Instruction::Mvi { k } => write!(f, "mvi    0x{:x}", k),
            &Instruction::Sprite { rx, ry, s } => write!(f, "sprite {},{},{}", rx, ry, s),
            &Instruction::Adi { vr } => write!(f, "adi    v{}", vr),
            &Instruction::Str { vr } => write!(f, "str    v0-v{}", vr),
            &Instruction::Ldr { vr } => write!(f, "ldr    v0-v{}", vr),
        }
    }
}

use std::convert::TryFrom;
use std::fmt;

pub enum Instruction {
    Cls,
    Jmp { addr: usize },
    Jsr { addr: usize },
    Skeq { vr: usize, k: u8 },
    Mov { vr: usize, k: u8 },
    Movr {vr: usize, vy: usize },
    Shr {vr: usize},
    Add { vr: usize, k: u8 },
    Mvi { k: u16 },
    Sprite { rx: usize, ry: usize, s: usize },
    Key { vr: usize },
    Adi { vr: usize },
    Str { vr: usize },
    Ldr { vr: usize },
}

impl TryFrom<u16> for Instruction {
    type Err = String;
    fn try_from(opcode: u16) -> Result<Self, Self::Err> {
        match opcode & 0xF000 {
            0x0000 => {
                match opcode & 0x00FF {
                    0x00E0 => Ok(Instruction::Cls),
                    _ => Err(format!("Opcode 0x{:x} not yet implemented (in 0x0000 branch)",
                                opcode))
                }
            }
            0x1000 => k_op(opcode, |addr| Instruction::Jmp { addr: addr as usize }),
            0x2000 => k_op(opcode, |addr| Instruction::Jsr { addr: addr as usize }),
            0x3000 => vr_k_op(opcode, |vr, k| Instruction::Skeq { vr: vr, k: k }),
            0x6000 => vr_k_op(opcode, |vr, k| Instruction::Mov { vr: vr, k: k }),
            0x7000 => vr_k_op(opcode, |vr, k| Instruction::Add { vr: vr, k: k }),
            0x8000 => {
                match opcode & 0xF00F {
                    0x8000 => vr_vy_op(opcode, |vr, vy| Instruction::Movr {vr:vr, vy:vy}),
                    0x8006 => vr_op(opcode, |vr| Instruction::Shr {vr:vr}),
                    _ => Err(format!("Opcode 0x{:x} not yet implemented (in 0x8000 branch)",
                                     opcode))
                }
            }
            0xA000 => k_op(opcode, |k| Instruction::Mvi { k: k }),
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
                    0xF00A => vr_op(opcode, |vr| Instruction::Key { vr: vr }),
                    0xF01E => vr_op(opcode, |vr| Instruction::Adi { vr: vr }),
                    0xF055 => vr_op(opcode, |vr| Instruction::Str { vr: vr }),
                    0xF065 => vr_op(opcode, |vr| Instruction::Ldr { vr: vr }),
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

fn vr_op<F>(opcode: u16, f: F) -> Result<Instruction, String>
    where F: FnOnce(usize) -> Instruction
{
    let vr = ((opcode & 0x0F00) >> 8) as usize;
    Ok(f(vr))
}

fn vr_k_op<F>(opcode: u16, f: F) -> Result<Instruction, String>
    where F: FnOnce(usize, u8) -> Instruction
{
    let k = (opcode & 0x00FF) as u8;
    let vr = ((opcode & 0xF00) >> 8) as usize;
    Ok(f(vr, k))
}

fn vr_vy_op<F>(opcode: u16, f: F) -> Result<Instruction, String>
    where F: FnOnce(usize, usize) -> Instruction
{
    let vy = ((opcode & 0x00F0) >> 4) as usize;
    let vr = ((opcode & 0xF00) >> 8) as usize;
    Ok(f(vr, vy))
}

fn k_op<F>(opcode: u16, f: F) -> Result<Instruction, String>
    where F: FnOnce(u16) -> Instruction
{
    let k = (opcode & 0x0FFF) as u16;
    Ok(f(k))
}

impl fmt::Debug for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Instruction::Cls => write!(f, "cls"),
            &Instruction::Jmp { addr } => write!(f, "jmp    0x{:x}", addr),
            &Instruction::Jsr { addr } => write!(f, "jsr    0x{:x}", addr),
            &Instruction::Skeq { vr, k } => write!(f, "skeq   v{}, 0x{:x}", vr, k),
            &Instruction::Mov { vr, k } => write!(f, "mov    v{}, 0x{:x}", vr, k),
            &Instruction::Movr { vr, vy } => write!(f, "mov    v{}, v{}", vr, vy),
            &Instruction::Shr { vr } => write!(f, "shr    v{}", vr),
            &Instruction::Add { vr, k } => write!(f, "add    v{}, 0x{:x}", vr, k),
            &Instruction::Mvi { k } => write!(f, "mvi    0x{:x}", k),
            &Instruction::Sprite { rx, ry, s } => write!(f, "sprite {},{},{}", rx, ry, s),
            &Instruction::Key { vr } => write!(f, "key    v{}", vr),
            &Instruction::Adi { vr } => write!(f, "adi    v{}", vr),
            &Instruction::Str { vr } => write!(f, "str    v0-v{}", vr),
            &Instruction::Ldr { vr } => write!(f, "ldr    v0-v{}", vr),
        }
    }
}

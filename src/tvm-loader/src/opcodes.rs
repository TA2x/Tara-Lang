#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Opcode {
    PushI64 = 0x01,
    Add = 0x02,
    Syscall = 0x03,
    Halt = 0xFF,
}

impl Opcode {
    pub fn from_u8(b : u8) -> Option<Self> {
        match b {
            0x01 => Some(Self::PushI64),
            0x02 => Some(Self::Add),
            0x03 => Some(Self::Syscall),
            0xFF => Some(Self::Halt),
            _ => None,
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyscallId {
    PrintI64 = 0x01,
}

impl SyscallId {
    pub fn from_u8(b: u8) -> Option<Self> {
        match b {
            0x01 => Some(Self::PrintI64),
            _ => None,
        }
    }
}

pub const MAGIC: [u8; 4] = *b"TVM0";
pub const VERSION_V0: u16 = 1;
pub const HEADER_LEN: usize = 12; // 4 (magic) +  2 (version) + 2 (reserved) + 4 (code_len)

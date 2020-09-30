pub const DATAEOF: u16 = 0x0080;
pub const DATA: u16 = 0x0000;
pub const ACK: u16 = 0x8001;
pub const FILE: u16 = 0x3;

pub const MAXPACKETLEN: usize = 8;

pub struct Packet {
    pub flag: u16,
    pub len: u16,
    pub msgno: u32,
    pub offset: u32,
}

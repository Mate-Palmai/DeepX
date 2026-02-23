#[repr(u64)]
pub enum TunnelID {
    Log = 0,        // rdi: ptr, rsi: len
    VfsOpen = 1,    // rdi: path_ptr, rsi: path_len
    VfsRead = 2,    // rdi: handle, rsi: buffer_ptr, rdx: len
    VfsList = 3,    // rdi: buffer_ptr, rsi: max_len
    Exit = 60,      // rdi: exit_code
    Unknown,
}

impl From<u64> for TunnelID {
    fn from(id: u64) -> Self {
        match id {
            0 => TunnelID::Log,
            1 => TunnelID::VfsOpen,
            2 => TunnelID::VfsRead,
            3 => TunnelID::VfsList,
            60 => TunnelID::Exit,
            _ => TunnelID::Unknown,
        }
    }
}
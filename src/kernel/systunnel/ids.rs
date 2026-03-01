#[repr(u64)]
#[derive(Debug, PartialEq, Eq)]
pub enum TunnelID {
    Ok = 0,        
    Exit = 1,      
    Log = 2,
    Execute = 9,

    VfsExists = 10,    
    VfsOpen = 11,
    VfsRead = 12,

    Unknown,
}

impl From<u64> for TunnelID {
    fn from(id: u64) -> Self {
        match id {
            0 => TunnelID::Ok,    
            1 => TunnelID::Exit,
            2 => TunnelID::Log,
            9 => TunnelID::Execute,
            10 => TunnelID::VfsExists,
            11 => TunnelID::VfsOpen,
            12 => TunnelID::VfsRead,    
            
            _ => TunnelID::Unknown,
        }
    }
}
#[repr(u64)]
#[derive(Debug)]
pub enum TunnelError {
    Success = 0,
    UnknownError = 1,
    InvalidPointer = 2,
    AccessViolation = 3,
    NullPointer = 4,
    FileNotFound = 5,
    NotImplemented = 404,
}

impl From<TunnelError> for u64 {
    fn from(err: TunnelError) -> Self {
        err as u64
    }
}
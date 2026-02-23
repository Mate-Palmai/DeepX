use crate::kernel::systunnel::errors::TunnelError;

pub struct Validator;

impl Validator {
    pub fn check_buffer(ptr: u64, len: u64) -> Result<(), TunnelError> {
        let end_addr = ptr.checked_add(len).ok_or(TunnelError::InvalidPointer)?;

        // x86_64 Canonical Address limit (ez alatt van a User space)
        // A legtöbb kernelnél 0x0000_7FFF_FFFF_FFFF a határ.
        const USER_SPACE_LIMIT: u64 = 0x0000_7FFF_FFFF_FFFF;

        // DEBUG: Ha a cím kívül esik a várt tartományon
        if ptr > USER_SPACE_LIMIT {
            unsafe {
                crate::kernel::console::LOGGER.error(&alloc::format!(
                    "VALIDATE FAIL: Access Violation! Ptr: 0x{:016x}, Limit: 0x{:016x}",
                    ptr, USER_SPACE_LIMIT
                ));
            }
            return Err(TunnelError::AccessViolation);
        }

        if ptr == 0 {
            unsafe { crate::kernel::console::LOGGER.error("VALIDATE FAIL: Null Pointer!"); }
            return Err(TunnelError::NullPointer);
        }

        Ok(())
    }
}
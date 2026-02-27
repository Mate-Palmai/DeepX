/*
 * DeepX Project
 * Copyright (C) 2024-2026 - Máté Pálmai
 *
 * File: /src/kernel/systunnel/validate.rs
 * Description: Validates user-space memory access in system tunnels.
 */

use crate::kernel::systunnel::errors::TunnelError;

pub struct Validator;

impl Validator {
    pub fn check_buffer(ptr: u64, len: u64) -> Result<(), TunnelError> {
        let end_addr = ptr.checked_add(len).ok_or(TunnelError::InvalidPointer)?;

        const USER_SPACE_LIMIT: u64 = 0x0000_7FFF_FFFF_FFFF;

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
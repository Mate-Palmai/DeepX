/*
 * DeepX Project
 * Copyright (C) 2024-2026 - Máté Pálmai
 *
 * File: /src/kernel/systunnel/dispatch.rs
 * Description: System tunnel dispatch logic for handling user-space calls.
 */

use crate::kernel::systunnel::validate::Validator;
use crate::kernel::systunnel::ids::TunnelID;
use crate::kernel::systunnel::frame::SystunnelFrame;

use alloc::format;

#[no_mangle] 
pub extern "C" fn dispatch(frame: &mut SystunnelFrame) {
    let id = TunnelID::from(frame.rax);

    frame.rax = match id {
        TunnelID::Log => {
            let ptr = frame.rdi;
            let len = frame.rsi;

            match Validator::check_buffer(ptr, len) {
                Ok(_) => {
                    unsafe {
                        let s = core::slice::from_raw_parts(ptr as *const u8, len as usize);
                        if let Ok(msg) = core::str::from_utf8(s) {
                            
                            crate::kernel::console::LOGGER.tunnel(msg);
                            0
                        } else {
                            crate::kernel::console::LOGGER.error("SYSTUNNEL: UTF-8 Decode Error");
                            1
                        }
                    }
                },
                Err(e) => e as u64, 
            }
        },
        TunnelID::Exit => {
            let status = frame.rdi;
            crate::kernel::console::LOGGER.warn("OS Discovery Exit Triggered");
            crate::prepare_recovery_space_and_jump();
            0 
        },

        
        _ => {
            unsafe {
                crate::kernel::console::LOGGER.warn("SYSTUNNEL: Unknown Call ID requested.");
            }
            404
        },
    };
}
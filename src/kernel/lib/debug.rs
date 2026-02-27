/*
 * DeepX Project
 * Copyright (C) 2024-2026 - Máté Pálmai
 *
 * File: /src/kernel/lib/debug.rs
 * Description: Debugging utilities for kernel.
 */

use crate::kernel::drivers::input;

pub static mut IS_WAITING_FOR_INPUT: bool = false;

pub fn wait_for_input() {
    unsafe { IS_WAITING_FOR_INPUT = true; }

    while input::pop_key().is_some() {}

    loop {
        if input::pop_key().is_some() {
            break;
        }
        unsafe { core::arch::asm!("pause"); }
    }

    unsafe { IS_WAITING_FOR_INPUT = false; }
}
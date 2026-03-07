/*
 * DeepX Project
 * Copyright (C) 2024-2026 - Máté Pálmai
 *
 * File: /src/kernel/console/safe_console.rs
 * Description: Safe console implementation for kernel logging and display.
 */


use crate::kernel::console::CONSOLE;
use crate::kernel::console::ring_buffer::LOG_BUFFER;
use crate::kernel::console::SAFE_CONSOLE;
use crate::kernel::console::DisplayMode;
use crate::kernel::console::display_manager::CURRENT_MODE;


pub struct SafeConsole;

impl SafeConsole {
    pub const fn new() -> Self {
        Self
    }

    pub fn write_str(&self, text: &str) {
        if let Some(mut log) = LOG_BUFFER.try_lock() {
            log.push_str(text);
        }
    }

    pub fn render_buffer(&self) {
        if unsafe { CURRENT_MODE != DisplayMode::SafeConsole } { return; }

        let mut console_lock = CONSOLE.lock(); 
        if let Some(console) = console_lock.as_mut() {
            let log = LOG_BUFFER.lock();
            console.render_buffer(&log);
        }
    }

    pub fn clear(&self) {
        if let Some(mut console_lock) = CONSOLE.try_lock() {
            if let Some(console) = console_lock.as_mut() {
                console.clear();
            }
        }
    }

    pub fn render_safely(&self) {
        if unsafe { CURRENT_MODE != DisplayMode::SafeConsole } { return; }

        let mut console_lock = CONSOLE.lock(); 
        if let Some(console) = console_lock.as_mut() {
            let log = LOG_BUFFER.lock();
            console.render_buffer(&log);
        }
    }
}

pub fn safe_console_task_entry() {
    unsafe { core::arch::asm!("sti"); }
    loop {
        crate::kernel::console::display_manager::process_keyboard_queue();
        
        if unsafe { CURRENT_MODE == DisplayMode::SafeConsole } {
            if let Some(mut console_lock) = CONSOLE.try_lock() {
                if let Some(console) = console_lock.as_mut() {
                    if let Some(log) = LOG_BUFFER.try_lock() {
                        console.render_buffer(&log);
                    }
                }
            }
        }

        crate::kernel::process::Scheduler::yield_now();
    }
}
/* /src/kernel/console/safe_console.rs */
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

    // &self-et használunk, hogy a statikus példányon keresztül hívható legyen
    pub fn write_str(&self, text: &str) {
        if let Some(mut log) = LOG_BUFFER.try_lock() {
            log.push_str(text);
        }
    }

    pub fn render_buffer(&self) {
        if unsafe { CURRENT_MODE != DisplayMode::SafeConsole } { return; }

        if let Some(mut console_lock) = CONSOLE.try_lock() {
            if let Some(console) = console_lock.as_mut() {
                if let Some(log) = LOG_BUFFER.try_lock() {
                    console.render_buffer(&log);
                }
            }
        }
    }

    pub fn clear(&self) {
        if let Some(mut console_lock) = CONSOLE.try_lock() {
            if let Some(console) = console_lock.as_mut() {
                console.clear();
            }
        }
    }
}

use crate::kernel::console::display_manager; // Használd a modult közvetlenül

// src/kernel/console/safe_console.rs
pub fn safe_console_task_entry() {
    unsafe { core::arch::asm!("sti"); }
    loop {
        crate::kernel::console::display_manager::process_keyboard_queue();
        

        // CSAK try_lock! Ha foglalt a konzol (mert épp a LOGGER ír rá), 
        // akkor ebben a körben nem rajzolunk, és kész.
        let is_safe = unsafe { crate::kernel::console::display_manager::CURRENT_MODE == DisplayMode::SafeConsole };
        if is_safe {
            if let Some(mut console_lock) = crate::kernel::console::CONSOLE.try_lock() {
                if let Some(console) = console_lock.as_mut() {
                    
                    
                    
                        // Itt rajzolunk
                        if let Some(log) = crate::kernel::console::ring_buffer::LOG_BUFFER.try_lock() {
                            console.render_buffer(&log);
                        }
                        console.info_panel();
                    }
                }
        }

       

        // Fontos: pihenjen a task, hogy a többiek hozzáférjenek a lakathoz!
        for _ in 0..50_000 { unsafe { core::arch::asm!("pause"); } }
    }
}
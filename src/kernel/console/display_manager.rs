/*
 * DeepX Project
 * Copyright (C) 2024-2026 - Máté Pálmai
 *
 * File: /src/kernel/console/display_manager.rs
 * Description: Display manager for handling different console modes and keyboard input in the kernel.
 */

use crate::kernel::drivers::keyboard::Keyboard;
use crate::kernel::drivers::input;
use crate::kernel::console::DisplayMode;

pub static mut CURRENT_MODE: DisplayMode = DisplayMode::SafeConsole;

struct ModifierState {
    ctrl: bool,
    shift: bool,
}

impl DisplayMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            DisplayMode::RecoveryConsole => "RECOVERY",
            DisplayMode::SafeConsole => "LOGS",
            #[cfg(feature = "dev")]
            DisplayMode::KernelShell => "SHELL",
        }
    }
}

static mut MODIFIERS: ModifierState = ModifierState {
    ctrl: false,
    shift: false,
};

pub static mut NEEDS_FULL_REDRAW: bool = false;


pub fn set_display_mode(mode: DisplayMode) {
    unsafe {
        if CURRENT_MODE == mode { return; }
        
        if let Some(mut console_lock) = crate::kernel::console::CONSOLE.try_lock() {
            if let Some(console) = console_lock.as_mut() {
                console.clear();
            }
        }

        CURRENT_MODE = mode;
        NEEDS_FULL_REDRAW = true;
    }

}
pub fn process_keyboard_queue() {
    if let Some(mut queue) = crate::kernel::drivers::keyboard::KEY_QUEUE.try_lock() {
        while let Some(scancode) = queue.pop_front() {
            handle_scancode(scancode);
        }
    }
}

fn handle_scancode(scancode: u8) {
    unsafe {
        match scancode {
            0x1D => { MODIFIERS.ctrl = true; }   // Left Ctrl press
            0x9D => { MODIFIERS.ctrl = false; }  // Left Ctrl release
            0x2A | 0x36 => { MODIFIERS.shift = true; } // Shift press
            0xAA | 0xB6 => { MODIFIERS.shift = false; } // Shift release
            _ => {}
        }

        if scancode < 0x80 && MODIFIERS.ctrl && MODIFIERS.shift {
            let handled = match scancode {
                0x3B => { set_display_mode(DisplayMode::RecoveryConsole); true }    // F1
                0x3C => { set_display_mode(DisplayMode::SafeConsole); true }        // F2
                #[cfg(feature = "dev")]
                0x3D => { set_display_mode(DisplayMode::KernelShell); true }        // F3
                _ => false,
            };
            
            if handled { 
                return; 
            }
        }

        #[cfg(feature = "dev")]
        if scancode < 0x80 && CURRENT_MODE == DisplayMode::KernelShell {
            if let Some(c) = Keyboard::scancode_to_char(scancode) {
                input::push_key(c);
            }
        }
    }
}
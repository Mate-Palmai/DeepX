/*
 * DeepX OS Project
 * Copyright (C) 2024-2026 - Máté Pálmai
 *
 * File: /src/kernel/console/display_manager.rs
 */

use crate::kernel::drivers::keyboard::Keyboard;
use crate::kernel::drivers::input;
use crate::kernel::console::DisplayMode;

// A mód tárolása atomi módon biztonságosabb, de maradunk a statikus mut-nál, 
// ha az egész rendszered erre épül.
pub static mut CURRENT_MODE: DisplayMode = DisplayMode::SafeConsole;

// Modifikátorok állapotának tárolása
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

// /src/kernel/console/display_manager.rs

pub fn set_display_mode(mode: DisplayMode) {
    unsafe {
        if CURRENT_MODE == mode { return; }
        
        // 1. Megpróbáljuk megszerezni a konzolt a törléshez
        if let Some(mut console_lock) = crate::kernel::console::CONSOLE.try_lock() {
            if let Some(console) = console_lock.as_mut() {
                console.clear(); // AZONNAL töröljük a régit
            }
        }

        CURRENT_MODE = mode;
        NEEDS_FULL_REDRAW = true;
    }

}
/// A billentyűzet sor feldolgozása
pub fn process_keyboard_queue() {
    // try_lock-ot használunk, hogy ne akasszuk meg a kernelt, ha az IRQ épp írja a queue-t
    if let Some(mut queue) = crate::kernel::drivers::keyboard::KEY_QUEUE.try_lock() {
        while let Some(scancode) = queue.pop_front() {
            handle_scancode(scancode);
        }
    }
}

/// Egyetlen scancode feldolgozása és módváltás kezelése
fn handle_scancode(scancode: u8) {
    unsafe {
        // 1. Modifiers (Ctrl, Shift) frissítése
        match scancode {
            0x1D => { MODIFIERS.ctrl = true; }   // Left Ctrl press
            0x9D => { MODIFIERS.ctrl = false; }  // Left Ctrl release
            0x2A | 0x36 => { MODIFIERS.shift = true; } // Shift press
            0xAA | 0xB6 => { MODIFIERS.shift = false; } // Shift release
            _ => {}
        }

        // 2. Módváltás detektálása (Ctrl + Shift + F-billentyűk)
        // Csak lenyomáskor (scancode < 0x80)
        if scancode < 0x80 && MODIFIERS.ctrl && MODIFIERS.shift {
            let handled = match scancode {
                0x3B => { set_display_mode(DisplayMode::RecoveryConsole); true }         // F1
                0x3C => { set_display_mode(DisplayMode::SafeConsole); true } // F2
                #[cfg(feature = "dev")]
                0x3D => { set_display_mode(DisplayMode::KernelShell); true } // F3
                _ => false,
            };
            
            if handled { 
                // Ha sikeres módváltás történt, ne küldjük tovább a karaktert a shellnek
                return; 
            }
        }

        // 3. Karakter továbbítása
        // Csak lenyomáskor, és ha Shell módban vagyunk
        #[cfg(feature = "dev")]
        if scancode < 0x80 && CURRENT_MODE == DisplayMode::KernelShell {
            if let Some(c) = Keyboard::scancode_to_char(scancode) {
                input::push_key(c);
            }
        }
    }
}
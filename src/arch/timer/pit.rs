/*
 * DeepX OS Project
 * Copyright (C) 2024-2026 - Máté Pálmai
 *
 * File: /src/arch/timer/pit.rs
 * Description: PIT (Programmable Interval Timer) for calibration and legacy timing.
 */

use core::sync::atomic::{AtomicU32, Ordering};
use alloc::format;


const PIT_CHANNEL_0: u16 = 0x40;
const PIT_COMMAND: u16 = 0x43;
const PIT_BASE_FREQ: u32 = 1193182;

static PIT_FREQ: AtomicU32 = AtomicU32::new(0);

/// Inicializálja a PIT-et folyamatos megszakítás küldésére (IRQ0)
pub fn init(freq: u32) {
    let divisor = (PIT_BASE_FREQ / freq) as u16;
    PIT_FREQ.store(freq, Ordering::SeqCst);

    unsafe {
        // Mode 3: Square Wave Generator
        // Binary counter, Mode 3, Write LSB then MSB, Channel 0
        core::arch::asm!("out 0x43, al", in("al") 0x36u8);
        core::arch::asm!("out 0x40, al", in("al") (divisor & 0xFF) as u8);
        core::arch::asm!("out 0x40, al", in("al") ((divisor >> 8) & 0xFF) as u8);
    }

    print_ok();
}

/// PIT beállítása "One-shot" módba egy adott milliszekundumra (kalibrációhoz)
/// Ez leállítja a folyamatos IRQ0 ketyegést!
pub fn prepare_sleep(ms: u32) {
    // Kiszámoljuk a tick-eket. Maximum ~55ms-ig pontos 16-biten.
    let count = (PIT_BASE_FREQ / 1000) * ms;
    let count = if count > 0xFFFF { 0xFFFF } else { count as u16 };

    unsafe {
        // Mode 0: Interrupt on Terminal Count
        core::arch::asm!("out 0x43, al", in("al") 0x30u8);
        core::arch::asm!("out 0x40, al", in("al") (count & 0xFF) as u8);
        core::arch::asm!("out 0x40, al", in("al") ((count >> 8) & 0xFF) as u8);
    }
}

/// Várunk, amíg a PIT le nem ketyeg a prepare_sleep-ben megadott értékre
pub fn wait_sleep() {
    unsafe {
        loop {
            // Read-back command a Channel 0-ra
            core::arch::asm!("out 0x43, al", in("al") 0xE2u8);
            let status: u8;
            core::arch::asm!("in al, 0x40", out("al") status);
            
            // A 7-es bit (Output Pin State) 1-re vált, ha lejárt a számláló
            if (status & 0x80) != 0 {
                break;
            }
        }
    }
}

pub fn get_freq() -> u32 {
    PIT_FREQ.load(Ordering::SeqCst)
}

fn print_ok() {
    // Betöltjük az elmentett frekvenciát
    let freq = get_freq();

    unsafe {
            if freq == 0 {
                crate::kernel::console::LOGGER.warn("PIT Timer status: initialized (unknown frequency)");
            } else {
                crate::kernel::console::LOGGER.ok(&format!("PIT Timer initialized at ^&f{} Hz", freq));
            }
        
    }
}
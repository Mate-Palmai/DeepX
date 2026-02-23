// src/kernel/lib/debug.rs

use crate::kernel::drivers::input;

pub static mut IS_WAITING_FOR_INPUT: bool = false;

pub fn wait_for_input() {
    unsafe { IS_WAITING_FOR_INPUT = true; }

    // Megvárjuk, amíg ürül a sor (hogy ne egy korábbi gombnyomás oldja fel)
    while input::pop_key().is_some() {}

    // Aktív várakozás
    loop {
        if input::pop_key().is_some() {
            break;
        }
        // Megengedjük a CPU-nak, hogy pihenjen egy kicsit, de a megszakítások (IRQ) fussanak
        unsafe { core::arch::asm!("pause"); }
    }

    unsafe { IS_WAITING_FOR_INPUT = false; }
}
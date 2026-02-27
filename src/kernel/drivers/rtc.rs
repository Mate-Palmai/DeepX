/*
 * DeepX Project
 * Copyright (C) 2024-2026 - Máté Pálmai
 *
 * File: /src/kernel/drivers/rtc.rs
 * Description: Read CMOS/BIOS RTC for early boot.
 */

use core::arch::asm;

// CMOS ports
const CMOS_ADDR: u16 = 0x70;
const CMOS_DATA: u16 = 0x71;

fn read_cmos(register: u8) -> u8 {
    unsafe {
        asm!(
            "out 0x70, al",
            "in al, 0x71",
            in("al") register,
            options(nomem, nostack, preserves_flags)
        );
        let mut value: u8;
        asm!(
            "in al, 0x71",
            out("al") value,
            options(nomem, nostack, preserves_flags)
        );
        value
    }
}

fn bcd_to_bin(value: u8) -> u8 {
    ((value & 0xF0) >> 4) * 10 + (value & 0x0F)
}

/// Returns (hour, minute, second)
pub fn read_rtc_time() -> (u8, u8, u8) {
    // 0x0 = seconds, 0x2 = minutes, 0x4 = hours
    let sec = bcd_to_bin(read_cmos(0x0));
    let min = bcd_to_bin(read_cmos(0x2));
    let hour = bcd_to_bin(read_cmos(0x4));

    (hour, min, sec)
}

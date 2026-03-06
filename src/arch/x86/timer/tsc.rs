/*
 * DeepX Project
 * Copyright (C) 2024-2026 - Máté Pálmai
 *
 * File: /src/arch/timer/tsc.rs
 * Description: TSC (Time Stamp Counter) 
 */


use core::sync::atomic::{AtomicU64, Ordering};
use crate::arch::x86::idt::get_timer_ticks;
use alloc::format;

static TSC_TICKS_PER_SEC: AtomicU64 = AtomicU64::new(0);
static BOOT_TSC: AtomicU64 = AtomicU64::new(0);

pub fn read_tsc() -> u64 {
    unsafe { core::arch::x86_64::_rdtsc() }
}

pub fn calibrate_tsc() {
    unsafe { crate::kernel::console::LOGGER.wait("TSC: Calibration started..."); }
    
    BOOT_TSC.store(read_tsc(), Ordering::SeqCst);

    let start_tick = get_timer_ticks();
    let mut timeout = 0;
    while get_timer_ticks() == start_tick {
        core::hint::spin_loop();
        timeout += 1;
        if timeout > 100_000_000 {
            unsafe { crate::kernel::console::LOGGER.warn("TSC: Calibration timeout! Using fallback ^&f2GHz."); }
            TSC_TICKS_PER_SEC.store(2_000_000_000, Ordering::SeqCst);
            return; 
        }
    }

    let tsc_start = read_tsc();
    let pit_start = get_timer_ticks();
    
    let wait_ticks = 100; 

    while get_timer_ticks() < pit_start + wait_ticks {
        core::hint::spin_loop();
    }

    let tsc_end = read_tsc();
    let tsc_elapsed = tsc_end - tsc_start;

    let pit_real_hz = crate::arch::x86::timer::pit::get_freq() as u64;
    
    if pit_real_hz > 0 {
        let ticks_per_sec = (tsc_elapsed * pit_real_hz) / wait_ticks;
        TSC_TICKS_PER_SEC.store(ticks_per_sec, Ordering::SeqCst);

        unsafe {
            let mhz = ticks_per_sec / 1_000_000;
            crate::kernel::console::LOGGER.ok(&format!(
                "TSC: Calibrated! Freq: ^&f{} MHz ^&7(Elapsed TSC: ^&f{}, ^&7PIT: ^&f{} Hz^&7)", 
                mhz, tsc_elapsed, pit_real_hz
            ));

        }
    } else {
        TSC_TICKS_PER_SEC.store(tsc_elapsed, Ordering::SeqCst);
    }
}

pub fn get_uptime() -> (u64, u64) {
    let freq = TSC_TICKS_PER_SEC.load(Ordering::SeqCst);
    if freq == 0 { return (0, 0); }

    let total_tsc = read_tsc().saturating_sub(BOOT_TSC.load(Ordering::SeqCst));
    
    let sec = total_tsc / freq;
    let remainder = total_tsc % freq;

    let frac = (remainder * 10_000) / freq;

    (sec, frac)
}

pub fn get_tsc_frequency() -> u64 {
    TSC_TICKS_PER_SEC.load(Ordering::SeqCst)
}
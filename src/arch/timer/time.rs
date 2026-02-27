/*
 * DeepX Project
 * Copyright (C) 2024-2026 - Máté Pálmai
 *
 * File: /src/arch/timer/time.rs
 * Description: Global system time tracking and sleep functions.
 */

use core::sync::atomic::{AtomicU64, Ordering};

static TICKS: AtomicU64 = AtomicU64::new(0);

pub fn tick() {
    TICKS.fetch_add(1, Ordering::SeqCst);
}

pub fn get_ticks() -> u64 {
    TICKS.load(Ordering::SeqCst)
}

/// 100Hz (1 tick = 10ms).
pub fn get_uptime_ms() -> u64 {
    TICKS.load(Ordering::SeqCst) * 10
}

pub fn sleep_ms(ms: u64) {
    let start_ticks = get_ticks();
    let ticks_to_wait = ms / 10;
    
    while get_ticks() < start_ticks + ticks_to_wait {
        core::hint::spin_loop();
    }
}
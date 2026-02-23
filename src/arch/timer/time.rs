/*
 * DeepX OS Project
 * Copyright (C) 2024-2026 - Máté Pálmai
 *
 * File: /src/arch/timer/time.rs
 * Description: Global system time tracking and sleep functions.
 */

use core::sync::atomic::{AtomicU64, Ordering};

// A rendszer indulása óta eltelt tickek száma
static TICKS: AtomicU64 = AtomicU64::new(0);

/// Ezt hívja meg az IDT timer_interrupt_handler minden egyes ketyegésnél
pub fn tick() {
    TICKS.fetch_add(1, Ordering::SeqCst);
}

/// Visszaadja az aktuális tickek számát
pub fn get_ticks() -> u64 {
    TICKS.load(Ordering::SeqCst)
}

/// Visszaadja az uptime-ot milliszekundumban. 
/// Feltételezzük a 100Hz-es frekvenciát (1 tick = 10ms).
pub fn get_uptime_ms() -> u64 {
    TICKS.load(Ordering::SeqCst) * 10
}

/// Szoftveres várakozás (nem blokkolja a CPU-t, ha már van Scheduler)
/// Addig vár, amíg el nem telik a megadott milliszekundum.
pub fn sleep_ms(ms: u64) {
    let start_ticks = get_ticks();
    let ticks_to_wait = ms / 10; // 100Hz esetén
    
    while get_ticks() < start_ticks + ticks_to_wait {
        // Itt később hívhatunk egy 'hlt' utasítást vagy 
        // egy scheduler_yield()-et, hogy ne pörögjön feleslegesen a CPU
        core::hint::spin_loop();
    }
}
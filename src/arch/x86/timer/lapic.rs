/*
 * DeepX Project
 * Copyright (C) 2024-2026 - Máté Pálmai
 *
 * File: /src/arch/timer/lapic.rs
 * Description: LAPIC Timer initialization and calibration using PIT.
 */

use alloc::format;
use crate::arch::x86::apic;
use crate::arch::x86::timer::pit;
use core::ptr::{addr_of, addr_of_mut};

static mut TICKS_PER_10MS: u32 = 0;

pub unsafe fn init() {
    if !crate::arch::x86::apic::has_apic() {
        crate::kernel::console::LOGGER.warn("LAPIC Timer: APIC not found, skipping.");
        return;
    }

    let lapic_ptr = apic::get_lapic_base() as *mut u32;

    lapic_ptr.add(0x320 / 4).write_volatile(0x10000); 

    lapic_ptr.add(0x3E0 / 4).write_volatile(0x03);

    pit::prepare_sleep(10);

    lapic_ptr.add(0x380 / 4).write_volatile(0xFFFFFFFF);

    pit::wait_sleep();

    lapic_ptr.add(0x320 / 4).write_volatile(0x10000);

    let current_count = lapic_ptr.add(0x390 / 4).read_volatile();
    let ticks_in_10ms = 0xFFFFFFFF - current_count;

    core::ptr::write_volatile(addr_of_mut!(TICKS_PER_10MS), ticks_in_10ms);

    lapic_ptr.add(0x320 / 4).write_volatile(32 | 0x20000);
    
    lapic_ptr.add(0x3E0 / 4).write_volatile(0x03);
    
    lapic_ptr.add(0x380 / 4).write_volatile(ticks_in_10ms);

    print_ok();
}

pub unsafe fn init_ap() {
    let lapic_ptr = crate::arch::x86::apic::get_lapic_base() as *mut u32;
    let ticks = core::ptr::read_volatile(addr_of!(TICKS_PER_10MS));
    
    if ticks == 0 { 
        return; 
    }

    lapic_ptr.add(0x320 / 4).write_volatile(32 | 0x20000); // Vector 32, Periodic
    lapic_ptr.add(0x3E0 / 4).write_volatile(0x03);        // Divider 16
    lapic_ptr.add(0x380 / 4).write_volatile(ticks);
}

fn print_ok() {
    use crate::arch::x86::timer::pit::get_freq;
    let freq = get_freq();



    unsafe {
            if freq == 0 {
                crate::kernel::console::LOGGER.warn("LAPIC Timer status: initialized (unknown frequency)");
            } else {
                crate::kernel::console::LOGGER.ok(&format!("LAPIC Timer calibrated via PIT and initialized at ^&f{} Hz", freq));
            }
        
    }
}
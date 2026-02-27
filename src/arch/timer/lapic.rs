/*
 * DeepX Project
 * Copyright (C) 2024-2026 - Máté Pálmai
 *
 * File: /src/arch/timer/lapic.rs
 * Description: LAPIC Timer initialization and calibration using PIT.
 */

 use alloc::format;
use crate::arch::apic;
use crate::arch::timer::pit;


pub unsafe fn init() {
    if !crate::arch::apic::has_apic() {
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

    lapic_ptr.add(0x320 / 4).write_volatile(32 | 0x20000);
    
    lapic_ptr.add(0x3E0 / 4).write_volatile(0x03);
    
    lapic_ptr.add(0x380 / 4).write_volatile(ticks_in_10ms);

    print_ok();
}

fn print_ok() {
    use crate::arch::timer::pit::get_freq;
    let freq = get_freq();



    unsafe {
            if freq == 0 {
                crate::kernel::console::LOGGER.warn("LAPIC Timer status: initialized (unknown frequency)");
            } else {
                crate::kernel::console::LOGGER.ok(&format!("LAPIC Timer calibrated via PIT and initialized at ^&f{} Hz", freq));
            }
        
    }
}
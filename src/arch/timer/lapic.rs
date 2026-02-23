/*
 * DeepX OS Project
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

    // 1. LAPIC Timer leállítása és maszkolása a kalibráció idejére
    // LVT Timer Register (Offset 0x320)
    lapic_ptr.add(0x320 / 4).write_volatile(0x10000); 

    // 2. Divider beállítása 16-ra (0x03 érték)
    // Divide Configuration Register (Offset 0x3E0)
    lapic_ptr.add(0x3E0 / 4).write_volatile(0x03);

    // 3. Kalibráció a PIT segítségével
    // Beállítjuk a PIT-et 10ms-re
    pit::prepare_sleep(10);

    // Initial Count Register (Offset 0x380) - elindítjuk a maximumról
    lapic_ptr.add(0x380 / 4).write_volatile(0xFFFFFFFF);

    // Várunk a PIT-re (pontosan 10ms-t)
    pit::wait_sleep();

    // Megállítjuk a LAPIC számlálót
    lapic_ptr.add(0x320 / 4).write_volatile(0x10000);

    // Megnézzük, hány tick telt el a LAPIC-ban 10ms alatt
    // Current Count Register (Offset 0x390)
    let current_count = lapic_ptr.add(0x390 / 4).read_volatile();
    let ticks_in_10ms = 0xFFFFFFFF - current_count;

    // 4. LAPIC Timer élesítése
    // Beállítjuk: Periodikus mód (bit 17), 32-es vektor, és feloldjuk a maszkot
    lapic_ptr.add(0x320 / 4).write_volatile(32 | 0x20000);
    
    // Újra beállítjuk a divider-t (16)
    lapic_ptr.add(0x3E0 / 4).write_volatile(0x03);
    
    // Beállítjuk az Initial Count-ot (ha 100Hz-es schedulert akarunk, akkor 10ms-enként ketyegjen)
    lapic_ptr.add(0x380 / 4).write_volatile(ticks_in_10ms);

    print_ok();
}

fn print_ok() {
    // Betöltjük az elmentett frekvenciát
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
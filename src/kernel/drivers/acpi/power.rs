use crate::kernel::acpi::tables::{PM1A_CNT_BLK, PM1B_CNT_BLK, SMI_CMD, ACPI_ENABLE};

pub fn shutdown() -> ! {

    unsafe {
        let port_a = PM1A_CNT_BLK;
        let port_b = PM1B_CNT_BLK;
        let smi = SMI_CMD;
        let enable = ACPI_ENABLE;

        if smi != 0 && enable != 0 {
            core::arch::asm!("out dx, al", in("dx") smi as u16, in("al") enable);
            for _ in 0..10 { core::arch::asm!("out 0x80, al", in("al") 0u8); }
        }

        if port_a != 0 {
            let slp_en = 1 << 13;

            core::arch::asm!("cli");

            for slp_typ in [7u16, 5u16, 0u16, 0x34u16] {
                let val = (slp_typ << 10) | slp_en;
                
                core::arch::asm!("out dx, ax", in("dx") port_a as u16, in("ax") val);
                
                if port_b != 0 {
                    core::arch::asm!("out dx, ax", in("dx") port_b as u16, in("ax") val);
                }

                for _ in 0..2_000_000 { 
                    core::arch::asm!("nop");
                }
            }
        }
    }

    crate::kernel::console::LOGGER.error("ACPI Shutdown failed. System Halted.");
    loop {
        unsafe { core::arch::asm!("cli; hlt"); }
    }
}

pub fn reboot() -> ! {
    unsafe {
        // PS/2 Keyboard Controller reset
        core::arch::asm!("out 0x64, al", in("al") 0xFEu8);
        // Triple Fault fallback
        core::arch::asm!("lidt [{}]", in(reg) 0);
        core::arch::asm!("int 3");
    }
    loop { unsafe { core::arch::asm!("cli; hlt"); } }
}
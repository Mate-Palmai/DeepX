 /*
 * DeepX Project
 * Copyright (C) 2024-2026 - Máté Pálmai
 *
 * File: /src/arch/cpu.rs
 * Description: Low-level CPU instructions and register management.
 */



use core::arch::x86_64::__cpuid;
use core::arch::asm;

pub struct CpuInfo {
    pub brand: [u8; 48],
    pub vendor: [u8; 12],
}

impl CpuInfo {
    pub fn new() -> Option<Self> {
        let v_res = unsafe { __cpuid(0) };
        let mut vendor = [0u8; 12];
        
        vendor[0..4].copy_from_slice(&v_res.ebx.to_le_bytes());
        vendor[4..8].copy_from_slice(&v_res.edx.to_le_bytes());
        vendor[8..12].copy_from_slice(&v_res.ecx.to_le_bytes());

        let check_res = unsafe { __cpuid(0x80000000) };
        if check_res.eax < 0x80000004 {
            return Some(Self { 
                brand: [0u8; 48], 
                vendor 
            });
        }

        let mut brand = [0u8; 48];
        for i in 0..3 {
            let res = unsafe { __cpuid(0x80000002 + i) };
            let offset = (i as usize) * 16;
            brand[offset..offset+4].copy_from_slice(&res.eax.to_le_bytes());
            brand[offset+4..offset+8].copy_from_slice(&res.ebx.to_le_bytes());
            brand[offset+8..offset+12].copy_from_slice(&res.ecx.to_le_bytes());
            brand[offset+12..offset+16].copy_from_slice(&res.edx.to_le_bytes());
        }

        Some(Self { brand, vendor })
    }

    pub fn brand_as_str(&self) -> &str {
        let len = self.brand.iter().position(|&c| c == 0).unwrap_or(48);
        unsafe { core::str::from_utf8_unchecked(&self.brand[..len]) }.trim()
    }

    pub fn vendor_as_str(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(&self.vendor) }.trim()
    }
}


pub fn reboot() -> ! {
    unsafe {
        let mut timeout = 0xFFFF;
        while timeout > 0 && (inb(0x64) & 0x02) != 0 {
            timeout -= 1;
            core::hint::spin_loop();
        }
        
        outb(0x64, 0xFE);

        
        asm!(
            "lidt [rax]",
            "int 3",
            in("rax") &0u64,
            options(noreturn)
        );
    }

    loop {
        unsafe { asm!("hlt"); }
    }
}

#[inline(always)]
pub unsafe fn outb(port: u16, val: u8) {
    asm!("out dx, al", in("dx") port, in("al") val, options(nomem, nostack, preserves_flags));
}

#[inline(always)]
pub unsafe fn inb(port: u16) -> u8 {
    let res: u8;
    asm!("in al, dx", out("al") res, in("dx") port, options(nomem, nostack, preserves_flags));
    res
}
 /*
 * DeepX OS Project
 * Copyright (C) 2024-2026 - Máté Pálmai
 *
 * File: /src/arch/cpu.rs
 * Description: Low-level CPU instructions and register management.
 */



use core::arch::x86_64::__cpuid;
use core::arch::asm;

pub struct CpuInfo {
    pub brand: [u8; 48],
    pub vendor: [u8; 12], // A vendor mindig pontosan 12 karakter
}

impl CpuInfo {
    pub fn new() -> Option<Self> {
        // 1. Vendor lekérése (EAX = 0)
        let v_res = unsafe { __cpuid(0) };
        let mut vendor = [0u8; 12];
        
        // Fontos a sorrend: EBX, EDX, majd ECX!
        vendor[0..4].copy_from_slice(&v_res.ebx.to_le_bytes());
        vendor[4..8].copy_from_slice(&v_res.edx.to_le_bytes());
        vendor[8..12].copy_from_slice(&v_res.ecx.to_le_bytes());

        // 2. Brand string támogatás ellenőrzése
        let check_res = unsafe { __cpuid(0x80000000) };
        if check_res.eax < 0x80000004 {
            // Ha a brand nem támogatott, a vendort még visszaadhatjuk üres brand-el
            return Some(Self { 
                brand: [0u8; 48], 
                vendor 
            });
        }

        // 3. Brand string lekérése (EAX = 80000002..80000004)
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
        // A vendor általában nem null-terminált, hanem fix 12 byte
        unsafe { core::str::from_utf8_unchecked(&self.vendor) }.trim()
    }
}


pub fn reboot() -> ! {
    unsafe {
        // 1. Megpróbáljuk a PS/2 Controller (8042) resetet (0xFE parancs)
        // Ez a legszabványosabb módja a hardveres resetnek x86-on.
        
        // Várakozunk, amíg a bemeneti puffer üres nem lesz (bit 1 == 0)
        let mut timeout = 0xFFFF;
        while timeout > 0 && (inb(0x64) & 0x02) != 0 {
            timeout -= 1;
            core::hint::spin_loop();
        }
        
        // Reset parancs küldése a 0x64 portra
        outb(0x64, 0xFE);

        // 2. Ha a PS/2 nem működött, jön a Triple Fault kényszerítése.
        // Betöltünk egy üres IDT-t (méret 0), majd hívunk egy megszakítást.
        // A CPU nem talál kezelőt -> Double Fault -> Mivel azt sem találja -> Triple Fault -> RESET.
        
        asm!(
            "lidt [rax]",
            "int 3",
            in("rax") &0u64, // Egy nulla értékű pointer az IDTR-nek
            options(noreturn)
        );
    }

    // Biztonsági hurok, ha valami csoda folytán mégis itt lenne a vezérlés
    loop {
        unsafe { asm!("hlt"); }
    }
}

/// Alacsony szintű port írás (8-bit)
#[inline(always)]
pub unsafe fn outb(port: u16, val: u8) {
    asm!("out dx, al", in("dx") port, in("al") val, options(nomem, nostack, preserves_flags));
}

/// Alacsony szintű port olvasás (8-bit)
#[inline(always)]
pub unsafe fn inb(port: u16) -> u8 {
    let res: u8;
    asm!("in al, dx", out("al") res, in("dx") port, options(nomem, nostack, preserves_flags));
    res
}
 /*
 * DeepX OS Project
 * Copyright (C) 2024-2026 - Máté Pálmai
 *
 * File: /src/arch/cpu.rs
 * Description: Low-level CPU instructions and register management.
 */



use core::arch::x86_64::__cpuid;

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
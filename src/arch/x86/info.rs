/*
 * DeepX Project
 * Copyright (C) 2024-2026 - Máté Pálmai
 *
 * File: /src/arch/info.rs
 * Description: CPU information gathering.
 */

#[derive(Debug, Clone)]
pub struct CpuInfo {
    pub vendor: &'static str,
    pub brand: [u8; 48],
    pub cores: u32,
    pub threads: u32,
    pub features: CpuFeatures,
    pub temp_support: bool,
}

#[derive(Debug, Clone)]
pub struct CpuFeatures {
    pub sse: bool,
    pub sse2: bool,
    pub avx: bool,
    pub nx: bool,      
    pub htt: bool,     
}

pub fn get_cpu_info() -> CpuInfo {
    let mut brand = [0u8; 48];
    
    let res_0 = unsafe { core::arch::x86_64::__cpuid(0) };
    let vendor = match (res_0.ebx, res_0.edx, res_0.ecx) {
        (0x756e6547, 0x49656e69, 0x6c65746e) => "GenuineIntel",
        (0x68747541, 0x69746e65, 0x444d4163) => "AuthenticAMD",
        _ => "Unknown",
    };

    for i in 0..3 {
        let res = unsafe { core::arch::x86_64::__cpuid(0x80000002 + i) };
        let offset = i as usize * 16;
        brand[offset..offset+4].copy_from_slice(&res.eax.to_le_bytes());
        brand[offset+4..offset+8].copy_from_slice(&res.ebx.to_le_bytes());
        brand[offset+8..offset+12].copy_from_slice(&res.ecx.to_le_bytes());
        brand[offset+12..offset+16].copy_from_slice(&res.edx.to_le_bytes());
    }

    let mut cores = 1;
    let mut threads = 1;

    let res_1 = unsafe { core::arch::x86_64::__cpuid(1) };
    let logical_cpus = (res_1.ebx >> 16) & 0xFF;

    if vendor == "AuthenticAMD" {
        let res_8 = unsafe { core::arch::x86_64::__cpuid(0x80000008) };
        cores = (res_8.ecx & 0xFF) + 1;
        threads = logical_cpus;
    } else if vendor == "GenuineIntel" {
        let res_4 = unsafe { core::arch::x86_64::__cpuid_count(4, 0) };
        cores = ((res_4.eax >> 26) & 0x3F) + 1;
        threads = logical_cpus;
    } else {
        threads = logical_cpus;
    }

    let res_1 = unsafe { core::arch::x86_64::__cpuid(1) };
    let res_ext = unsafe { core::arch::x86_64::__cpuid(0x80000001) };
    
    let features = CpuFeatures {
        sse: (res_1.edx & (1 << 25)) != 0,
        sse2: (res_1.edx & (1 << 26)) != 0,
        avx: (res_1.ecx & (1 << 28)) != 0,
        htt: (res_1.edx & (1 << 28)) != 0,
        nx: (res_ext.edx & (1 << 20)) != 0,
    };

    let res_6 = unsafe { core::arch::x86_64::__cpuid(6) };
    let temp_support = (res_6.eax & 0x1) != 0;

    CpuInfo {
        vendor,
        brand,
        cores,
        threads,
        features,
        temp_support,
    }
}
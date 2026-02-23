/*
 * DeepX OS Project
 * Copyright (C) 2024-2026 - Máté Pálmai
 *
 * File: /src/kernel/mem/mapper.rs
 * Description: Memory mapper for kernel.
 */

use crate::kernel::mem::paging::{Mapper, VirtAddr, PageTableFlags};

pub struct MemoryMapper {
    pub mapper: Mapper, 
}

impl MemoryMapper {
    pub unsafe fn new() -> Self {
        Self {
            mapper: Mapper::new(),
        }
    }

    #[allow(dead_code)]
    pub fn map_to(&mut self, virt: VirtAddr, phys: u64, flags: PageTableFlags) {
        self.mapper.map_to(virt, phys, flags);
    }

    /// Egy egész tartományt leképez (szekvenciális fizikai és virtuális címek)
    /// Például: 1MB-nyi területet a videómemóriának.
    pub fn map_range(&mut self, start_virt: u64, start_phys: u64, size: u64, flags: PageTableFlags) {
        // Mindig lapmérethez (4096) igazítunk
        let start = start_virt & !0xfff;
        let end = (start_virt + size + 4095) & !0xfff;
        
        let mut current_virt = start;
        let mut current_phys = start_phys & !0xfff;

        while current_virt < end {
            self.mapper.map_to(
                VirtAddr(current_virt),
                current_phys,
                flags
            );
            current_virt += 4096;
            current_phys += 4096;
        }
    }

    #[allow(dead_code)]
    pub fn identity_map(&mut self, start_phys: u64, size: u64, flags: PageTableFlags) {
        self.map_range(start_phys, start_phys, size, flags);
    }
}
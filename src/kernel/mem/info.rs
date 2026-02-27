/*
 * DeepX Project
 * Copyright (C) 2024-2026 - Máté Pálmai
 *
 * File: /src/kernel/mem/info.rs
 * Description: Memory information retrieval and logging (heap-free).
 */

use limine::request::MemoryMapRequest;

#[derive(Debug, Copy, Clone)]
pub struct MemoryStats {
    pub usable: u64,
    pub reserved: u64,
    pub kernel: u64,
    pub boot_reclaim: u64,
    pub reserved_count: u64,
}

pub fn get_memory_stats(memmap_request: &MemoryMapRequest) -> Option<MemoryStats> {
    if let Some(memmap) = memmap_request.get_response() {
        let mut usable: u64 = 0;
        let mut reserved: u64 = 0;
        let mut kernel: u64 = 0;
        let mut boot_reclaim: u64 = 0;
        let mut reserved_count: u64 = 0;

        for entry in memmap.entries() {
            match entry.entry_type {
                limine::memory_map::EntryType::USABLE => usable += entry.length,
                limine::memory_map::EntryType::RESERVED => {
                    reserved += entry.length;
                    reserved_count += 1;
                },
                limine::memory_map::EntryType::EXECUTABLE_AND_MODULES => kernel += entry.length,
                limine::memory_map::EntryType::BOOTLOADER_RECLAIMABLE => boot_reclaim += entry.length,
                _ => {}
            }
        }

        Some(MemoryStats {
            usable,
            reserved,
            kernel,
            boot_reclaim,
            reserved_count,
        })
    } else {
        None
    }
}

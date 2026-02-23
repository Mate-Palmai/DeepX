/*
 * DeepX OS Project
 * Copyright (C) 2024-2026 - Máté Pálmai
 *
 * File: /src/kernel/mem/pmm.rs
 * Description: Physical Memory Manager for kernel.
 */

use limine::request::MemoryMapRequest;
use limine::memory_map::EntryType;

static mut BITMAP: *mut u8 = core::ptr::null_mut();
static mut BITMAP_SIZE: usize = 0;
static mut TOTAL_PAGES: usize = 0;

pub fn init(memmap_request: &MemoryMapRequest) {
    let response = memmap_request.get_response().expect("PMM: No memmap response");
    
    // 1. Számoljuk ki a teljes memóriát és a lapok számát
    let mut max_address: u64 = 0;
    for entry in response.entries() {
        let top = entry.base + entry.length;
        if top > max_address {
            max_address = top;
        }
    }
    
    unsafe {
        TOTAL_PAGES = (max_address / 4096) as usize;
        BITMAP_SIZE = TOTAL_PAGES / 8; // 8 lap = 1 bájt a bitmapben
    }

    // 2. Keressünk egy USABLE helyet a Bitmapnek
    for entry in response.entries() {
        if entry.entry_type == EntryType::USABLE && entry.length >= unsafe { BITMAP_SIZE } as u64 {
            unsafe {
                BITMAP = entry.base as *mut u8;
                // Inicializáljuk a bitmapet: alapértelmezésben minden foglalt (1)
                core::ptr::write_bytes(BITMAP, 0xFF, BITMAP_SIZE);
            }
            break;
        }
    }

    // 3. Szabadítsuk fel a ténylegesen USABLE területeket a bitmapben
    for entry in response.entries() {
        if entry.entry_type == EntryType::USABLE {
            for page in 0..(entry.length / 4096) {
                let addr = entry.base + (page * 4096);
                unreserve_page(addr);
            }
        }
    }

    // 4. A Bitmap által elfoglalt területet újra foglalttá tesszük, nehogy felülírjuk magunkat
    for i in 0..(unsafe { BITMAP_SIZE + 4095 } / 4096) {
        reserve_page(unsafe { BITMAP as u64 } + (i as u64 * 4096));
    }
}

// Segédfüggvények a bitek állításához
fn reserve_page(addr: u64) {
    let page_idx = (addr / 4096) as usize;
    unsafe {
        let byte_idx = page_idx / 8;
        let bit_idx = page_idx % 8;
        *BITMAP.add(byte_idx) |= 1 << bit_idx;
    }
}

fn unreserve_page(addr: u64) {
    let page_idx = (addr / 4096) as usize;
    unsafe {
        let byte_idx = page_idx / 8;
        let bit_idx = page_idx % 8;
        *BITMAP.add(byte_idx) &= !(1 << bit_idx);
    }
}

// Keressünk egy szabad lapot a bitmapben
pub fn alloc_frame() -> Option<u64> {
    unsafe {
        if BITMAP.is_null() { return None; }

        for byte_idx in 0..BITMAP_SIZE {
            let byte = *BITMAP.add(byte_idx);
            
            // Ha a bájt nem 0xFF, akkor van benne legalább egy 0-s bit (szabad lap)
            if byte != 0xFF {
                for bit_idx in 0..8 {
                    if (byte & (1 << bit_idx)) == 0 {
                        let page_idx = byte_idx * 8 + bit_idx;
                        let addr = page_idx as u64 * 4096;
                        
                        reserve_page(addr); // Megjelöljük foglaltként
                        return Some(addr);
                    }
                }
            }
        }
    }
    None // Nincs több szabad memória!
}

#[allow(dead_code)]
pub fn free_frame(addr: u64) {
    unreserve_page(addr);
}

pub fn print_ok() {
    unsafe {   
            crate::kernel::console::LOGGER.ok("PMM initialized");
        
    }
}

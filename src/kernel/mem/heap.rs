/*
 * DeepX Project
 * Copyright (C) 2024-2026 - Máté Pálmai
 *
 * File: /src/kernel/mem/heap.rs
 * Description: Heap initialization for kernel.
 */

use linked_list_allocator::LockedHeap;
use crate::kernel::mem::paging::{PageTableFlags, VirtAddr};
use crate::kernel::mem::mapper::MemoryMapper;
use crate::kernel::mem::pmm;

pub const HEAP_START: u64 = 0x_4444_4444_0000;
pub const HEAP_SIZE: u64 = 2 * 1024 * 1024; // 2 MiB

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

pub fn init(mapper: &mut MemoryMapper) {
    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

    for offset in (0..HEAP_SIZE).step_by(4096) {
        let phys = pmm::alloc_frame().expect("No physical memory for heap!");
        let virt = VirtAddr(HEAP_START + offset);

        mapper.map_range(virt.0, phys, 4096, flags);
        
    }

    unsafe {
        ALLOCATOR.lock().init(HEAP_START as *mut u8, HEAP_SIZE as usize);
    }
}
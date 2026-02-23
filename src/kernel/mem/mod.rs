
pub mod info;
pub mod pmm;
pub mod paging;
pub mod mapper;
pub mod heap;

#[allow(unused_imports)]
use alloc::format;

use crate::kernel::mem::mapper::MemoryMapper;
use limine::request::MemoryMapRequest;
use crate::kernel::mem::info::get_memory_stats;

pub fn init(memmap_request: &MemoryMapRequest) {
    // 1. PMM inicializálása (most már átadjuk a memmap-et)
    pmm::init(memmap_request);
    pmm::print_ok();

    // 2. Mapper létrehozása
    let mut mapper = unsafe { MemoryMapper::new() };

    // 3. Heap inicializálása
    heap::init(&mut mapper);
}

pub fn print_ok_memory(memmap_request: &MemoryMapRequest) {
    unsafe {
            crate::kernel::console::LOGGER.ok("Memory initialized");

            if let Some(stats) = get_memory_stats(memmap_request) {
                crate::kernel::console::LOGGER.info(&format!(
                    "RAM Usable:     ^&f{} MB",
                    stats.usable / 1024 / 1024
                ));
                crate::kernel::console::LOGGER.info(&format!(
                    "RAM Reserved:   ^&f{} MB",
                    stats.reserved / 1024 / 1024
                ));
                crate::kernel::console::LOGGER.info(&format!(
                    "Kernel Code:    ^&f{} MB",
                    stats.kernel / 1024 / 1024
                ));
                crate::kernel::console::LOGGER.info(&format!(
                    "Boot Reclaim:   ^&f{} MB",
                    stats.boot_reclaim / 1024 / 1024
                ));
                crate::kernel::console::LOGGER.info(&format!(
                    "Reserved Count: ^&f{}",
                    stats.reserved_count
                ));
            } else {
                crate::kernel::console::LOGGER.error("Memory map not found!");
            }
    }
}
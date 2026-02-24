
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

pub fn print_ok() {
    unsafe {
            crate::kernel::console::LOGGER.ok("Memory initialized");
    }
}

fn format_size(bytes: u64) -> alloc::string::String {
    if bytes >= 1024 * 1024 {
        format!("{} MB", bytes / 1024 / 1024)
    } else if bytes >= 1024 {
        format!("{} KB", bytes / 1024)
    } else {
        format!("{} B", bytes)
    }
}

pub fn print_memory_info(memmap_request: &MemoryMapRequest) {
    if let Some(stats) = get_memory_stats(memmap_request) {
        crate::kernel::console::LOGGER.info(&format!(
            "RAM Usable:     ^&f{}",
            format_size(stats.usable)
        ));
        crate::kernel::console::LOGGER.info(&format!(
            "RAM Reserved:   ^&f{}",
            format_size(stats.reserved)
        ));
        crate::kernel::console::LOGGER.info(&format!(
            "Kernel Code:    ^&f{}",
            format_size(stats.kernel)
        ));
        crate::kernel::console::LOGGER.info(&format!(
            "Boot Reclaim:   ^&f{}",
            format_size(stats.boot_reclaim)
        ));
        crate::kernel::console::LOGGER.info(&format!(
            "Reserved Count: ^&f{}",
            stats.reserved_count
        ));
    } else {
        crate::kernel::console::LOGGER.error("Memory map not found!");
    }
}
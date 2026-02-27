pub mod cpu;
pub mod idt;
pub mod gdt;
pub mod tss;
pub mod pic;
pub mod info;
pub mod apic;
pub mod timer;

#[allow(unused_imports)]
use alloc::format;

pub fn print_cpu_info() {
    let cpu_info = crate::arch::info::get_cpu_info();
    
    let brand_str = core::str::from_utf8(&cpu_info.brand)
        .unwrap_or("Invalid UTF-8")
        .trim();

    unsafe {
            crate::kernel::console::LOGGER.info(&format!("Vendor:       ^&f{}", cpu_info.vendor));
            crate::kernel::console::LOGGER.info(&format!("Brand:        ^&f{}", brand_str));
            crate::kernel::console::LOGGER.info(&format!("Cores:        ^&f{}", cpu_info.cores));
            crate::kernel::console::LOGGER.info(&format!("Threads:      ^&f{}", cpu_info.threads));
            crate::kernel::console::LOGGER.info(&format!("Features:     ^&f{:?}", cpu_info.features));
            crate::kernel::console::LOGGER.info(&format!("Temp Sensor:  ^&f{}", cpu_info.temp_support));
        
    }
}
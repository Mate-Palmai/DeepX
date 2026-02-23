use core::ptr::{read_volatile, write_volatile};

pub struct LocalApic {
    base_addr: usize,
}

impl LocalApic {
    pub unsafe fn new(base_addr: usize) -> Self {
        Self { base_addr }
    }

    unsafe fn write(&self, reg: u32, value: u32) {
        write_volatile((self.base_addr + reg as usize) as *mut u32, value);
    }

    unsafe fn read(&self, reg: u32) -> u32 {
        read_volatile((self.base_addr + reg as usize) as *const u32)
    }

    pub unsafe fn init(&self) {
        // Spurious Interrupt Vector Register + APIC Enable (bit 8)
        self.write(0xF0, self.read(0xF0) | 0x100 | 0xFF);

        crate::kernel::console::LOGGER.ok("Local APIC initialized");
    }

    pub fn eoi(&self) {
        unsafe { self.write(0xB0, 0); }
    }
}
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
        self.write(0xF0, self.read(0xF0) | 0x100 | 0xFF);

        crate::kernel::console::LOGGER.ok("Local APIC initialized");
    }

    pub fn eoi(&self) {
        unsafe { self.write(0xB0, 0); }
    }

    pub unsafe fn send_init(&self, apic_id: u8) {
        // INIT IPI: 0x000C4500 (Physical, Assert, Level, INIT)
        self.write(0x310, (apic_id as u32) << 24); // Destination
        self.write(0x300, 0x00004500);             // Command
    }

    pub unsafe fn send_sipi(&self, apic_id: u8, vector: u8) {
        // Startup IPI: 0x000C4600 + vector (address / 4096)
        self.write(0x310, (apic_id as u32) << 24);
        self.write(0x300, 0x00004600 | vector as u32);
    }
}
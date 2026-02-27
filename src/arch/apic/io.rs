pub struct IoApic {
    base_addr: usize,
}

impl IoApic {
    pub unsafe fn new(base_addr: usize) -> Self {
        Self { base_addr }
    }

    unsafe fn write(&self, reg: u8, value: u32) {
        let ioregsel = self.base_addr as *mut u32;
        let iowin = (self.base_addr + 0x10) as *mut u32;
        
        core::ptr::write_volatile(ioregsel, reg as u32);
        core::ptr::write_volatile(iowin, value);
    }

    pub unsafe fn init(&self) {
        crate::kernel::console::LOGGER.ok("I/O APIC initialized");
    }

    pub unsafe fn set_irq(&self, irq: u8, vector: u8) {
        let low_index = 0x10 + irq * 2;
        let high_index = 0x11 + irq * 2;
        let low_value = vector as u32;
        
        self.write(high_index, 0); // Destination CPU (0)
        self.write(low_index, low_value); 
    }
}
pub struct IoApic {
    base_addr: usize,
}

impl IoApic {
    pub unsafe fn new(base_addr: usize) -> Self {
        Self { base_addr }
    }

    unsafe fn write(&self, reg: u8, value: u32) {
        // Az Index regiszter az alapcímen van (0x00)
        let ioregsel = self.base_addr as *mut u32;
        // Az Adat (Window) regiszter mindig 0x10-re van az alapcímtől
        let iowin = (self.base_addr + 0x10) as *mut u32;
        
        core::ptr::write_volatile(ioregsel, reg as u32);
        core::ptr::write_volatile(iowin, value);
    }

    pub unsafe fn init(&self) {
        // Itt lehetne beállítani az IRQ átirányításokat (Redirection Table)
        crate::kernel::console::LOGGER.ok("I/O APIC initialized");
    }

    pub unsafe fn set_irq(&self, irq: u8, vector: u8) {
        let low_index = 0x10 + irq * 2;
        let high_index = 0x11 + irq * 2;

        // FONTOS: Az APIC regiszterek alapértelmezett állapota sokszor "Masked" (16. bit = 1).
        // Ha csak a vektort írod be, de nem kényszeríted a 16. bitet 0-ra, 
        // a megszakítás le lesz tiltva!
        
        // Alacsony 32 bit: 0-s bit = vector, 16-os bit = 0 (unmasked)
        let low_value = vector as u32; // (A 16. bit itt automatikusan 0)
        
        self.write(high_index, 0); // Destination CPU (0)
        self.write(low_index, low_value); 
    }
}
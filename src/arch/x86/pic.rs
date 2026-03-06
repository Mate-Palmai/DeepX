/*
 * DeepX Project
 * Copyright (C) 2024-2026 - Máté Pálmai
 *
 * File: /src/arch/pic.rs
 * Description: Legacy Programmable Interrupt Controller (8259) driver.
 */

use x86_64::instructions::port::Port;

const PIC_EOI: u8 = 0x20;

struct Pic {
    command: Port<u8>,
    data: Port<u8>,
}

impl Pic {
    const fn new(command: u16, data: u16) -> Self {
        Self {
            command: Port::new(command),
            data: Port::new(data),
        }
    }
}

pub struct ChainedPics {
    master: Pic,
    slave: Pic,
    offset_master: u8,
    offset_slave: u8,
}

impl ChainedPics {
    pub const fn new(offset_master: u8, offset_slave: u8) -> Self {
        Self {
            master: Pic::new(0x20, 0x21),
            slave: Pic::new(0xA0, 0xA1),
            offset_master,
            offset_slave,
        }
    }

    /// Initialize PICs and remap IRQs
    pub unsafe fn initialize(&mut self) {
        // ICW1: init + ICW4 needed
        self.master.command.write(0x11);
        self.slave.command.write(0x11);

        // ICW2: vector offsets
        self.master.data.write(self.offset_master);
        self.slave.data.write(self.offset_slave);

        // ICW3: cascading
        self.master.data.write(4); // slave on IRQ2
        self.slave.data.write(2);

        // ICW4: 8086 mode
        self.master.data.write(0x01);
        self.slave.data.write(0x01);

        // Mask everything by default
        self.master.data.write(0xFF);
        self.slave.data.write(0xFF);
    }

    /// Enable a specific IRQ line
    pub unsafe fn enable_irq(&mut self, irq: u8) {
        let (pic, bit) = if irq < 8 {
            (&mut self.master, irq)
        } else {
            (&mut self.slave, irq - 8)
        };

        let mask = pic.data.read();
        pic.data.write(mask & !(1 << bit));
    }

    /// Disable a specific IRQ line
    pub unsafe fn disable_irq(&mut self, irq: u8) {
        let (pic, bit) = if irq < 8 {
            (&mut self.master, irq)
        } else {
            (&mut self.slave, irq - 8)
        };

        let mask = pic.data.read();
        pic.data.write(mask | (1 << bit));
    }

    pub unsafe fn disable_pic(&mut self) {
        for i in 0..16 {
            self.disable_irq(i);
        }
        print_pic_disabled();
    }
    
    pub unsafe fn notify_end_of_interrupt(&mut self, int_id: u8) {
        if int_id >= self.offset_slave {
            self.slave.command.write(PIC_EOI);
        }
        self.master.command.write(PIC_EOI);
    }
}

pub static PICS: spinning_top::Spinlock<ChainedPics> =
    spinning_top::Spinlock::new(ChainedPics::new(32, 40));

pub fn print_pic_disabled() {
    unsafe {
            crate::kernel::console::LOGGER.warn("Legacy PIC disabled");
        
    }
}   

pub fn print_ok() {
    unsafe {
            crate::kernel::console::LOGGER.ok("PIC initialized");
        
    }
}
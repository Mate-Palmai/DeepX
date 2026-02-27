/*
 * DeepX Project
 * Copyright (C) 2024-2026 - Máté Pálmai
 *
 * File: /src/arch/gdt.rs
 * Description: Global Descriptor Table setup for memory segments.
 */

use core::mem::size_of;
use crate::arch::tss::TaskStateSegment;

#[repr(C, packed)]
struct GdtPtr {
    limit: u16,
    base: u64,
}

static mut TSS: TaskStateSegment = TaskStateSegment::new();
static mut GDT: [u64; 8] = [0; 8];

pub fn init(kernel_stack: u64) {
    unsafe {
       
        GDT[0] = 0; // Null

        // GDT[1]: Kernel Code (0x08)
        GDT[1] = 0x00AF9A000000FFFF; 

        // GDT[2]: Kernel Data (0x10)
        GDT[2] = 0x00CF92000000FFFF; 
        
        // GDT[3]: User Code (0x1B)
        GDT[3] = 0x00AFFA000000FFFF;

        // GDT[4]: User Data (0x23)
        GDT[4] = 0x00CFF2000000FFFF;

        TSS.privilege_stack_table[0] = kernel_stack;
        TSS.interrupt_stack_table[0] = kernel_stack;

        let tss_addr = core::ptr::addr_of!(TSS) as u64;
        let tss_limit = (size_of::<TaskStateSegment>() - 1) as u64;

        let mut low = 0u64;
        low |= tss_limit & 0xFFFF;
        low |= (tss_addr & 0xFFFFFF) << 16;
        low |= 0x89u64 << 40;
        low |= ((tss_limit >> 16) & 0xF) << 48;
        low |= (tss_addr & 0xFF000000) << 32;

        GDT[5] = low;
        GDT[6] = tss_addr >> 32;

        let ptr = GdtPtr {
            limit: (size_of::<[u64; 8]>() - 1) as u16,
            base: core::ptr::addr_of!(GDT) as u64,
        };

        core::arch::asm!(
            "lgdt [{0}]",
            "mov ax, 0x10",
            "mov ds, ax", "mov es, ax", "mov ss, ax", "mov fs, ax", "mov gs, ax",
            "push 0x08",
            "lea rax, [rip + 2f]",
            "push rax",
            "retfq",
            "2:",
            "mov ax, 0x28",
            "ltr ax",
            in(reg) &ptr,
            out("rax") _,
        );
    }
}

pub fn print_ok() {
    unsafe {
        crate::kernel::console::LOGGER.ok("TSS initialized");
        crate::kernel::console::LOGGER.ok("GDT initialized");
    }
}










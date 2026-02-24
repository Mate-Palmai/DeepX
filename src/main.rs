/*
 * DeepX OS Project
 * Copyright (C) 2024-2026 - Máté Pálmai
 *
 * File: /src/main.rs
 * Description: Kernel entry point and core initialization sequence.
 */

#![no_std]
#![no_main]
#![feature(abi_x86_interrupt, alloc_error_handler)]
#![allow(warnings)]

extern crate alloc;

mod arch;
mod kernel;

use core::panic::PanicInfo;
use limine::request::{FramebufferRequest, MemoryMapRequest, ModuleRequest};
use crate::kernel::boot::{set_phase, BootPhase};
use crate::kernel::mem::paging::{Mapper, VirtAddr, PageTableFlags};

// --- System Information ---
pub const KERNEL_VERSION: &str = "26m02-v0.0.7_Dev9";
pub const KERNEL_NAME: &str = "DeepX Kernel";
pub const KERNEL_MAJOR_VERSION_NAME: &str = "Proxima Deimos";

// Embedded userspace binaries
static OS_DISCOVERY: &[u8] = include_bytes!("kernel/os_discovery.bin");
static RECOVERY: &[u8] = include_bytes!("kernel/recovery.bin");

// --- Limine Bootloader Requests ---
#[used]
#[link_section = ".limine_requests"]
pub static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new(); 
#[used]
#[link_section = ".limine_requests"]
pub static MEMMAP_REQUEST: MemoryMapRequest = MemoryMapRequest::new(); 
#[used]
#[link_section = ".limine_requests"]
pub static MODULE_REQUEST: ModuleRequest = ModuleRequest::new();

/// Low-level assembly wrapper to switch the CPU to Ring 3 (User Mode).
/// Sets up segment selectors and performs an `iretq` to jump to user code.
core::arch::global_asm!(r#"
.global enter_user_mode
enter_user_mode:
    cli
    mov ax, 0x23
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    push 0x23      /* SS (Stack Segment) */
    push rsi       /* RSP (Stack Pointer) */
    push 0x202     /* RFLAGS (Interrupts enabled) */
    push 0x1B      /* CS (Code Segment) */
    push rdi       /* RIP (Instruction Pointer) */
    iretq
"#);

extern "C" {
    pub fn enter_user_mode(rip: u64, rsp: u64);
}

/// Main entry point called by the bootloader.
#[no_mangle]
pub extern "C" fn _start() -> ! {
    let kernel_stack: u64;
    unsafe { core::arch::asm!("mov {}, rsp", out(reg) kernel_stack); }

    if let Some(fb_res) = FRAMEBUFFER_REQUEST.get_response() {
        if let Some(fb) = fb_res.framebuffers().next() {
            init_sequence(&fb, kernel_stack);
        }
    }

    loop { unsafe { core::arch::asm!("hlt"); } }
}

/// Core initialization sequence.
fn init_sequence(fb: &limine::framebuffer::Framebuffer, stack: u64) {
    // 1. Graphics & Welcome
    let static_fb: &'static _ = unsafe { core::mem::transmute(fb) };
    {
        let mut console = crate::kernel::console::CONSOLE.lock();
        *console = Some(crate::kernel::console::console_base::ConsoleBase::new(static_fb));
        console.as_mut().unwrap().clear();
    }
    crate::kernel::boot::welcome::show_welcome();

    // 2. CPU Abstraction (GDT, IDT)
    set_phase(BootPhase::EarlyCpuInit);
    crate::arch::gdt::init(stack);
    arch::gdt::print_ok();
    crate::arch::idt::init();
    arch::idt::print_ok();

    // 3. Memory Management (PMM, VMM, Heap)
    set_phase(BootPhase::MemoryInit);
    kernel::mem::init(&MEMMAP_REQUEST); 
    kernel::mem::print_ok();

    // 4. Interrupt Controllers (APIC/PIC) & Timers
    set_phase(BootPhase::CpuInit);
    init_interrupt_controllers();
    crate::arch::timer::pit::init(100);
    unsafe { crate::arch::timer::lapic::init(); }

    #[cfg(feature = "dev")]
    {
        crate::kernel::console::LOGGER.info("--- System Diagnostics ---");
        arch::print_cpu_info();
        kernel::mem::print_memory_info(&MEMMAP_REQUEST);
        crate::kernel::console::LOGGER.info("--------------------------");
    }
    
    // 5. Drivers & Filesystem
    crate::kernel::drivers::input::init_input();
    set_phase(BootPhase::VfsInit);
    crate::kernel::fs::vfs::init_vfs(crate::kernel::fs::vfs::RootRamFS::new_node());

    // 6. System Services & Scheduling
    set_phase(BootPhase::SystunnelInit);
    crate::kernel::systunnel::init();
    setup_tasks();

    // 7. Ring 3 Transition
    crate::kernel::console::LOGGER.info("Starting Ring 3 Transition...");
    prepare_and_jump(OS_DISCOVERY, "OS Discovery");
}

/// Prepares the virtual address space for a user binary and jumps to Ring 3.
fn prepare_and_jump(binary: &[u8], name: &str) {
    let mut mapper = unsafe { Mapper::new() };
    let base_addr = 0x400000; // Standard entry point for user apps

    // Map binary pages
    let page_count = (binary.len() + 4095) / 4096;
    for i in 0..page_count {
        let phys = unsafe { crate::kernel::mem::pmm::alloc_frame() }.expect("PMM Exhausted");
        unsafe {
            let dest = (phys + crate::kernel::mem::paging::HHDM_OFFSET) as *mut u8;
            core::ptr::write_bytes(dest, 0, 4096);
            let offset = i * 4096;
            let size = core::cmp::min(4096, binary.len() - offset);
            core::ptr::copy_nonoverlapping(binary.as_ptr().add(offset), dest, size);
        }
        mapper.map_to(
            VirtAddr(base_addr + (i as u64 * 4096)),
            phys,
            PageTableFlags::PRESENT | PageTableFlags::USER_ACCESSIBLE | PageTableFlags::WRITABLE
        );
    }

    // Map user stack
    let stack_phys = unsafe { crate::kernel::mem::pmm::alloc_frame() }.expect("Stack PMM Exhausted");
    mapper.map_to(VirtAddr(0x500000), stack_phys, PageTableFlags::PRESENT | PageTableFlags::USER_ACCESSIBLE | PageTableFlags::WRITABLE);

    crate::kernel::console::LOGGER.ok(name);
    unsafe { enter_user_mode(base_addr, 0x501000); }
}

pub fn prepare_recovery_space_and_jump() -> ! {
    prepare_and_jump(RECOVERY, "Recovery Environment");
    loop { unsafe { core::arch::asm!("hlt"); } }
}

// Initializes the APIC or PIC based on hardware support.
fn init_interrupt_controllers() {
    unsafe {
        if arch::apic::has_apic() {
            arch::apic::init();
        } else {
            let mut pics = arch::pic::PICS.lock();
            pics.initialize();
            pics.enable_irq(0); // Timer
            pics.enable_irq(1); // Keyboard
            crate::kernel::console::LOGGER.warn("Using Legacy PIC");
        }
    }
}

fn setup_tasks() {
    use crate::kernel::process::{task::Task, SCHEDULER};
    let mut sched = SCHEDULER.lock();
    sched.add_task(Task::new_kernel_task()); // Idle task
    sched.add_task(Task::new(1, crate::kernel::console::safe_console::safe_console_task_entry as u64));
    
    #[cfg(feature = "dev")]
    sched.add_task(Task::new(2, crate::kernel::console::kernel_shell::shell_task_entry as u64));
}

// --- Error Handling ---

#[alloc_error_handler]
fn alloc_error_handler(_: core::alloc::Layout) -> ! {
    panic!("MEMORY ALLOCATION FAILURE");
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(fb_res) = FRAMEBUFFER_REQUEST.get_response() {
        if let Some(fb) = fb_res.framebuffers().next() {
            kernel::lib::panic::kernel_panic(&fb, "SYSTEM PANIC", &[], Some(info));
        }
    }
    loop { unsafe { core::arch::asm!("cli; hlt") } }
}
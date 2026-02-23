/*
 * DeepX OS Project
 * Copyright (C) 2024-2026 - Máté Pálmai
 *
 * File: /src/main.rs
 * Description: Kernel entry point and core initialization sequence.
 */

#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![allow(warnings)]

extern crate alloc;

mod arch;
mod kernel;

use limine::request::{FramebufferRequest, MemoryMapRequest, ModuleRequest};
use core::panic::PanicInfo;
use crate::kernel::console::Logger;
use crate::kernel::boot::{set_phase, BootPhase};
use crate::kernel::fs::vfs;
use crate::kernel::mem::paging::{Mapper, VirtAddr, PageTableFlags};

pub const KERNEL_VERSION: &str = "26m02-v0.0.7_Dev9";
pub const KERNEL_NAME: &str = "DeepX Kernel";
pub const KERNEL_MAJOR_VERSION_NAME: &str = "Proxima Deimos"; // Proxima = 0.x.x Deimos = 0.0.x

// Beágyazott rendszerfájlok
static OS_DISCOVERY: &[u8] = include_bytes!("kernel/os_discovery.bin");
static RECOVERY: &[u8] = include_bytes!("kernel/recovery.bin");

#[used]
#[link_section = ".limine_requests"]
pub static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new(); 

#[used]
#[link_section = ".limine_requests"]
pub static MEMMAP_REQUEST: MemoryMapRequest = MemoryMapRequest::new(); 

#[used]
#[link_section = ".limine_requests"]
pub static MODULE_REQUEST: ModuleRequest = ModuleRequest::new();

// Ring 3 átmenetet segítő assembly rutin
core::arch::global_asm!(r#"
.global enter_user_mode
enter_user_mode:
    cli
    mov ax, 0x23
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax

    push 0x23      /* SS */
    push rsi       /* RSP */
    push 0x202     /* RFLAGS (Interrupts enabled) */
    push 0x1B      /* CS (User Code Selector) */
    push rdi       /* RIP */
    iretq
"#);

extern "C" {
    pub fn enter_user_mode(rip: u64, rsp: u64);
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    let kernel_stack: u64;
    unsafe { core::arch::asm!("mov {}, rsp", out(reg) kernel_stack); }

    // 1. Grafikus felület (Framebuffer) inicializálása
    if let Some(fb_res) = FRAMEBUFFER_REQUEST.get_response() {
        if let Some(fb) = fb_res.framebuffers().next() {
            let static_fb: &'static limine::framebuffer::Framebuffer = unsafe { 
                core::mem::transmute(&fb) 
            };

            // Globális renderelő motor (ConsoleBase) létrehozása
            {
                let mut console_lock = crate::kernel::console::CONSOLE.lock();
                *console_lock = Some(crate::kernel::console::console_base::ConsoleBase::new(static_fb));
                
                if let Some(c) = console_lock.as_mut() {
                    c.clear();
                }
            } // Lock elengedve



            // Üdvözlő képernyő
            crate::kernel::boot::welcome::show_welcome();
            
            // 2. CPU és Megszakításkezelés inicializálása
            set_phase(BootPhase::EarlyCpuInit);
            crate::arch::gdt::init(kernel_stack);
            arch::gdt::print_ok();
            crate::arch::idt::init();
            arch::idt::print_ok();

            // 3. Memóriakezelés (PMM, VMM, Heap)
            set_phase(BootPhase::MemoryInit);
            kernel::mem::init(&MEMMAP_REQUEST); 
            kernel::mem::print_ok_memory(&MEMMAP_REQUEST);

            // 4. Hardveres időzítők és APIC
            set_phase(BootPhase::CpuInit);
            unsafe {
                let mut pics = arch::pic::PICS.lock();
                pics.initialize(); 
                arch::pic::print_ok();
            }

            if arch::apic::has_apic() {
                unsafe { arch::apic::init(); }
            } else {
                unsafe {
                    let mut pics = arch::pic::PICS.lock();
                    pics.enable_irq(0); // timer
                    pics.enable_irq(1); // keyboard
                    crate::kernel::console::LOGGER.warn("APIC not found, falling back to Legacy PIC");
                }
            }

            crate::arch::timer::pit::init(100);
            unsafe { crate::arch::timer::lapic::init(); }
            arch::print_cpu_info();

            // Driverek
            crate::kernel::drivers::input::init_input();

            // 5. Fájlrendszer inicializálása
            set_phase(BootPhase::VfsInit);
            let root_node = vfs::RootRamFS::new_node();
            vfs::init_vfs(root_node);
            vfs::dump_vfs_at_boot();

            // 6. Rendszerhívás interfész (Systunnel)
            set_phase(BootPhase::SystunnelInit);
            crate::kernel::systunnel::init();

            // 7. Scheduler és Taskok inicializálása
            use crate::kernel::process::task::Task;
            #[cfg(feature = "dev")]
            use crate::kernel::console::kernel_shell::shell_task_entry;
            use crate::kernel::console::safe_console::safe_console_task_entry;

            {
                let mut sched = crate::kernel::process::SCHEDULER.lock();
                
                // 0. task: Idle / Jelenlegi kontextus
                sched.add_task(Task::new_kernel_task());
                
                // 1. task: SafeConsole háttér-renderelő (RingBuffer -> Képernyő)
                sched.add_task(Task::new(1, safe_console_task_entry as u64));
                
                // 2. task: Kernel Shell
                #[cfg(feature = "dev")]
                sched.add_task(Task::new(2, shell_task_entry as u64));
            }

            // 8. Felhasználói mód előkészítése és ugrás
            crate::kernel::console::LOGGER.info("Starting Ring 3 Transition...");
            prepare_user_space_and_jump();

        }
    }

    // Kernel Idle Loop
    loop {
        unsafe { core::arch::asm!("hlt"); }
    }
}

/// Felhasználói memória mappolása és ugrás Ring 3-ba
fn prepare_user_space_and_jump() {
    let mut mapper = unsafe { Mapper::new() };
    let base_addr = 0x400000;

    // Az egész binárist (kód + rodata + data) egyetlen blokkba mappoljuk
    let page_count = (OS_DISCOVERY.len() + 4095) / 4096;
    for i in 0..page_count {
        let phys = unsafe { crate::kernel::mem::pmm::alloc_frame() }.expect("PMM failure");
        unsafe {
            let dest = (phys + crate::kernel::mem::paging::HHDM_OFFSET) as *mut u8;
            core::ptr::write_bytes(dest, 0, 4096); 
            let offset = i * 4096;
            let size = core::cmp::min(4096, OS_DISCOVERY.len() - offset);
            core::ptr::copy_nonoverlapping(OS_DISCOVERY.as_ptr().add(offset), dest, size);
        }
        mapper.map_to(
            VirtAddr(base_addr + (i as u64 * 4096)),
            phys,
            PageTableFlags::PRESENT | PageTableFlags::USER_ACCESSIBLE | PageTableFlags::WRITABLE
        );
    }
    
    // Adat terület mappolása (Virtual: 0x200000)
    // for i in 0..2 {
    //     let data_phys = unsafe { crate::kernel::mem::pmm::alloc_frame() }.expect("Data PMM failure");
    //     mapper.map_to(
    //         VirtAddr(0x200000 + (i as u64 * 4096)),
    //         data_phys,
    //         PageTableFlags::PRESENT | PageTableFlags::USER_ACCESSIBLE | PageTableFlags::WRITABLE
    //     );
    // }

    // Felhasználói stack mappolása (Virtual: 0x500000)
    let stack_phys = unsafe { crate::kernel::mem::pmm::alloc_frame() }.expect("Stack PMM failure");
    mapper.map_to(
        VirtAddr(0x500000),
        stack_phys,
        PageTableFlags::PRESENT | PageTableFlags::USER_ACCESSIBLE | PageTableFlags::WRITABLE
    );

    crate::kernel::console::LOGGER.ok("Transitioning to User Mode...");
    unsafe {
        enter_user_mode(0x400000, 0x501000);
    }
}

// kernel/main.rs (vagy ahol a betöltőid vannak)

// kernel/main.rs

pub fn prepare_recovery_space_and_jump() -> ! {
    crate::kernel::console::LOGGER.warn("RECOVERY: Re-mapping user space for Recovery Console...");

    let mut mapper = unsafe { Mapper::new() };
    let base_addr = 0x400000;
    let recovery_bin = crate::RECOVERY; // A beágyazott bináris

    // Pontosan ugyanaz a logika, mint az os_discovery-nél:
    // 1. Lefoglalunk új fizikai kereteket
    // 2. Bemásoljuk a recovery kódját a HHDM-en keresztül
    // 3. Újramappoljuk a 0x400000-et az ÚJ fizikai címekre
    let page_count = (recovery_bin.len() + 4095) / 4096;
    for i in 0..page_count {
        let phys = unsafe { crate::kernel::mem::pmm::alloc_frame() }.expect("Recovery PMM failure");
        unsafe {
            let dest = (phys + crate::kernel::mem::paging::HHDM_OFFSET) as *mut u8;
            core::ptr::write_bytes(dest, 0, 4096); 
            let offset = i * 4096;
            let size = core::cmp::min(4096, recovery_bin.len() - offset);
            core::ptr::copy_nonoverlapping(recovery_bin.as_ptr().add(offset), dest, size);
        }
        
        // Ez felülírja a korábbi OS_DISCOVERY mappingot az új fizikai címre
        mapper.map_to(
            VirtAddr(base_addr + (i as u64 * 4096)),
            phys,
            PageTableFlags::PRESENT | PageTableFlags::USER_ACCESSIBLE | PageTableFlags::WRITABLE
        );
    }

    crate::kernel::console::LOGGER.ok("Recovery code mapped successfully.");

    unsafe {
        crate::kernel::console::LOGGER.info("Starting Recovery Environment...");
        // A stack (0x501000) maradhat ugyanaz, a CPU egyszerűen felülírja a régit
        enter_user_mode(0x400000, 0x501000);
    }
    
    loop { unsafe { core::arch::asm!("hlt"); } }
}

#[alloc_error_handler]
fn alloc_error_handler(layout: core::alloc::Layout) -> ! {
    if let Some(fb_res) = crate::FRAMEBUFFER_REQUEST.get_response() {
        if let Some(fb) = fb_res.framebuffers().next() {
            crate::kernel::lib::panic::kernel_panic(
                &fb, 
                "MEMORY ALLOCATION FAILURE", 
                &["Failed to allocate heap memory."], 
                None
            );
        }
    }
    loop { unsafe { core::arch::asm!("cli; hlt") } }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(fb_res) = FRAMEBUFFER_REQUEST.get_response() {
        if let Some(fb) = fb_res.framebuffers().next() {
            let msg = info.message().as_str().unwrap_or("RUST RUNTIME PANIC");
            kernel::lib::panic::kernel_panic(&fb, msg, &[], Some(info));
        }
    }
    loop { unsafe { core::arch::asm!("cli; hlt") } }
}
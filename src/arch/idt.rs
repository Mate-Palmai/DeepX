/*
 * DeepX OS Project
 * Copyright (C) 2024-2026 - Máté Pálmai
 *
 * File: /src/arch/idt.rs
 * Description: Interrupt Descriptor Table and exception handlers.
 */

use core::mem::size_of;
use core::ptr::addr_of;
use crate::kernel::lib::utils::{u64_to_hex, u64_to_str};

// Alacsony szintű IDT struktúrák
#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct IdtEntry {
    offset_low: u16,
    selector: u16,
    ist: u8,
    flags: u8,
    offset_mid: u16,
    offset_high: u32,
    reserved: u32,
}

impl IdtEntry {
    pub const fn empty() -> Self {
        Self {
            offset_low: 0,
            selector: 0,
            ist: 0,
            flags: 0,
            offset_mid: 0,
            offset_high: 0,
            reserved: 0,
        }
    }

    pub fn set_handler(&mut self, handler: u64) {
        self.offset_low = handler as u16;
        self.offset_mid = (handler >> 16) as u16;
        self.offset_high = (handler >> 32) as u32;
        self.selector = 8; // Kernel Code Segment
        self.flags = 0x8E; // Present, Ring 0, Interrupt Gate
        self.ist = 0;
    }

    pub fn set_ist(&mut self, ist_index: u8) {
        self.ist = ist_index & 0b111;
    }
}

#[repr(C, packed)]
struct IdtPtr {
    limit: u16,
    base: u64,
}

// A CPU állapota kivételkor (extern "x86-interrupt" ABI szerinti sorrend)
#[repr(C)]
#[derive(Debug)]
pub struct InterruptStackFrame {
    pub instruction_pointer: u64,
    pub code_segment: u64,
    pub cpu_flags: u64,
    pub stack_pointer: u64,
    pub stack_segment: u64,
}

#[repr(C)]
pub struct CpuState {
    pub rax: u64, pub rbx: u64, pub rcx: u64, pub rdx: u64,
    pub rsi: u64, pub rdi: u64, pub rbp: u64,
}

static mut IDT: [IdtEntry; 256] = [IdtEntry::empty(); 256];
static mut USING_APIC: bool = false;

pub fn set_apic_mode(enabled: bool) {
    unsafe { USING_APIC = enabled; }
}

fn send_eoi(vector: u8) {
    unsafe {
        if USING_APIC {
            let lapic_eoi_ptr = 0xFEE0_00B0 as *mut u32;
            core::ptr::write_volatile(lapic_eoi_ptr, 0);
        } else {
            
            crate::arch::pic::PICS.lock().notify_end_of_interrupt(vector);
        }
    }
}

// ========== PANIC HANDLERS ==========

fn generic_panic_handler(msg: &str, details: &[&str]) {
    if let Some(response) = crate::FRAMEBUFFER_REQUEST.get_response() {
        if let Some(fb) = response.framebuffers().next() {
            crate::kernel::lib::panic::kernel_panic(&fb, msg, details, None);
        }
    }
    loop { unsafe { core::arch::asm!("cli; hlt"); } }
}

macro_rules! exception_handler {
    ($name:ident, $msg:expr) => {
        extern "x86-interrupt" fn $name(stack_frame: InterruptStackFrame) {
            let mut rip_buf = [0u8; 18];
            let details = ["RIP: 0x", u64_to_hex(stack_frame.instruction_pointer, &mut rip_buf)];
            generic_panic_handler($msg, &details);
        }
    };
}

macro_rules! exception_handler_err {
    ($name:ident, $msg:expr) => {
        extern "x86-interrupt" fn $name(stack_frame: InterruptStackFrame, error_code: u64) {
            let mut rip_buf = [0u8; 18];
            let mut err_buf = [0u8; 18];
            let details = [
                "RIP: 0x", u64_to_hex(stack_frame.instruction_pointer, &mut rip_buf),
                " ERR: ", u64_to_str(error_code, &mut err_buf)
            ];
            generic_panic_handler($msg, &details);
        }
    };
}

// ========== SYSTUNNEL ENTRY ==========

extern "C" {
    fn systunnel_entry();
}

core::arch::global_asm!(r#"
.global systunnel_entry
.extern systunnel_dispatch

systunnel_entry:
    cli
    push r15
    push r14
    push r13
    push r12
    push r11
    push r10
    push r9
    push r8
    push rbp
    push rdi
    push rsi
    push rdx
    push rcx
    push rbx
    push rax
    
    mov ax, ds
    push rax              
    
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    
    mov rdi, rsp
    call systunnel_dispatch
    
    pop rax               
    
    pop rax               
    pop rbx
    pop rcx
    pop rdx
    pop rsi
    pop rdi
    pop rbp
    pop r8
    pop r9
    pop r10
    pop r11
    pop r12
    pop r13
    pop r14
    pop r15
    
    iretq
"#);

#[no_mangle]
extern "C" fn systunnel_dispatch(frame_ptr: u64) {
    let frame = unsafe {
        &mut *(frame_ptr as *mut crate::kernel::systunnel::frame::SystunnelFrame)
    };
    crate::kernel::systunnel::dispatch::dispatch(frame);
}

// ========== SPECIAL HANDLERS ==========

extern "x86-interrupt" fn invalid_opcode(stack_frame: InterruptStackFrame) {
    let mut rip_buf = [0u8; 18];
    let mut rsp_buf = [0u8; 18];
    let details = [
        "RIP: 0x", u64_to_hex(stack_frame.instruction_pointer, &mut rip_buf),
        " | RSP: 0x", u64_to_hex(stack_frame.stack_pointer, &mut rsp_buf)
    ];
    generic_panic_handler("INVALID OPCODE (0x06)", &details);
}

extern "x86-interrupt" fn page_fault(stack_frame: InterruptStackFrame, error_code: u64) {
    let cr2: u64;
    let mut regs = CpuState { rax: 0, rbx: 0, rcx: 0, rdx: 0, rsi: 0, rdi: 0, rbp: 0 };

    unsafe {
        core::arch::asm!(
            "mov {0}, cr2",
            "mov {1}, rax", "mov {2}, rbx", "mov {3}, rcx", "mov {4}, rdx",
            "mov {5}, rsi", "mov {6}, rdi", "mov {7}, rbp",
            out(reg) cr2,
            out(reg) regs.rax, out(reg) regs.rbx, out(reg) regs.rcx, out(reg) regs.rdx,
            out(reg) regs.rsi, out(reg) regs.rdi, out(reg) regs.rbp
        );
    }

    let mut b_cr2 = [0u8; 18]; let mut b_rip = [0u8; 18];
    let mut b_rsp = [0u8; 18]; let mut b_err = [0u8; 18];
    let mut b_rax = [0u8; 18]; let mut b_rbx = [0u8; 18];
    let mut b_cs = [0u8; 18];

    let details = [
        "CR2: 0x", u64_to_hex(cr2, &mut b_cr2),
        " ERR: ",  u64_to_str(error_code, &mut b_err),
        " CS: ",   u64_to_str(stack_frame.code_segment, &mut b_cs),
        "\nRIP: 0x", u64_to_hex(stack_frame.instruction_pointer, &mut b_rip),
        " RSP: 0x", u64_to_hex(stack_frame.stack_pointer, &mut b_rsp),
        "\nRAX: 0x", u64_to_hex(regs.rax, &mut b_rax),
        " RBX: 0x", u64_to_hex(regs.rbx, &mut b_rbx),
    ];

    generic_panic_handler("PAGE FAULT (0x0E)", &details);
}

// ========== EXCEPTIONS ==========

exception_handler!(div_zero, "DIVIDE BY ZERO (0x00)");
exception_handler!(debug, "DEBUG EXCEPTION (0x01)");
exception_handler!(non_maskable, "NON-MASKABLE INTERRUPT (0x02)");
exception_handler!(breakpoint, "BREAKPOINT (0x03)");
exception_handler!(overflow, "OVERFLOW (0x04)");
exception_handler!(bound_range, "BOUND RANGE EXCEEDED (0x05)");
exception_handler!(device_not_avail, "DEVICE NOT AVAILABLE (0x07)");
exception_handler_err!(double_fault, "DOUBLE FAULT (0x08)");
exception_handler_err!(invalid_tss, "INVALID TSS (0x0A)");
exception_handler_err!(seg_not_present, "SEGMENT NOT PRESENT (0x0B)");
exception_handler_err!(stack_seg_fault, "STACK SEGMENT FAULT (0x0C)");
exception_handler_err!(gen_prot_fault, "GENERAL PROTECTION FAULT (0x0D)");
exception_handler!(fpu_error, "FPU FLOATING POINT ERROR (0x10)");
exception_handler_err!(align_check, "ALIGNMENT CHECK (0x11)");
exception_handler!(machine_check, "MACHINE CHECK (0x12)");

// ========== TIMER & KEYBOARD ==========

static mut TIMER_TICKS: u64 = 0;

pub extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    unsafe {
        // 1. Tick növelése azonnal
        TIMER_TICKS += 1;

        // 2. EOI KÜLDÉSE AZONNAL (Még a scheduler előtt!)
        // Ezzel mondjuk meg az APIC-nak, hogy jöhet a következő tick, 
        // függetlenül attól, hogy váltunk-e taskot.
        let lapic_eoi_ptr = 0xFEE0_00B0 as *mut u32;
        core::ptr::write_volatile(lapic_eoi_ptr, 0);

        // 3. CSAK EZUTÁN jöhet a scheduler
        // Ha a schedule() átvált egy másik taskra, az IRETQ ott fog lefutni,
        // de az APIC már megkapta az EOI-t, így a timer ketyeg tovább.
        if let Some(mut sched) = crate::kernel::process::SCHEDULER.try_lock() {
            sched.schedule();
        }
    }
}

pub fn get_timer_ticks() -> u64 {
    unsafe { TIMER_TICKS }
}

pub static mut LAST_SCANCODE: u8 = 0;

pub extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    use crate::kernel::drivers::keyboard::{Keyboard, KEY_QUEUE};

    let scancode = Keyboard::read_scancode();

    if let Some(mut queue) = KEY_QUEUE.try_lock() {
        queue.push_back(scancode);
    }

    unsafe {

        LAST_SCANCODE = scancode;

        if USING_APIC {
            let lapic_eoi_ptr = 0xFEE0_00B0 as *mut u32;
            core::ptr::write_volatile(lapic_eoi_ptr, 0);
        } else {
            crate::arch::pic::PICS.lock().notify_end_of_interrupt(33);
        }
    }
}

// ========== INITIALIZATION ==========

pub fn init() {
    unsafe {
        IDT[0].set_handler(div_zero as u64);
        IDT[1].set_handler(debug as u64);
        IDT[2].set_handler(non_maskable as u64);
        IDT[3].set_handler(breakpoint as u64);
        IDT[4].set_handler(overflow as u64);
        IDT[5].set_handler(bound_range as u64);
        IDT[6].set_handler(invalid_opcode as u64);
        IDT[7].set_handler(device_not_avail as u64);
        IDT[8].set_handler(double_fault as u64);
        IDT[8].set_ist(1);
        IDT[10].set_handler(invalid_tss as u64);
        IDT[11].set_handler(seg_not_present as u64);
        IDT[12].set_handler(stack_seg_fault as u64);
        IDT[13].set_handler(gen_prot_fault as u64);
        IDT[14].set_handler(page_fault as u64);
        IDT[14].set_ist(1);
        IDT[16].set_handler(fpu_error as u64);
        IDT[17].set_handler(align_check as u64);
        IDT[18].set_handler(machine_check as u64);

        IDT[32].set_handler(timer_interrupt_handler as u64);
        IDT[33].set_handler(keyboard_interrupt_handler as u64);

        IDT[0x80].set_handler(systunnel_entry as u64);
        IDT[0x80].flags = 0xEE; // Present, DPL 3, Interrupt Gate

        let idt_ptr = IdtPtr {
            limit: (size_of::<[IdtEntry; 256]>() - 1) as u16,
            base: addr_of!(IDT) as u64,
        };

        core::arch::asm!("lidt [{}]", in(reg) &idt_ptr);
    }
}

pub fn print_ok() {
    unsafe {
        crate::kernel::console::LOGGER.ok("IDT initialized");
    }
}
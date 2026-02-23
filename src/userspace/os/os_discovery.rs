#![no_std]
#![no_main]
#![feature(naked_functions)]

use core::arch::naked_asm;

// os/os_discovery.rs

#[no_mangle]
#[unsafe(naked)]
pub extern "C" fn _start() -> ! {

    

    unsafe {
        naked_asm!(
            "mov rsp, 0x501000",   // Stack beállítása
            "jmp {logic}",         // CALL helyett JMP, így nincs visszatérési cím mizéria
            logic = sym main_logic,
        );
    }


}

#[inline(never)]
fn main_logic() -> ! {
    // 1. Logolás
    syscall_log("Searching for os...");
    
    // 2. Rövidített várakozás a biztonság kedvéért
    for _ in 0..1_000_000 { 
        unsafe { core::arch::asm!("pause"); } 
    }

    unsafe {
        core::arch::asm!(
            "int 0x80",
            in("rax") 60,
            in("rdi") 1,
            options(noreturn)
        );
    }
}

#[inline(never)]
fn syscall_log(msg: &str) {
    unsafe {
        core::arch::asm!(
            "int 0x80",
            in("rax") 0,               // TunnelID::Log
            in("rdi") msg.as_ptr() as u64,
            in("rsi") msg.len() as u64,
            options(nostack)
        );
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
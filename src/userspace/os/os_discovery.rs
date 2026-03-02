#![no_std]
#![no_main]
#![feature(naked_functions)]

use core::arch::naked_asm;

#[no_mangle]
#[unsafe(naked)]
pub extern "C" fn _start() -> ! {
    unsafe {
        naked_asm!(
            "mov rsp, 0x501000", 
            "jmp {logic}", 
            logic = sym main_logic,
        );
    }
}

#[inline(never)]
fn main_logic() -> ! {
    systunnel_log("DeepX OS Discovery v0.2 starting...");
    let installer_path = "programs/installer.bin";
    systunnel_log("Checking for installation media...");

    if vfs_exists(installer_path) {
        systunnel_log("OK: DeepX Installer found! Initializing setup...");
        
        process_execute(installer_path);
    } else {
        systunnel_log("not found");
        systunnel_log("ERROR: No OS or Installer found on this system.");
        exit(1);
    }

    loop { unsafe { core::arch::asm!("hlt"); } }
}


#[inline(never)]
fn vfs_exists(path: &str) -> bool {
    let mut exists: u64 = 0;
    unsafe {
        core::arch::asm!(
            "int 0x80",
            inout("rax") 10u64=> exists, // ID 10: VfsExists
            in("rdi") path.as_ptr() as u64,
            in("rsi") path.len() as u64,
            options(nostack)
        );
    }

    exists == 1
}
#[inline(never)]
fn process_execute(path: &str) {
    unsafe {
        core::arch::asm!(
            "int 0x80",
            in("rax") 9,                  // ID 9: Execute
            in("rdi") path.as_ptr() as u64,
            in("rsi") path.len() as u64,
            options(nostack)
        );
    }
}


#[inline(never)]
fn systunnel_log(msg: &str) {
    unsafe {
        core::arch::asm!(
            "int 0x80",
            in("rax") 2,                  // ID 2: Log
            in("rdi") msg.as_ptr() as u64,
            in("rsi") msg.len() as u64,
            options(nostack)
        );
    }
}

#[inline(never)]
fn exit(code: u64) -> ! {
    unsafe {
        core::arch::asm!(
            "int 0x80",
            in("rax") 1,                  // ID 1: Exit
            in("rdi") code,
            options(noreturn)
        );
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
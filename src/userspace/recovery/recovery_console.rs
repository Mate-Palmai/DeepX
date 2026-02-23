#![no_std]
#![no_main]

use core::panic::PanicInfo;

// recovery_console/src/main.rs
#[no_mangle]
pub extern "C" fn _start() -> ! {
    let msg = "Welcome to DeepX Recovery Console!";
    unsafe {
        core::arch::asm!(
            "int 0x80",
            in("rax") 0,               // TunnelID::Log
            in("rdi") msg.as_ptr() as u64,
            in("rsi") msg.len() as u64,
        );
    }
    loop {}
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {} 
}
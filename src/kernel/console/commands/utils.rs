use crate::kernel::console::ring_buffer::SHELL_LOG_BUFFER;
use core::ptr;
use alloc::string::String;

#[allow(unused_imports)]
use alloc::format;

pub fn command_help() {
    let mut shell_log = SHELL_LOG_BUFFER.lock();
    shell_log.push_str(crate::kernel::console::commands::command_manager::SEPARATOR);
    shell_log.push_str(" ^&fDeepX Shell - Help Menu\n");
    shell_log.push_str("\n");

    let commands = [
        ("help",    "Show this help menu"),
        ("info",    "Display system information"),
        ("version", "Show kernel version"),
        ("clear",   "Clear the console screen"),
    ];

    for (cmd, desc) in commands {
        shell_log.push_str(&format!("^&9{:<10} ^&f- {}\n", cmd, desc));
    }
    shell_log.push_str(crate::kernel::console::commands::command_manager::SEPARATOR);
}




pub fn command_mdump(args: &[&str]) {
    let mut shell_log = SHELL_LOG_BUFFER.lock();

    if args.is_empty() {
        shell_log.push_str("^&cUsage: mdump <hex_address>\n");
        return;
    }

    let addr_str = args[0].trim_start_matches("0x");
    if let Ok(addr) = u64::from_str_radix(addr_str, 16) {
        let ptr = addr as *const u8;

        shell_log.push_str(crate::kernel::console::commands::command_manager::SEPARATOR);
        shell_log.push_str(&format!("^&eMemory Dump at: ^&f0x{:X}\n", addr));
        shell_log.push_str(crate::kernel::console::commands::command_manager::SEPARATOR);

        for row in 0..64 { // 32 line = 512 byte
            let row_addr = addr + (row * 16);
            shell_log.push_str(&format!("^&9{:016X}  ^&f", row_addr));
            
            let mut ascii_part = String::with_capacity(16);

            for col in 0..16 {
                let val = unsafe { ptr::read_volatile(ptr.add((row * 16 + col) as usize)) };
                shell_log.push_str(&format!("{:02X} ", val));

                if val >= 32 && val <= 126 {
                    ascii_part.push(val as char);
                } else {
                    ascii_part.push('.'); 
                }
            }

            shell_log.push_str(&format!(" ^&8| ^&7{}\n", ascii_part));
        }
        shell_log.push_str(crate::kernel::console::commands::command_manager::SEPARATOR);
    } else {
        shell_log.push_str("^&cInvalid hexadecimal address!\n");
    }
}
use crate::kernel::console::ring_buffer::SHELL_LOG_BUFFER;

#[allow(unused_imports)]
use alloc::format;

pub fn command_info(args: &[&str]) {
    let mut shell_log = SHELL_LOG_BUFFER.lock();

    let subcommand = args.get(0).map(|s| *s).unwrap_or("");
    let separator = crate::kernel::console::commands::command_manager::SEPARATOR;

    use crate::kernel::lib::utils::format_size;
    use crate::kernel::mem::info::get_memory_stats;

    let mem_stats = get_memory_stats(&crate::MEMMAP_REQUEST);
    let cpu_info = crate::arch::info::get_cpu_info();
    let brand_str = core::str::from_utf8(&cpu_info.brand)
        .unwrap_or("Invalid UTF-8")
        .trim();

    match subcommand {
        "ver" => {
            // --- VERSION INFO ---
            shell_log.push_str(separator);
            shell_log.push_str(&format!("^&9Kernel:      ^&f{} v{}\n", crate::KERNEL_NAME, crate::KERNEL_VERSION));
            shell_log.push_str(&format!("^&9OS: ^&f{}\n", "Unknown/Not installed"));
            
            shell_log.push_str(&format!("^&9Scheduler API:   ^&f{}\n", crate::kernel::process::SCHEDULER_VERSION));
            shell_log.push_str(&format!("^&9VFS API:         ^&f{}\n", crate::kernel::fs::vfs::VFS_VERSION));
            shell_log.push_str(&format!("^&9Systunnel ABI:   ^&f{}\n", crate::kernel::systunnel::SYSTUNNEL_VERSION));
            #[cfg(feature = "dev")]
            shell_log.push_str(&format!("^&9KernelShell API: ^&f{}\n", crate::kernel::console::kernel_shell::KERNEL_SHELL_VERSION));
            
            shell_log.push_str(separator);
        },
        "hw" => {
            // --- HARDWARE INFO ---
            shell_log.push_str(separator);
            shell_log.push_str("^&fHardware Resources\n");
            shell_log.push_str(" ^&7CPU:\n");
            shell_log.push_str(&format!("  ^&9Vendor:         ^&f{}\n", cpu_info.vendor));
            shell_log.push_str(&format!("  ^&9Brand:          ^&f{}\n", brand_str));
            shell_log.push_str(&format!("  ^&9Cores:          ^&f{}\n", cpu_info.cores));
            shell_log.push_str(&format!("  ^&9Threads:        ^&f{}\n", cpu_info.threads));
            shell_log.push_str(&format!("  ^&9Features:       ^&f{:?}\n", cpu_info.features));
            shell_log.push_str(&format!("  ^&9Temp Sensor:    ^&f{}\n", cpu_info.temp_support));
            shell_log.push_str(" ^&7MEM:\n");
            if let Some(stats) = mem_stats {
                shell_log.push_str(&format!("  ^&9RAM Usable:     ^&f{}\n", format_size(stats.usable)));
                shell_log.push_str(&format!("  ^&9RAM Reserved:   ^&f{}\n", format_size(stats.reserved)));
                shell_log.push_str(&format!("  ^&9Kernel Code:    ^&f{}\n", format_size(stats.kernel)));
                shell_log.push_str(&format!("  ^&9Boot Reclaim:   ^&f{}\n", format_size(stats.boot_reclaim)));
                shell_log.push_str(&format!("  ^&9Reserved Count: ^&f{}\n", stats.reserved_count));
            } else {
                shell_log.push_str("  ^&cError: Memory map not available\n");
            }
            shell_log.push_str(separator);
        },
        "help" => {
            // --- HELP ---
            shell_log.push_str(separator);

            let commands = [
                ("info",    "System summary"),
                ("info hw", "Hardware information"),
                ("info ver", "Version information"),
            ];

            for (cmd, desc) in commands {
                // Balra igazított parancsnév + leírás
                shell_log.push_str(&format!("^&9{:<10} ^&f- {}\n", cmd, desc));
            }

            shell_log.push_str(separator);
        },
        _ => {
            // --- DEFAULT ---
            shell_log.push_str(separator);

            shell_log.push_str(&format!("^&9Kernel:  ^&f{}\n", crate::KERNEL_VERSION));
            shell_log.push_str(&format!("^&9OS:      ^&f{}\n", "Unknown/Not installed"));

            shell_log.push_str(&format!("^&9Uptime:  ^&f{}s\n", "Not implemented"));

            if let Some(s) = mem_stats {
                let total = s.usable + s.kernel + s.boot_reclaim;
                shell_log.push_str(&format!("^&9Memory:  ^&f{} / {}\n", format_size(s.kernel), format_size(total)));
            }

            shell_log.push_str(&format!("^&9Cpu:     ^&f{}\n", brand_str)); 
            
            shell_log.push_str(&format!("\n^&7Type '^&finfo help^&7' for more details.\n"));
            shell_log.push_str(separator);
        }
    }
}

pub fn command_version() {
    let mut shell_log = SHELL_LOG_BUFFER.lock();
    shell_log.push_str(&format!("^&9Kernel version: ^&f{} ^&f{} (^&f{})\n", crate::KERNEL_NAME, crate::KERNEL_VERSION, crate::KERNEL_MAJOR_VERSION_NAME));
}




pub fn command_reboot() {
    let mut shell_log = SHELL_LOG_BUFFER.lock();
    shell_log.push_str("^&eRebooting system...\n");
    
    // Itt hívjuk az új arch függvényt
    crate::arch::cpu::reboot();
}
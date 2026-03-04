use crate::kernel::console::ring_buffer::SHELL_LOG_BUFFER;
use crate::kernel::fs::vfs::{ROOT_NODE, NodeType};
use crate::kernel::process::SCHEDULER;
use alloc::format;
use alloc::vec;

#[allow(unused_imports)]


// SYSTEM COMMANDS
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

    let (sec, frac) = crate::arch::timer::tsc::get_uptime();

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
                shell_log.push_str(&format!("^&9{:<10} ^&f- {}\n", cmd, desc));
            }

            shell_log.push_str(separator);
        },
        _ => {
            // --- DEFAULT ---
            shell_log.push_str(separator);

            shell_log.push_str(&format!("^&9Kernel:  ^&f{}\n", crate::KERNEL_VERSION));
            shell_log.push_str(&format!("^&9OS:      ^&f{}\n", "Unknown/Not installed"));

            
            shell_log.push_str(&format!("^&9Uptime:  ^&f{}.{:02}s\n", sec, frac / 100));

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
    
    crate::arch::cpu::reboot();
}

pub fn command_uptime() {
    let mut shell_log = SHELL_LOG_BUFFER.lock();
    
    let (sec, frac) = crate::arch::timer::tsc::get_uptime();
    shell_log.push_str(&format!("^&9Uptime: ^&f{}.{:02} seconds\n", sec, frac / 100));
}

// FS COMMANDS
pub fn command_ls() {
    let mut shell_log = SHELL_LOG_BUFFER.lock();
    let root_node = ROOT_NODE.lock();
    

    if let Some(root) = root_node.as_ref() {
        if let Ok(entries) = root.operations.readdir() {
            for entry in entries {
                shell_log.push_str(&format!("  ^&7/{:<16} ^&7{:>8} bytes\n", entry.name, entry.size));
            }
        }
    }
}

pub fn command_rd(args: &[&str]) {
    let mut shell_log = SHELL_LOG_BUFFER.lock();
    
    let filename = match args.get(0) {
        Some(name) => *name,
        None => {
            shell_log.push_str("^&cUsage: rd <filename>\n");
            return;
        }
    };

    let root_node = ROOT_NODE.lock();
    if let Some(root) = root_node.as_ref() {
        match root.operations.finddir(filename) {
            Ok(node) => {
                let read_limit = 512 * 1024;
                let size_to_read = core::cmp::min(node.size as usize, read_limit);
                let mut buffer = vec![0u8; size_to_read];

                match node.operations.read(0, &mut buffer) {
                    Ok(read_bytes) => {
                        shell_log.push_str("^&7--- START OF FILE ---\n");
                        match core::str::from_utf8(&buffer[..read_bytes]) {
                            Ok(text) => shell_log.push_str(text),
                            Err(_) => shell_log.push_str("^&8[Binary data - cannot display as text]\n"),
                        }
                        shell_log.push_str("\n^&7--- END OF FILE ---\n");
                    },
                    Err(_) => shell_log.push_str("^&cError: Failed to read file content.\n"),
                }
            },
            Err(_) => shell_log.push_str(&format!("^&cError: File '{}' not found.\n", filename)),
        }
    } else {
        shell_log.push_str("^&cError: VFS not initialized.\n");
    }
}


// SCHEDULER COMMANDS
use crate::kernel::process::task::TaskState;

pub fn command_tasks() {
    let mut task_data = alloc::vec::Vec::new();
    
    {
        let scheduler = SCHEDULER.lock();
        for task in scheduler.get_tasks() {
            task_data.push((
                task.id, 
                task.name.clone(), 
                task.state, 
                task.stack_top, 
                task.stack_bottom, 
                task.stack_pointer
            ));
        }
    }

    let mut shell_log = SHELL_LOG_BUFFER.lock();
    let separator = crate::kernel::console::commands::command_manager::SEPARATOR;

    shell_log.push_str(separator);
    shell_log.push_str("^&fID   NAME             STATE      SP            USAGE\n");
    shell_log.push_str(separator);

    for (id, name, state, top, bottom, sp) in task_data {
        let state_str = match state {
            crate::kernel::process::task::TaskState::Running => "^&aRunning",
            crate::kernel::process::task::TaskState::Ready   => "^&eReady  ",
            crate::kernel::process::task::TaskState::Blocked => "^&cBlocked",
        };

        let stack_size = top - bottom;
        let stack_used = top - sp;
        let usage_percent = if stack_size > 0 { (stack_used * 100) / stack_size } else { 0 };

        let line = format!(
            " ^&9{:<3} ^&f{:<16} {:<10} ^&70x{:08x}       ^&f{:>3}%\n",
            id, name, state_str, (sp & 0xFFFFFFFF) as u32, usage_percent
        );
        shell_log.push_str(&line);
    }
    
    shell_log.push_str(separator);
}
pub fn command_kill(args: &[&str]) {
    let mut shell_log = SHELL_LOG_BUFFER.lock();
    
    let id_str = match args.get(0) {
        Some(s) => *s,
        None => {
            shell_log.push_str("^&cUsage: kill <task_id>\n");
            return;
        }
    };

    if let Ok(id) = id_str.parse::<u64>() {
        let mut scheduler = SCHEDULER.lock();
        if scheduler.remove_task(id) {
            shell_log.push_str(&format!("^&aTask {} terminated successfully.\n", id));
        } else {
            shell_log.push_str(&format!("^&cError: Cannot kill task {} (protected or not found).\n", id));
        }
    } else {
        shell_log.push_str("^&cError: Invalid task ID.\n");
    }
}

pub fn command_block(args: &[&str]) {
    let mut shell_log = SHELL_LOG_BUFFER.lock();
    if let Some(id_str) = args.get(0) {
        if let Ok(id) = id_str.parse::<u64>() {
            if id == 0 {
                shell_log.push_str("^&cError: Cannot block kernel_main.\n");
                return;
            }
            let mut scheduler = SCHEDULER.lock();
            if scheduler.block_task(id) {
                shell_log.push_str(&format!("^&eTask {} is now Blocked.\n", id));
            } else {
                shell_log.push_str("^&cError: Task not found.\n");
            }
        }
    }
}

pub fn command_resume(args: &[&str]) {
    let mut shell_log = SHELL_LOG_BUFFER.lock();
    if let Some(id_str) = args.get(0) {
        if let Ok(id) = id_str.parse::<u64>() {
            let mut scheduler = SCHEDULER.lock();
            if scheduler.resume_task(id) {
                shell_log.push_str(&format!("^&aTask {} is now Ready.\n", id));
            } else {
                shell_log.push_str("^&cError: Task not found.\n");
            }
        }
    }
}


// MISC COMMANDS
pub fn command_panic() {
    let mut shell_log = SHELL_LOG_BUFFER.lock();
    
    drop(shell_log); 

    unsafe {
        core::arch::asm!(
            "mov rax, 123",
            "xor rdx, rdx",
            "mov rcx, 0",
            "div rcx",
            out("rax") _,
            out("rdx") _,
            out("rcx") _,
        );
    }
}
use crate::kernel::console::ring_buffer::SHELL_LOG_BUFFER;

#[allow(unused_imports)]
use alloc::format;

pub fn command_info() {
    let mut shell_log = SHELL_LOG_BUFFER.lock();

    let adressstring = "DeepX Kernel Memory Address Test String"; 
    let address = adressstring.as_ptr() as u64;

    shell_log.push_str(crate::kernel::console::commands::command_manager::SEPARATOR);
    shell_log.push_str(&format!("^&9Kernel: ^&f{} ^&f{}\n", crate::KERNEL_NAME, crate::KERNEL_VERSION));
    shell_log.push_str(&format!("^&9OS: ^&f{}\n", "Unknown/Not installed"));
    #[cfg(feature = "dev")]
    shell_log.push_str(&format!("^&9Kernel shell: ^&f{} ^&f{}\n", crate::kernel::console::kernel_shell::KERNEL_SHELL_NAME, crate::kernel::console::kernel_shell::KERNEL_SHELL_VERSION));
    shell_log.push_str(&format!("^&9Vfs version: ^&f{}\n", crate::kernel::fs::vfs::VFS_VERSION));
    shell_log.push_str(&format!("^&9Systunnel version: ^&f{}\n", crate::kernel::systunnel::SYSTUNNEL_VERSION));
    shell_log.push_str(crate::kernel::console::commands::command_manager::SEPARATOR);
}

pub fn command_version() {
    let mut shell_log = SHELL_LOG_BUFFER.lock();
    shell_log.push_str(&format!("^&9Kernel version: ^&f{} ^&f{} (^&f{})\n", crate::KERNEL_NAME, crate::KERNEL_VERSION, crate::KERNEL_MAJOR_VERSION_NAME));
}
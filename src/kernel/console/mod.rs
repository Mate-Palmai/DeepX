pub mod safe_console;
pub mod logger;
pub mod ring_buffer;
pub mod display_manager;
pub mod console_base;
pub mod commands;

#[cfg(feature = "dev")]
pub mod kernel_shell;

pub use logger::Logger;
pub use display_manager::*;

use spinning_top::Spinlock;
use crate::kernel::console::console_base::ConsoleBase;
pub use safe_console::SafeConsole;

#[cfg(feature = "dev")]
pub use kernel_shell::KernelShell;

#[derive(PartialEq, Copy, Clone)]
pub enum DisplayMode {
    RecoveryConsole,
    SafeConsole,
    #[cfg(feature = "dev")]
    KernelShell,
}

pub static LOGGER: Logger = Logger::new();

// CONSOLEBASE
pub static CONSOLE: Spinlock<Option<ConsoleBase>> = Spinlock::new(None);

// SAFE_CONSOLE
pub static SAFE_CONSOLE: SafeConsole = SafeConsole::new();

// KERNEL_SHELL
#[cfg(feature = "dev")]
pub static KERNEL_SHELL: Spinlock<Option<KernelShell>> = Spinlock::new(None);
pub mod safe_console;
pub mod logger;
pub mod ring_buffer;
pub mod display_manager;
pub mod console_base;
pub mod commands;

// 1. Csak dev módban létezik a modul
#[cfg(feature = "dev")]
pub mod kernel_shell;

pub use logger::Logger;
pub use display_manager::*;

use spinning_top::Spinlock;
use crate::kernel::console::console_base::ConsoleBase;
pub use safe_console::SafeConsole;

// 2. A use-t is védeni kell!
#[cfg(feature = "dev")]
pub use kernel_shell::KernelShell;

#[derive(PartialEq, Copy, Clone)]
pub enum DisplayMode {
    RecoveryConsole,
    SafeConsole,
    #[cfg(feature = "dev")] // Az enum opciót is elrejtheted
    KernelShell,
}

pub static LOGGER: Logger = Logger::new();

// CONSOLEBASE
pub static CONSOLE: Spinlock<Option<ConsoleBase>> = Spinlock::new(None);

// SAFE_CONSOLE
pub static SAFE_CONSOLE: SafeConsole = SafeConsole::new();

// KERNEL_SHELL - Csak akkor létezik a statikus változó, ha dev módban vagyunk
#[cfg(feature = "dev")]
pub static KERNEL_SHELL: Spinlock<Option<KernelShell>> = Spinlock::new(None);
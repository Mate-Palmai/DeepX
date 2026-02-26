// src/kernel/console/commands/command_manager.rs
use alloc::vec::Vec; // Ez hiányzott!
use super::system;
use super::utils;
use crate::kernel::console::ring_buffer::SHELL_LOG_BUFFER;

pub enum CommandResult {
    None,
    ClearScreen,
}

// public separator string
pub const SEPARATOR: &str = "^&8-------------------------------------------------\n";

pub fn dispatch(input: &str) -> CommandResult {
    let parts: Vec<&str> = input.split_whitespace().collect();
    if parts.is_empty() { return CommandResult::None; }

    let cmd = parts[0];
    let args = &parts[1..];

    // Végrehajtjuk a parancsot és elmentjük az eredményt
    let result = match cmd {
        // system commands
        "info" => { system::command_info(args); CommandResult::None }
        "version" => { system::command_version(); CommandResult::None }

        "help" => { utils::command_help(); CommandResult::None }
        "mdump" => { utils::command_mdump(args); CommandResult::None }

        "reboot" => { system::command_reboot(); CommandResult::None }

        "clear" => {
            SHELL_LOG_BUFFER.lock().clear();
            CommandResult::ClearScreen
        }
        _ => {
            let mut shell_log = SHELL_LOG_BUFFER.lock();
            shell_log.push_str("^&cUnknown command: ");
            shell_log.push_str(cmd);
            shell_log.push_str("\n^&fType 'help' for available commands\n");
            CommandResult::None
        }
    };

    // BIZTONSÁGI RESET: Ha nem töröltük a képernyőt, 
    // küldünk egy ^&7 kódót, hogy a prompt színe helyreálljon.
    if let CommandResult::None = result {
        SHELL_LOG_BUFFER.lock().push_str("^&f");
    }

    result
}
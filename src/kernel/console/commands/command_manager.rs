// src/kernel/console/commands/command_manager.rs
use alloc::vec::Vec;
use super::system;
use super::utils;
use crate::kernel::console::ring_buffer::SHELL_LOG_BUFFER;

pub enum CommandResult {
    None,
    ClearScreen,
}

pub const SEPARATOR: &str = "^&8-------------------------------------------------\n";

pub fn dispatch(input: &str) -> CommandResult {
    let parts: Vec<&str> = input.split_whitespace().collect();
    if parts.is_empty() { return CommandResult::None; }

    let cmd = parts[0];
    let args = &parts[1..];

    let result = match cmd {
        "info" => { system::command_info(args); CommandResult::None }
        "version" => { system::command_version(); CommandResult::None }

        "help" => { utils::command_help(); CommandResult::None }
        "mdump" => { utils::command_mdump(args); CommandResult::None }

        "reboot" => { system::command_reboot(); CommandResult::None }

        "ls" => { system::command_ls(); CommandResult::None }
        "rd" => { system::command_rd(args); CommandResult::None }

        "tasks" => { system::command_tasks(); CommandResult::None }
        "kill" => { system::command_kill(args); CommandResult::None }

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

    if let CommandResult::None = result {
        SHELL_LOG_BUFFER.lock().push_str("^&f");
    }

    result
}
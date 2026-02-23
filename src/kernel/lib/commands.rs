// /*
//  * DeepX OS Project
//  * Copyright (C) 2024-2026 - Máté Pálmai
//  *
//  * File: src/kernel/lib/commands.rs
//  * Description: Kernel command implementations and dispatcher.
//  */

// use crate::kernel::lib::shell::Shell;
// use alloc::vec::Vec;
// use alloc::string::String;
// use crate::{OS_NAME, OS_VERSION};
// use alloc::format;

// struct CommandInfo {
//     name: &'static str,
//     description: &'static str,
// }

// const COMMANDS: &[CommandInfo] = &[
//     CommandInfo { name: "help",   description: "Show help menu" },
//     CommandInfo { name: "ver",    description: "Display OS name and version" },
//     CommandInfo { name: "ls",     description: "List files in the current directory" },
//     CommandInfo { name: "cat",    description: "Display the content of a file" },
//     CommandInfo { name: "run",    description: "Execute a DXS script" },
//     CommandInfo { name: "clear",  description: "Clear the screen" },
//     CommandInfo { name: "reboot", description: "Restart the system" },
//     CommandInfo { name: "stop",   description: "Halt the CPU" },
// ];

// pub fn dispatch(shell: &mut Shell, parts: Vec<String>) {
//     if parts.is_empty() { return; }

//     match parts[0].as_str() {
//         "help"  => cmd_help(shell),
//         "ver" => cmd_version(shell),
//         "ls" => cmd_ls(shell),
//         "cat" => cmd_cat(shell, &parts),
//         "clear" => {
//             crate::kernel::lib::shell::clear_screen(shell.fb, 0x000000);
//             shell.reset_cursor();
//         }
//         "run" => cmd_run(shell, &parts),
//         "reboot" => cmd_reboot(shell),
//         "stop" => cmd_stop(shell),
//         _ => {
//             shell.newline();
//             shell.log("^&cUnknown command! Type 'help' for a list.");
//         }
//     }
// }

// fn cmd_help(shell: &mut Shell) {
//     shell.newline();
//     shell.log("^&eAvailable commands:");
//     shell.log("^&8-------------------------------------------");
    
//     for cmd in COMMANDS {
//         // Kiszámoljuk a szóközöket a név után, hogy a leírások egy oszlopba essenek
//         // A betűk 8 pixel szélesek, de a shell.log fix szélességűnek kezeli a karaktereket
//         let padding = " ".repeat(10 - cmd.name.len()); 
//         let line = alloc::format!("^&b{} {}^&f- {}", cmd.name, padding, cmd.description);
//         shell.log(&line);
//     }
    
//     shell.log("^&8-------------------------------------------");
// }

// fn cmd_version(shell: &mut Shell) {
//     shell.newline();
//     // Összeállítjuk a stringet a konstansokból
//     let ver_info = format!("^&b{}: ^&f{}", OS_NAME, OS_VERSION);
//     shell.log(&ver_info);
// }

// fn cmd_ls(shell: &mut Shell) {
//     shell.newline();
//     let vfs = crate::kernel::fs::VFS.lock();
//     for file in vfs.get_files() {
//         shell.log(file.name);
//     }
// }

// fn cmd_cat(shell: &mut Shell, parts: &[String]) {
//     shell.newline();
//     if parts.len() > 1 {
//         let vfs = crate::kernel::fs::VFS.lock();
//         if let Some(file) = vfs.get_files().iter().find(|f| f.name == parts[1]) {
//             if let Ok(content) = core::str::from_utf8(file.data) {
//                 shell.log(content);
//             } else { shell.log("^&cError: Binary file!"); }
//         } else { shell.log("^&cError: File not found!"); }
//     } else { shell.log("^&eUsage: cat <filename>"); }
// }

// fn cmd_run(shell: &mut Shell, parts: &[String]) {
//     shell.newline();
//     if parts.len() > 1 {
//         let vfs = crate::kernel::fs::VFS.lock();
//         if let Some(file) = vfs.get_files().iter().find(|f| f.name == parts[1]) {
//             if let Ok(script_text) = core::str::from_utf8(file.data) {
//                 crate::kernel::lib::scripting::run_script(script_text, shell);
//             }
//         } else { shell.log("^&cError: Script not found!"); }
//     } else { shell.log("^&eUsage: run <filename>"); }
// }

// // src/kernel/lib/commands.rs

// pub fn cmd_reboot(shell: &mut Shell) {
//     shell.newline();
//     shell.log("^&eAre you sure you want to reboot? (y/n)");
//     shell.waiting_for_confirm = true; // Megállítjuk a shellt
// }

// pub fn handle_confirmation(shell: &mut Shell, key: char) {
//     if !shell.waiting_for_confirm { return; }

//     match key {
//         'y' | 'Y' => {
//             shell.log("^&aRebooting...");
//             // Itt jön a hardveres reset kód
//             // Pl. a 8042 PS/2 controlleren keresztül:
//             unsafe {
//                 core::arch::asm!("out 0x64, al", in("al") 0xFEu8);
//             }
//         }
//         'n' | 'N' => {
//             shell.log("^&cAborted.");
//             shell.waiting_for_confirm = false;
//             shell.newline();
//             shell.print_prompt();
//         }
//         _ => {} // Egyéb gombokra nem reagál
//     }
// }

// fn cmd_stop(shell: &mut Shell) {
//     shell.newline();
//     shell.log("^&cSystem halting...");
//     // Itt hívható a CPU halt utasítás
//     unsafe { core::arch::asm!("cli; hlt"); }
// }
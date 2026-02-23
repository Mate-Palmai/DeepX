// /*
//  * DeepX OS Project
//  * Copyright (C) 2024-2026 - Máté Pálmai
//  *
//  * File: /src/kernel/lib/scripting.rs
//  * Description: Scripting engine for DXS.
//  */

// use crate::kernel::lib::shell::Shell;
// use alloc::format;

// pub fn run_script(code: &str, shell: &mut Shell) {
//     let mut lines = code.lines().peekable();
//     let mut is_display = false;

//     shell.is_running_script = true;

//     // 1. Fejléc beolvasása
//     if let Some(first_line) = lines.peek() {
//         if first_line.trim() == "script_type = display" {
//             is_display = true;
//             lines.next(); // Átugorjuk a fejlécet
//             crate::kernel::lib::shell::clear_screen(shell.fb, 0x000000);
//             shell.reset_cursor();
//             shell.draw_input_bar();
//         } else if first_line.trim() == "script_type = shell" {
//             lines.next();
//         }
//     }



//     // 2. Parancsok futtatása
//     for line in lines {
//         let line = line.trim();
//         if line.is_empty() || line.starts_with('#') { continue; }

//         let parts: alloc::vec::Vec<&str> = line.split_whitespace().collect();
//         match parts[0] {
//             "printc" => { // print without [DXS] prefix
//                 let msg = line[6..].trim().trim_matches('"'); 
//                 shell.log(&format!("^&f{}", msg));
//             }
//             "print" => {
//                 let msg = line[5..].trim().trim_matches('"'); 
//                 shell.log(&format!("^&a[DXS] ^&f{}", msg));
//             }
//             "draw" if parts.len() == 4 => {
//                 let x = parts[1].parse().unwrap_or(0);
//                 let y = parts[2].parse().unwrap_or(0);
//                 let color = u32::from_str_radix(parts[3].trim_start_matches("0x"), 16).unwrap_or(0xFFFFFF);
//                 shell.put_pixel(x, y, color);
//             }
//             "rect" if parts.len() == 6 => {
//                 let x = parts[1].parse().unwrap_or(0);
//                 let y = parts[2].parse().unwrap_or(0);
//                 let w = parts[3].parse().unwrap_or(0);
//                 let h = parts[4].parse().unwrap_or(0);
//                 let color = u32::from_str_radix(parts[5].trim_start_matches("0x"), 16).unwrap_or(0xFFFFFF);
//                 shell.draw_rect(x, y, w, h, color);
//             }
//             "clear" => {
//                 crate::kernel::lib::shell::clear_screen(shell.fb, 0x000000);
//                 if is_display { shell.draw_input_bar(); }
//                 shell.reset_cursor();
//             }
//             _ => {}
//         }
//     }

//     // Ha NEM display mód, akkor rögtön kikapcsoljuk a script módot futás után
//     if !is_display {
//         shell.is_running_script = false;
//     }
//     // Ha display mód, akkor shell.is_running_script TRUE marad, 
//     // amíg a user be nem írja az 'exit'-et a handle_script_input-ban.
// }
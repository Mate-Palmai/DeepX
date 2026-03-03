/*
 * DeepX Project
 * Copyright (C) 2024-2026 - Máté Pálmai
 *
 * File: /src/kernel/console/kernel_shell.rs
 * Description: Kernel shell implementation for interactive command execution and system information display.
 */

#![cfg(feature = "dev")]

use crate::kernel::console::console_base::ConsoleBase;
use crate::kernel::console::ring_buffer::SHELL_LOG_BUFFER;
use crate::kernel::console::CONSOLE;
use crate::kernel::console::DisplayMode;
use crate::kernel::console::display_manager::{CURRENT_MODE, NEEDS_FULL_REDRAW};
use crate::kernel::console::commands::command_manager;
use alloc::string::String;
use spinning_top::Spinlock;

pub static INPUT_BUFFER: Spinlock<String> = Spinlock::new(String::new());

pub const KERNEL_SHELL_VERSION: &str = "v0.3.2";

pub struct CursorPos {
    pub x: u64,
    pub y: u64,
}

static mut LAST_CURSOR_POS: CursorPos = CursorPos { x: 0, y: 0 };

pub struct KernelShell;

impl KernelShell {
    pub const fn new() -> Self {
        Self
    }

    pub fn init(&self) {

        if let Some(mut log) = SHELL_LOG_BUFFER.try_lock() {
            log.push_str("^&fWelcome to the DeepX Kernel Shell!\n");
            log.push_str("^&fType 'help' for a list of commands.\n");
            log.push_str("^&f-----------------------------------\n");
        }
    }
    
    pub fn render(&self, console: &mut ConsoleBase) {
        if let Some(log) = SHELL_LOG_BUFFER.try_lock() {
            console.render_buffer(&log);
        }
    }

    pub fn write_str(&self, text: &str) {
        if let Some(mut log) = SHELL_LOG_BUFFER.try_lock() {
            log.push_str(text);
        }
    }

    pub fn handle_char(&self, c: char) {
        match c {
            '\n' => {
                let mut input = INPUT_BUFFER.lock();
                let command = input.clone();
                input.clear();
                drop(input); 

                self.write_str("\n");
                self.execute(&command);
                self.write_prompt();

            },
            '\x08' => {
                let mut input = INPUT_BUFFER.lock();
                if !input.is_empty() {
                    input.pop(); 
                    drop(input);

                    if let Some(mut log) = SHELL_LOG_BUFFER.try_lock() {
                        log.pop();
                    }
                }
            },
            _ => {
                if let Some(mut input) = INPUT_BUFFER.try_lock() {
                    input.push(c);
                }
                let mut s = alloc::string::String::new();
                s.push(c);
                self.write_str(&s);
            }
        }
    }

    pub fn write_prompt(&self) {
        self.write_str("\n> ");
    }

    fn execute(&self, cmd: &str) {
        let trimmed = cmd.trim();
        if trimmed.is_empty() { return; }
        
        let result = command_manager::dispatch(trimmed);


        match result {
            command_manager::CommandResult::ClearScreen => {
                if let Some(mut console_lock) = CONSOLE.try_lock() {
                    if let Some(console) = console_lock.as_mut() {
                        console.clear();
                    }
                }
            }
            command_manager::CommandResult::None => {}
        }

    }
}

use crate::kernel::console::display_manager;



pub fn shell_task_entry() {
    unsafe { core::arch::asm!("sti"); }
    let shell = KernelShell::new();
    shell.init();
    shell.write_prompt();


    loop {
        display_manager::process_keyboard_queue();

        if unsafe { CURRENT_MODE == DisplayMode::KernelShell } {
            while let Some(c) = crate::kernel::drivers::input::pop_key() {
                shell.handle_char(c);
            }

            if let Some(mut console_lock) = CONSOLE.try_lock() {
                if let Some(console) = console_lock.as_mut() {
                    
                    if let Some(log) = SHELL_LOG_BUFFER.try_lock() {
                        console.render_buffer(&log);
                    }

                    unsafe {
                        let old_fg = console.current_fg;
                        let saved_x = console.cursor_x;
                        let saved_y = console.cursor_y;

                        console.cursor_x = LAST_CURSOR_POS.x;
                        console.cursor_y = LAST_CURSOR_POS.y;
                        console.current_fg = 0x000000; 
                        console.draw_char(b' ');      

                        console.cursor_x = saved_x;
                        console.cursor_y = saved_y;
                        console.current_fg = 0xFFFFFF; 
                        console.draw_char(b'_');

                        LAST_CURSOR_POS.x = saved_x;
                        LAST_CURSOR_POS.y = saved_y;

                        console.cursor_x = saved_x;
                        console.cursor_y = saved_y;
                        console.current_fg = old_fg;
                    }

                }
            }
        }
        for _ in 0..10_000 { unsafe { core::arch::asm!("pause"); } }
    }
}
/*
 * DeepX Project
 * Copyright (C) 2024-2026 - Máté Pálmai
 *
 * File: /src/kernel/console/logger.rs
 * Description: Logger implementation for kernel, providing various log levels and formatting.
 */

use crate::kernel::console::ring_buffer::LOG_BUFFER;
use crate::kernel::drivers::rtc::read_rtc_time;
use crate::kernel::console::CONSOLE; 
use crate::kernel::console::DisplayMode;

pub struct Logger;

impl Logger {
    pub const fn new() -> Self {
        Self
    }

    fn log_internal(&self, level: &str, color: &str, msg: &str, with_prefix: bool, newline: bool) {
        let (h, m, s) = if with_prefix { read_rtc_time() } else { (0, 0, 0) };

        {
            let mut log = LOG_BUFFER.lock();
            use core::fmt::Write;

            if with_prefix {
                let _ = write!(
                    &mut *log,
                    "^&8[{:02}:{:02}:{:02}] ^&7[{}{}^&7] ^&7{}",
                    h, m, s, color, level, msg
                );
            } else {
                log.push_str(msg);
            }

            if newline {
                log.push_str("\n");
            }
        } 
        if unsafe { crate::kernel::console::display_manager::CURRENT_MODE == DisplayMode::SafeConsole } {
            if let Some(mut console_lock) = CONSOLE.try_lock() {
                if let Some(console) = console_lock.as_mut() {
                    let log_data = LOG_BUFFER.lock();
                    console.render_buffer(&log_data);
                }
            }
        }
    }

    // normal logs
    pub fn info(&self, msg: &str)  { self.log_internal(" INFO ", "^&9", msg, true, true); }
    pub fn ok(&self, msg: &str)    { self.log_internal("  OK  ", "^&2", msg, true, true); }
    pub fn warn(&self, msg: &str)  { self.log_internal(" WARN ", "^&6", msg, true, true); }
    pub fn error(&self, msg: &str) { self.log_internal("FAILED", "^&4", msg, true, true); }
    pub fn wait(&self, msg: &str)  { self.log_internal(" WAIT ", "^&e", msg, true, true); }
    pub fn debug(&self, msg: &str) { self.log_internal(" DBUG ", "^&5", msg, true, true); }

    pub fn tunnel(&self, msg: &str) { self.log_internal("TUNNEL", "^&3", msg, true, true); }
    pub fn scheduler(&self, msg: &str) { self.log_internal(" SCED ", "^&e", msg, true, true); }

    // logs without newline
    pub fn info_nl(&self, msg: &str)  { self.log_internal(" INFO ", "^&9", msg, true, false); }
    pub fn ok_nl(&self, msg: &str)    { self.log_internal("  OK  ", "^&2", msg, true, false); }
    pub fn warn_nl(&self, msg: &str)  { self.log_internal(" WARN ", "^&6", msg, true, false); }
    pub fn error_nl(&self, msg: &str) { self.log_internal("FAILED", "^&4", msg, true, false); }
    pub fn wait_nl(&self, msg: &str)  { self.log_internal(" WAIT ", "^&e", msg, true, false); }
    pub fn debug_nl(&self, msg: &str) { self.log_internal(" DBUG ", "^&5", msg, true, false); }

    // other logs
    pub fn raw(&self, msg: &str) { self.log_internal("", "", msg, false, false); } // without prefix and newline
    pub fn raw_line(&self, msg: &str) { self.log_internal("", "", msg, false, true); } // without prefix, with newline

    pub fn custom(&self, level: &str, color_code: &str, msg: &str) { self.log_internal(level, color_code, msg, true, true); }

}
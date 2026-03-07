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
use crate::kernel::sync::spinlock::Spinlock;
use crate::kernel::console::SAFE_CONSOLE;
use core::fmt::Write;

pub struct Logger;

static LOGGER_LOCK: Spinlock<()> = Spinlock::new(());

impl Logger {
    pub const fn new() -> Self {
        Self
    }

    fn log_internal(&self, level: &str, color: &str, msg: &str, with_prefix: bool, newline: bool) {
        let _lock = LOGGER_LOCK.lock();

        let mut buf = [0u8; 512];
        let mut wrapper = Wrapper::new(&mut buf);
        let (h, m, s) = if with_prefix { read_rtc_time() } else { (0, 0, 0) };

        if with_prefix {
            let _ = write!(wrapper, "^&8[{:02}:{:02}:{:02}] ^&7[{}{}^&7] ^&7{}", h, m, s, color, level, msg);
        } else {
            let _ = wrapper.write_str(msg);
        }
        if newline { let _ = wrapper.write_str("\n"); }

        {
            let mut log = LOG_BUFFER.lock();
            log.push_str(wrapper.as_str());
        }

        SAFE_CONSOLE.render_safely();
    }


    pub fn info(&self, msg: &str)      { self.log_internal(" INFO ", "^&9", msg, true, true); }
    pub fn ok(&self, msg: &str)        { self.log_internal("  OK  ", "^&2", msg, true, true); }
    pub fn warn(&self, msg: &str)      { self.log_internal(" WARN ", "^&6", msg, true, true); }
    pub fn error(&self, msg: &str)     { self.log_internal("FAILED", "^&4", msg, true, true); }
    pub fn wait(&self, msg: &str)      { self.log_internal(" WAIT ", "^&e", msg, true, true); }
    pub fn debug(&self, msg: &str)     { self.log_internal(" DBUG ", "^&5", msg, true, true); }
    pub fn tunnel(&self, msg: &str)    { self.log_internal("TUNNEL", "^&3", msg, true, true); }
    pub fn scheduler(&self, msg: &str) { self.log_internal(" SCED ", "^&e", msg, true, true); }

    pub fn info_nl(&self, msg: &str)   { self.log_internal(" INFO ", "^&9", msg, true, false); }
    pub fn ok_nl(&self, msg: &str)     { self.log_internal("  OK  ", "^&2", msg, true, false); }
    pub fn warn_nl(&self, msg: &str)   { self.log_internal(" WARN ", "^&6", msg, true, false); }
    pub fn error_nl(&self, msg: &str)  { self.log_internal("FAILED", "^&4", msg, true, false); }
    pub fn wait_nl(&self, msg: &str)   { self.log_internal(" WAIT ", "^&e", msg, true, false); }
    pub fn debug_nl(&self, msg: &str)  { self.log_internal(" DBUG ", "^&5", msg, true, false); } 

    pub fn raw(&self, msg: &str)       { self.log_internal("", "", msg, false, false); }
    pub fn raw_line(&self, msg: &str)  { self.log_internal("", "", msg, false, true); }
    pub fn custom(&self, level: &str, color_code: &str, msg: &str) { self.log_internal(level, color_code, msg, true, true); }
}

struct Wrapper<'a> {
    buf: &'a mut [u8],
    offset: usize,
}

impl<'a> Wrapper<'a> {
    fn new(buf: &'a mut [u8]) -> Self {
        Self { buf, offset: 0 }
    }
    fn as_str(&self) -> &str {
        core::str::from_utf8(&self.buf[..self.offset]).unwrap_or("")
    }
}

impl<'a> core::fmt::Write for Wrapper<'a> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let bytes = s.as_bytes();
        let remainder = self.buf.len() - self.offset;
        let to_copy = if bytes.len() > remainder { remainder } else { bytes.len() };
        self.buf[self.offset..self.offset + to_copy].copy_from_slice(&bytes[..to_copy]);
        self.offset += to_copy;
        Ok(())
    }
}
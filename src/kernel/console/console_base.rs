/*
 * DeepX Project
 * Copyright (C) 2024-2026 - Máté Pálmai
 *
 * File: /src/kernel/console/console_base.rs
 * Description: Base console implementation for rendering text and managing the framebuffer in the kernel.
 * This is the core rendering logic for the console, handling text output, colors, and basic formatting. It is designed to be used by higher-level console implementations like the SafeConsole.
 * The ConsoleBase directly interacts with the framebuffer to draw characters and manage the cursor position. It also includes an info panel that displays system information such as time, uptime, and last key pressed.
 * This module is critical for the kernel's logging and display functionality, and is used by the SafeConsole to render log messages and system information in a visually appealing way.
 */


use limine::framebuffer::Framebuffer;

const FONT: &[u8; 2048] = include_bytes!("../../font.bin");

static mut INTERNAL_DEBUG_TICK: u64 = 0;

pub struct ConsoleBase<'a> {
    pub fb: &'a Framebuffer<'a>,
    pub cursor_x: u64,
    pub cursor_y: u64,
    pub current_fg: u32,
    pub current_bg: u32,
    pub line_height: u64, 
}

impl<'a> ConsoleBase<'a> {
    pub fn new(fb: &'a Framebuffer<'a>) -> Self {
        Self {
            fb,
            cursor_x: 20,
            cursor_y: 20,
            current_fg: 0xFFFFFF,
            current_bg: 0x000000,
            line_height: 9,
        }
    }

    // ---RENDERERS---

    pub fn render_buffer(&mut self, buffer: &crate::kernel::console::ring_buffer::RingBuffer) {

        unsafe {
            if crate::kernel::console::display_manager::NEEDS_FULL_REDRAW {
                self.clear();
                self.cursor_x = 20;
                self.cursor_y = 20;
                crate::kernel::console::display_manager::NEEDS_FULL_REDRAW = false;
            }
        }

        let buf = buffer.get_buf();
        let pos = buffer.get_pos();

        self.cursor_x = 20;
        self.cursor_y = 20;

        if buffer.is_wrapped() {
            self.render_bytes(&buf[pos..]);
            self.render_bytes(&buf[..pos]);
        } else {
            self.render_bytes(&buf[..pos]);
        }
    }

    fn render_bytes(&mut self, data: &[u8]) {
        let mut i = 0;
        while i < data.len() {
            if data[i] == b'^' && i + 2 < data.len() && data[i + 1] == b'&' {
                self.current_fg = self.parse_color(data[i + 2]);
                i += 3;
                continue;
            }

            let b = data[i];
            if b == 0 { 
                i += 1; 
                continue; 
            }

            match b {
                b'\n' => self.newline(),
                b'\r' => self.cursor_x = 20,
                0x08 => {
                    if self.cursor_x > 20 {
                        self.cursor_x -= 8;
                        let old_fg = self.current_fg;
                        self.current_fg = self.current_bg;
                        self.draw_char(b' '); 
                        self.current_fg = old_fg;
                    }
                },
                32..=126 => {
                    self.draw_char(b);
                    self.cursor_x += 8;
                }
                _ => {}
            }
            i += 1;
        }
    }

    // ---DRAWERS---

    pub fn draw_char(&self, c: u8) {

        let index = c as usize;
        if index >= 256 { return; }

        let fb_ptr = self.fb.addr() as *mut u32;
        let pitch = (self.fb.pitch() / 4) as usize;

        let rows_to_draw = if self.line_height < 10 { self.line_height } else { 10 };

        for row in 0..rows_to_draw {
            let curr_y = self.cursor_y + row;
            if curr_y >= self.fb.height() { break; }

            let row_data = if row < 8 { FONT[index * 8 + row as usize] } else { 0 };
            
            let row_addr = unsafe { fb_ptr.add(curr_y as usize * pitch + self.cursor_x as usize) };
            
            for col in 0..8 {
                if self.cursor_x + col as u64 >= self.fb.width() { break; }
                
                let bit = (row_data >> (7 - col)) & 1;
                let color = if bit == 1 { self.current_fg } else { self.current_bg };

                unsafe {
                    row_addr.add(col).write_volatile(color);
                }
            }
        }
    }

    // TODO: Delete the line in the ringbuffer not just clear.
    pub fn scroll(&mut self) {
        let fb_ptr = self.fb.addr() as *mut u32;
        let pitch = (self.fb.pitch() / 4) as usize;
        let height = self.fb.height() as usize;
        let width = self.fb.width() as usize;
        let line_h = self.line_height as usize;

        unsafe {
            for y in line_h..height {
                let src_row = fb_ptr.add(y * pitch);
                let dst_row = fb_ptr.add((y - line_h) * pitch);
                core::ptr::copy_nonoverlapping(src_row, dst_row, width);
            }

            for y in (height - line_h)..height {
                let row_addr = fb_ptr.add(y * pitch);
                for x in 0..width {
                    row_addr.add(x).write_volatile(self.current_bg);
                }
            }
        }
        self.cursor_y -= self.line_height;
    }

    // ---UTILITY---
    pub fn newline(&mut self) {
        self.cursor_x = 20;
        self.cursor_y += self.line_height;
        
        if self.cursor_y + self.line_height >= self.fb.height() - 10 {
            self.scroll();
        }
    }

    pub fn clear(&mut self) {
        let fb_ptr = self.fb.addr() as *mut u32;
        let pitch = (self.fb.pitch() / 4) as usize;
        let width = self.fb.width() as usize;
        let height = self.fb.height() as usize;

        for y in 0..height {
            let row_addr = unsafe { fb_ptr.add(y * pitch) };
            for x in 0..width {
                unsafe {
                    row_addr.add(x).write_volatile(self.current_bg);
                }
            }
        }
        
        self.cursor_x = 20;
        self.cursor_y = 20;
    }

    fn parse_color(&self, code: u8) -> u32 {
        match code {
            b'0' => 0x000000, b'1' => 0x0000AA, b'2' => 0x00AA00, b'3' => 0x00AAAA,
            b'4' => 0xAA0000, b'5' => 0xAA00AA, b'6' => 0xFFAA00, b'7' => 0xAAAAAA,
            b'8' => 0x555555, b'9' => 0x5555FF, b'a' => 0x55FF55, b'b' => 0x55FFFF,
            b'c' => 0xFF5555, b'd' => 0xFF55FF, b'e' => 0xFFFF55, b'f' => 0xFFFFFF,
            _ => 0xFFFFFF,
        }
    }

}
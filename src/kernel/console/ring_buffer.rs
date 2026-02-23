/*
 * DeepX OS Project
 * Ring buffer for kernel logging
 */

use spinning_top::Spinlock;
use core::fmt;

pub const LOG_BUFFER_SIZE: usize = 16 * 1024;

/// Rendszer logok (SafeConsole számára)
pub static LOG_BUFFER: Spinlock<RingBuffer> = Spinlock::new(RingBuffer::new());

/// TEST ONLY FOR THE NEW CONSOLE_BASE.RS FILE
/// TEST ONLY FOR THE NEW CONSOLE_BASE.RS FILE
/// TEST ONLY FOR THE NEW CONSOLE_BASE.RS FILE
/// TEST ONLY FOR THE NEW CONSOLE_BASE.RS FILE
pub static LOG_BUFFER_TEST: Spinlock<RingBuffer> = Spinlock::new(RingBuffer::new());

/// Shell kimenetek (KernelShell számára)
pub static SHELL_LOG_BUFFER: Spinlock<RingBuffer> = Spinlock::new(RingBuffer::new());

/// Statikus, heapless log buffer
pub struct RingBuffer {
    pub buf: [u8; LOG_BUFFER_SIZE],
    pub write_pos: usize,
    pub wrapped: bool,

}

impl fmt::Write for RingBuffer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.push_str(s);
        Ok(())
    }
}

impl RingBuffer {
    /// Const konstruktor, heap nélkül
    pub const fn new() -> Self {
        Self {
            buf: [0; LOG_BUFFER_SIZE],
            write_pos: 0,
            wrapped: false,
        }
    }


    pub fn is_wrapped(&self) -> bool { self.wrapped }
    pub fn get_pos(&self) -> usize { self.write_pos }
    pub fn get_buf(&self) -> &[u8] { &self.buf }

    /// Bájtok beszúrása
    pub fn push_bytes(&mut self, data: &[u8]) {
        for &b in data {
            self.buf[self.write_pos] = b;
            self.write_pos += 1;

            if self.write_pos >= LOG_BUFFER_SIZE {
                self.write_pos = 0;
                self.wrapped = true;
            }
        }
    }

    /// Szöveg beszúrása
    pub fn push_str(&mut self, s: &str) {
        self.push_bytes(s.as_bytes());
    }

    /// Kiolvasás egy slice-ba
    pub fn read_all(&self, out: &mut [u8]) -> usize {
        let mut idx = 0;

        if self.wrapped {
            let tail = &self.buf[self.write_pos..];
            let head = &self.buf[..self.write_pos];

            for &b in tail.iter().chain(head.iter()) {
                if idx >= out.len() { break; }
                out[idx] = b;
                idx += 1;
            }
        } else {
            for &b in &self.buf[..self.write_pos] {
                if idx >= out.len() { break; }
                out[idx] = b;
                idx += 1;
            }
        }

        idx
    }
    pub fn pop(&mut self) {
        if self.write_pos > 0 {
            self.write_pos -= 1;
            self.buf[self.write_pos] = 0; // Nullázzuk a helyét
        } else if self.wrapped {
            // Ha éppen a puffer elején vagyunk, de már körbeért (wrapped), 
            // akkor az utolsó indexre ugrunk vissza
            self.write_pos = LOG_BUFFER_SIZE - 1;
            self.buf[self.write_pos] = 0;
            // Opcionális: ha nagyon precízek akarunk lenni, 
            // itt a wrapped-et hamisra állíthatnánk, ha kiürül a puffer, 
            // de egy karakter törlésénél ez nem szükséges.
        }
    }

    pub fn clear(&mut self) {
        self.write_pos = 0;
        self.wrapped = false;
        // Opcionális: a memória lenullázása (biztonság kedvéért)
        for i in 0..LOG_BUFFER_SIZE {
            self.buf[i] = 0;
        }
    }
}


/// Heapless helper: formázott számok írása (itoa helyett)
pub fn push_u32(buf: &mut RingBuffer, mut n: u32) {
    let mut digits = [0u8; 10];
    let mut i = 0;

    if n == 0 {
        buf.push_bytes(b"0");
        return;
    }

    while n > 0 {
        digits[i] = b'0' + (n % 10) as u8;
        n /= 10;
        i += 1;
    }

    // fordítva írás
    for j in (0..i).rev() {
        buf.push_bytes(&[digits[j]]);
    }
}

/// Heapless helper: push line (newline hozzáadva)
pub fn push_line(buf: &mut RingBuffer, s: &str) {
    buf.push_str(s);
    buf.push_bytes(b"\n");
}

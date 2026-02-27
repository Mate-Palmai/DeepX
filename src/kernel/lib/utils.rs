/*
 * DeepX OS Project
 * Copyright (C) 2024-2026 - Máté Pálmai
 *
 * File: /src/kernel/lib/utils.rs
 * Description: Utility functions for kernel.
 */

pub fn u64_to_str(mut num: u64, buf: &mut [u8]) -> &str {
    if num == 0 {
        buf[0] = b'0';
        return unsafe { core::str::from_utf8_unchecked(&buf[0..1]) };
    }
    
    let mut i = buf.len();
    while num > 0 && i > 0 {
        i -= 1;
        buf[i] = (num % 10) as u8 + b'0';
        num /= 10;
    }
    
    unsafe { core::str::from_utf8_unchecked(&buf[i..]) }
}

#[allow(dead_code)]
pub fn u64_to_hex(mut num: u64, buf: &mut [u8]) -> &str {
    if num == 0 {
        buf[0] = b'0';
        return unsafe { core::str::from_utf8_unchecked(&buf[0..1]) };
    }

    let hex_chars = b"0123456789ABCDEF";
    let mut i = buf.len();
    while num > 0 && i > 0 {
        i -= 1;
        buf[i] = hex_chars[(num % 16) as usize];
        num /= 16;
    }

    unsafe { core::str::from_utf8_unchecked(&buf[i..]) }
}

pub fn u8_to_hex(value: u8, buffer: &mut [u8; 4]) -> &str {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    buffer[0] = HEX[(value >> 4) as usize];
    buffer[1] = HEX[(value & 0x0F) as usize];
    buffer[2] = b' ';
    buffer[3] = 0;
    
    core::str::from_utf8(&buffer[..3]).unwrap_or("??")
}


pub fn format_size(bytes: u64) -> crate::alloc::string::String {
    if bytes >= 1024 * 1024 {
        alloc::format!("{} MB", bytes / 1024 / 1024)
    } else if bytes >= 1024 {
        alloc::format!("{} KB", bytes / 1024)
    } else {
        alloc::format!("{} B", bytes)
    }
}
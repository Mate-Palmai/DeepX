/*
 * DeepX  Project
 * Copyright (C) 2024-2026 - Máté Pálmai
 *
 * File: /src/kernel/drivers/keyboard.rs
 * Description: PS/2 keyboard driver implementation.
 */

use x86_64::instructions::port::Port;
use spinning_top::Spinlock;
use alloc::collections::VecDeque;

const KEYBOARD_PORT: u16 = 0x60;

pub struct Keyboard;

// RAW SCANCODE QUEUE
pub static KEY_QUEUE: Spinlock<VecDeque<u8>> = Spinlock::new(VecDeque::new());

impl Keyboard {
    pub fn read_scancode() -> u8 {
        let mut port = Port::new(KEYBOARD_PORT);
        unsafe { port.read() }
    }

    pub fn scancode_to_char(scancode: u8) -> Option<char> {
        match scancode {
            0x1E => Some('a'),
            0x30 => Some('b'),
            0x2E => Some('c'),
            0x20 => Some('d'),
            0x12 => Some('e'),
            0x21 => Some('f'),
            0x22 => Some('g'),
            0x23 => Some('h'),
            0x17 => Some('i'),
            0x24 => Some('j'),
            0x25 => Some('k'),
            0x26 => Some('l'),
            0x32 => Some('m'),
            0x31 => Some('n'),
            0x18 => Some('o'),
            0x19 => Some('p'),
            0x10 => Some('q'),
            0x13 => Some('r'),
            0x1F => Some('s'),
            0x14 => Some('t'),
            0x16 => Some('u'),
            0x2F => Some('v'),
            0x11 => Some('w'),
            0x2D => Some('x'),
            0x15 => Some('y'),
            0x2C => Some('z'),
            0x02 => Some('1'),
            0x03 => Some('2'),
            0x04 => Some('3'),
            0x05 => Some('4'),
            0x06 => Some('5'),
            0x07 => Some('6'),
            0x08 => Some('7'),
            0x09 => Some('8'),
            0x0A => Some('9'),
            0x0B => Some('0'),
            0x1A => Some('['),
            0x1B => Some(']'),
            0x27 => Some(';'),
            0x28 => Some('\''),
            0x29 => Some('`'),
            0x2B => Some('\\'),
            0x33 => Some(','),
            0x34 => Some('.'),
            0x35 => Some('/'),
            0x0C => Some('-'),
            0x0D => Some('='),
            0x39 => Some(' '),
            0x1C => Some('\n'),
            0x0E => Some('\x08'),
            _ => None,
        }
    }
}
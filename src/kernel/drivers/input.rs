/*
 * DeepX OS Project
 * Copyright (C) 2024-2026 - Máté Pálmai
 */


use spinning_top::Spinlock;
use alloc::collections::VecDeque;
// drivers/input.rs
static INPUT_QUEUE: Spinlock<VecDeque<char>> = Spinlock::new(VecDeque::new());

pub fn init_input() {
    // Itt már nem static környezetben vagyunk, meghívhatjuk a foglalást
    let mut queue = INPUT_QUEUE.lock();
    *queue = VecDeque::with_capacity(256);
}

pub fn get_input_queue() -> &'static Spinlock<VecDeque<char>> {
    &INPUT_QUEUE
}

// A KernelShell használja ezt az input olvasáshoz
pub fn pop_key() -> Option<char> {
    // Itt a Shell fut, nyugodtan várhatunk egy picit a lakatra
    INPUT_QUEUE.lock().pop_front()
}

pub fn push_key(c: char) {
    // A megszakításkezelő hívja, itt fontos a gyorsaság
    if let Some(mut queue) = INPUT_QUEUE.try_lock() {
        queue.push_back(c);
    }
    // Ha nem sikerült (mert a shell épp olvas), a karakter elveszik, 
    // de legalább nem fagy le a kernel interrupt közben.
}




/*
 * DeepX Project
 * Copyright (C) 2024-2026 - Máté Pálmai
 */


use spinning_top::Spinlock;
use alloc::collections::VecDeque;
static INPUT_QUEUE: Spinlock<VecDeque<char>> = Spinlock::new(VecDeque::new());

pub fn init_input() {
    let mut queue = INPUT_QUEUE.lock();
    *queue = VecDeque::with_capacity(256);
}

pub fn get_input_queue() -> &'static Spinlock<VecDeque<char>> {
    &INPUT_QUEUE
}

pub fn pop_key() -> Option<char> {
    INPUT_QUEUE.lock().pop_front()
}

pub fn push_key(c: char) {
    if let Some(mut queue) = INPUT_QUEUE.try_lock() {
        queue.push_back(c);
    }
}




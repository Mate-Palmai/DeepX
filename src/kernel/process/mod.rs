/*
 * DeepX Project
 * Copyright (C) 2024-2026 - Máté Pálmai
 *
 * File: /src/kernel/process/mod.rs
 * Description: Process and task management module.
 */

use crate::kernel::process::task::Task;
use spinning_top::Spinlock;
use alloc::collections::VecDeque;
use crate::kernel::process::task::TaskState;
use crate::kernel::process::task::context_switch;
use alloc::format;
use core::sync::atomic::Ordering;

pub mod task;

pub static SCHEDULER_VERSION: &str = "v1";

pub static SCHEDULER: Spinlock<Scheduler> = Spinlock::new(Scheduler::new());

pub struct Scheduler {
    tasks: VecDeque<Task>,
    current_task_index: usize,
}

impl Scheduler {
    pub const fn new() -> Self {
        Self {
            tasks: VecDeque::new(),
            current_task_index: 0,
        }
    }

    pub fn add_task(&mut self, task: Task) {
        self.tasks.push_back(task);
    }

    pub fn get_tasks(&self) -> &VecDeque<Task> {
        &self.tasks
    }

    pub fn get_task_count(&self) -> usize {
        self.tasks.len()
    }

    pub fn remove_task(&mut self, id: u64) -> bool {
        if id == 0 || id == 2 {
            return false;
        }

        let mut target_idx = None;
        for (i, t) in self.tasks.iter().enumerate() {
            if t.id == id {
                target_idx = Some(i);
                break;
            }
        }

        if let Some(idx) = target_idx {
            self.tasks.remove(idx);
            if idx <= self.current_task_index && self.current_task_index > 0 {
                self.current_task_index -= 1;
            }
            return true;
        }
        false
    }
    

    pub fn block_task(&mut self, id: u64) -> bool {
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == id) {
            task.state = crate::kernel::process::task::TaskState::Blocked;
            return true;
        }
        false
    }

    pub fn resume_task(&mut self, id: u64) -> bool {
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == id) {
            task.state = crate::kernel::process::task::TaskState::Ready;
            return true;
        }
        false
    }

    pub fn yield_now() {
        unsafe {
            core::arch::asm!("int 0x20");
        }
    }

    pub fn sleep(ms: u64) {
        let freq = crate::arch::timer::tsc::get_tsc_frequency();
        if freq == 0 { return; }

        let ticks_to_sleep = (freq * ms) / 1000;
        let wakeup_at = crate::arch::timer::tsc::read_tsc() + ticks_to_sleep;

        {
            let mut sched = SCHEDULER.lock();
            let current_idx = sched.current_task_index;
            sched.tasks[current_idx].sleep_until = wakeup_at;
            sched.tasks[current_idx].state = TaskState::Blocked;
        }

        Self::yield_now();
    }
    

    pub fn schedule(&mut self) {
        if self.tasks.len() < 2 { return; }

        let now = crate::arch::timer::tsc::read_tsc();

        for task in self.tasks.iter_mut() {
            if task.state == TaskState::Blocked && task.sleep_until > 0 {
                if now >= task.sleep_until {
                    task.state = TaskState::Ready;
                    task.sleep_until = 0;
                }
            }
        }

        let old_idx = self.current_task_index;
        
        let mut next_idx = (old_idx + 1) % self.tasks.len();

        let mut found = false;
        for _ in 0..self.tasks.len() {
            if self.tasks[next_idx].state == TaskState::Ready {
                found = true;
                break;
            }
            next_idx = (next_idx + 1) % self.tasks.len();
        }

        if !found { return; }

        while self.tasks[next_idx].state == TaskState::Blocked {
            next_idx = (next_idx + 1) % self.tasks.len();
            if next_idx == old_idx { return; }
        }

        if old_idx == next_idx { return; }

        if self.tasks[old_idx].state == TaskState::Running {
            self.tasks[old_idx].state = TaskState::Ready;
        }
        self.tasks[next_idx].state = TaskState::Running;
        self.current_task_index = next_idx;

        let old_rsp_ptr = &mut self.tasks[old_idx].stack_pointer as *mut u64;
        let new_rsp = self.tasks[next_idx].stack_pointer;

        unsafe {
            SCHEDULER.force_unlock();
            context_switch(old_rsp_ptr, new_rsp);
        }
    }

    fn generate_unique_id(&self, requested_id: Option<u64>) -> u64 {
        if let Some(id) = requested_id {
            let exists = self.tasks.iter().any(|t| t.id == id);
            if !exists {
                let current_next = crate::kernel::process::task::NEXT_ID.load(Ordering::SeqCst);
                if id >= current_next {
                    crate::kernel::process::task::NEXT_ID.store(id + 1, Ordering::SeqCst);
                }
                return id;
            }
        }
        
        crate::kernel::process::task::NEXT_ID.fetch_add(1, Ordering::SeqCst)
    }

    pub fn spawn(&mut self, entry_point: u64, name: Option<&str>, id: Option<u64>) {
        let final_id = self.generate_unique_id(id);
        let task = Task::new(final_id, entry_point, name);
        self.add_task(task);
    }
}
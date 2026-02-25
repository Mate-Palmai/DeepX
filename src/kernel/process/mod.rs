/* /src/kernel/process/mod.rs */

use crate::kernel::process::task::Task;
use spinning_top::Spinlock;
use alloc::collections::VecDeque;
use alloc::format; // Szükségünk lesz a format! makróra

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
    

    pub fn schedule(&mut self) {


        if self.tasks.len() < 2 { return; }

        let old_idx = self.current_task_index;
        let new_idx = (old_idx + 1) % self.tasks.len();
        self.current_task_index = new_idx;


        let old_rsp_ptr = &mut self.tasks[old_idx].stack_pointer as *mut u64;
        let new_rsp = self.tasks[new_idx].stack_pointer;

        unsafe {
        // Ha spinning_top-ot használsz, ez felszabadítja a lakatot
        // anélkül, hogy megvárná a változó élettartamának végét.
        crate::kernel::process::SCHEDULER.force_unlock();
        }

        unsafe {
            crate::kernel::process::task::context_switch(old_rsp_ptr, new_rsp);
        }
    }
}
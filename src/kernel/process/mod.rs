/* /src/kernel/process/mod.rs */

use crate::kernel::process::task::Task;
use spinning_top::Spinlock;
use alloc::collections::VecDeque;
use crate::kernel::process::task::TaskState;
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

    pub fn get_tasks(&self) -> &VecDeque<Task> {
        &self.tasks
    }

    pub fn get_task_count(&self) -> usize {
        self.tasks.len()
    }

    pub fn remove_task(&mut self, id: u64) -> bool {
        // Nem engedjük megölni a Kernelt (0) vagy a Shellt (2) - öngyilkosság ellen
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
            // Ha a törölt taszk az aktuális előtt volt, korrigáljuk az indexet
            if idx <= self.current_task_index && self.current_task_index > 0 {
                self.current_task_index -= 1;
            }
            return true;
        }
        false
    }
    

    pub fn schedule(&mut self) {
    if self.tasks.len() < 2 { return; }

    let old_idx = self.current_task_index;
    // 1. A régi taszkot visszatesszük Ready állapotba
    self.tasks[old_idx].state = TaskState::Ready;

    // 2. Index léptetés
    let new_idx = (old_idx + 1) % self.tasks.len();
    self.current_task_index = new_idx;
    
    // 3. Az új taszkot átállítjuk Running-ra
    self.tasks[new_idx].state = TaskState::Running;

    let old_rsp_ptr = &mut self.tasks[old_idx].stack_pointer as *mut u64;
    let new_rsp = self.tasks[new_idx].stack_pointer;

    unsafe {
        crate::kernel::process::SCHEDULER.force_unlock();
        crate::kernel::process::task::context_switch(old_rsp_ptr, new_rsp);
    }
}
}
/*
 * DeepX Project - SMP Scheduler v2
 */

use crate::kernel::process::task::{Task, TaskState, context_switch};
use spinning_top::Spinlock;
use alloc::collections::VecDeque;
use core::sync::atomic::Ordering;

pub mod task;

pub static SCHEDULER_VERSION: &str = "v2-SMP";
const MAX_CORES: usize = 8;

pub static SCHEDULER: Spinlock<Scheduler> = Spinlock::new(Scheduler::new());

static mut FALLBACK_RSP_STORAGE: [u64; MAX_CORES] = [0; MAX_CORES];

pub struct Scheduler {
    tasks: VecDeque<Task>,
    cpu_tasks: [u64; MAX_CORES],
}

impl Scheduler {
    pub const fn new() -> Self {
        Self {
            tasks: VecDeque::new(),
            cpu_tasks: [u64::MAX; MAX_CORES],
        }
    }

    pub fn get_tasks(&self) -> &VecDeque<Task> { &self.tasks }
    pub fn get_task_count(&self) -> usize { self.tasks.len() }

    pub fn add_task(&mut self, task: Task) {
        self.tasks.push_back(task);
    }

    pub fn remove_task(&mut self, id: u64) -> bool {
        if id == 0 || id == 2 { return false; }
        if let Some(idx) = self.tasks.iter().position(|t| t.id == id) {
            self.tasks.remove(idx);
            return true;
        }
        false
    }

    pub fn block_task(&mut self, id: u64) -> bool {
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == id) {
            task.state = TaskState::Blocked;
            return true;
        }
        false
    }

    pub fn resume_task(&mut self, id: u64) -> bool {
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == id) {
            task.state = TaskState::Ready;
            return true;
        }
        false
    }

    pub fn get_cpu_tasks(&self) -> [u64; MAX_CORES] {
        self.cpu_tasks
    }

    // --- Scheduling logic ---
    pub fn yield_now() {


        unsafe { core::arch::asm!("int 0x20"); }

       
    }

    pub fn schedule(&mut self) {
        if self.tasks.is_empty() { return; }

        let cpu_id = crate::kernel::cpu::get_id() as usize;
        let now = crate::arch::x86::timer::tsc::read_tsc();

        for task in self.tasks.iter_mut() {
            if task.state == TaskState::Blocked && task.sleep_until > 0 {
                if now >= task.sleep_until {
                    task.state = TaskState::Ready;
                    task.sleep_until = 0;
                }
            }
        }

        let current_task_id = self.cpu_tasks[cpu_id];
        let old_idx = self.tasks.iter().position(|t| t.id == current_task_id);

        let start_search = old_idx.map(|i| i + 1).unwrap_or(0);
        let mut next_idx = None;

        for i in 0..self.tasks.len() {
            let idx = (start_search + i) % self.tasks.len();
            let t = &self.tasks[idx];
            
            let is_running_elsewhere = self.cpu_tasks.iter().enumerate().any(|(cpu, &id)| {
                cpu != cpu_id && id == t.id && id != u64::MAX
            });

            if t.state == TaskState::Ready && !is_running_elsewhere {
                next_idx = Some(idx);
                break;
            }
        }

        if let Some(n_idx) = next_idx {
            if Some(n_idx) != old_idx {
                if let Some(o_idx) = old_idx {
                    if self.tasks[o_idx].state == TaskState::Running {
                        self.tasks[o_idx].state = TaskState::Ready;
                    }
                }

                self.tasks[n_idx].state = TaskState::Running;
                let new_id = self.tasks[n_idx].id;
                let new_rsp = self.tasks[n_idx].stack_pointer;
                self.cpu_tasks[cpu_id] = new_id;
--
                let old_rsp_ptr = if let Some(o_idx) = old_idx {
                    &mut self.tasks[o_idx].stack_pointer as *mut u64
                } else {
                    unsafe { &mut FALLBACK_RSP_STORAGE[cpu_id] as *mut u64 }
                };

                unsafe {
                    SCHEDULER.force_unlock(); 
                    context_switch(old_rsp_ptr, new_rsp);
                }
            }
        }
    }

    pub fn spawn(&mut self, entry_point: u64, name: Option<&str>, id: Option<u64>) {
        let final_id = self.generate_unique_id(id);
        let task = Task::new(final_id, entry_point, name);
        self.add_task(task);
    }

    fn generate_unique_id(&self, requested_id: Option<u64>) -> u64 {
        if let Some(id) = requested_id {
            if !self.tasks.iter().any(|t| t.id == id) { return id; }
        }
        crate::kernel::process::task::NEXT_ID.fetch_add(1, Ordering::SeqCst)
    }
}
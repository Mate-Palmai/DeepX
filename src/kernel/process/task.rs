/*
 * DeepX Project - Task v2
 */

use alloc::string::{String, ToString};
use alloc::format;
use core::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    Ready,     
    Running,  
    Blocked,  
}

pub static ALLOCATED_TASK_MEMORY: AtomicUsize = AtomicUsize::new(0);
pub static NEXT_ID: AtomicU64 = AtomicU64::new(1);

pub struct Task {
    pub id: u64,
    pub name: String,
    pub stack_pointer: u64, 
    pub state: TaskState,
    pub stack_bottom: u64,
    pub stack_top: u64,
    pub sleep_until: u64,
}

extern "C" {
    pub fn context_switch(old_rsp: *mut u64, new_rsp: u64);
}

core::arch::global_asm!(r#"
.global context_switch
context_switch:
    push rbx
    push rbp
    push r12
    push r13
    push r14
    push r15
    mov [rdi], rsp
    mov rsp, rsi
    pop r15
    pop r14
    pop r13
    pop r12
    pop rbp
    pop rbx
    ret
"#);

impl Task {
    pub fn new_kernel_task() -> Self {
        Self {
            id: 0,
            name: "KernelTask".into(),
            stack_pointer: 0, 
            state: TaskState::Running,
            stack_bottom: 0,
            stack_top: 0,
            sleep_until: 0,
        }
    }

    pub fn new(id: u64, entry_point: u64, name: Option<&str>) -> Self {
        let task_name = name.map(|s| s.to_string())
                            .unwrap_or_else(|| format!("task_0x{:x}", entry_point));

        let stack_size = 4096 * 4;
        let layout = core::alloc::Layout::from_size_align(stack_size, 16).unwrap();
        ALLOCATED_TASK_MEMORY.fetch_add(stack_size, Ordering::SeqCst);
        
        unsafe {
            let stack_ptr = alloc::alloc::alloc(layout);
            let stack_top = stack_ptr as u64 + stack_size as u64;
            let mut sp = stack_top;

            sp -= 8;
            *(sp as *mut u64) = entry_point;

            for _ in 0..6 { 
                sp -= 8;
                *(sp as *mut u64) = 0;
            }
            
            Self {
                id,
                name: task_name,
                stack_pointer: sp,
                state: TaskState::Ready,
                stack_bottom: stack_ptr as u64,
                stack_top,
                sleep_until: 0,
            }
        }
    }
}

impl Drop for Task {
    fn drop(&mut self) {
        if self.stack_bottom != 0 {
            let stack_size = 4096 * 4;
            let layout = core::alloc::Layout::from_size_align(stack_size, 16).unwrap();
            unsafe { alloc::alloc::dealloc(self.stack_bottom as *mut u8, layout); }
            ALLOCATED_TASK_MEMORY.fetch_sub(stack_size, Ordering::SeqCst);
        }
    }
}
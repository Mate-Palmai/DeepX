/*
 * DeepX Project
 * Copyright (C) 2024-2026 - Máté Pálmai
 *
 * File: /src/kernel/process/task.rs
 * Description: Task management and context switching logic.
 */

use alloc::alloc::handle_alloc_error;
use alloc::string::{String, ToString}; 
use alloc::format;                   
use core::sync::atomic::{AtomicU64, Ordering};


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    Ready,     
    Running,  
    Blocked,  
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct TaskContext {
    r15: u64, r14: u64, r13: u64, r12: u64,
    rbp: u64, rbx: u64,
    rip: u64, 
}

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

pub struct Task {
    pub id: u64,
    pub name: String,
    pub stack_pointer: u64, 
    pub state: TaskState,
    pub stack_bottom: u64,
    pub stack_top: u64,
}

use alloc::vec::Vec;
use alloc::vec;

use core::arch::global_asm;

global_asm!(r#"
.global context_switch
context_switch:
    # 1. Mentjük a régi task regisztereit a stackre
    push rbx
    push rbp
    push r12
    push r13
    push r14
    push r15

    # 2. Elmentjük a jelenlegi stack pointert (RSP) a régi taskba
    # RDI az első argumentum (old_rsp_ptr)
    mov [rdi], rsp

    # 3. Betöltjük az ÚJ task stack pointerét
    # RSI a második argumentum (new_rsp)
    mov rsp, rsi

    # 4. Visszatöltjük az új task regisztereit a stackről
    pop r15
    pop r14
    pop r13
    pop r12
    pop rbp
    pop rbx

    # 5. Visszaugrunk az új task RIP címére (ami a stack tetején van)
    ret
"#);

extern "C" {
    pub fn context_switch(old_rsp: *mut u64, new_rsp: u64);
}

impl Task {

    pub fn new_kernel_task() -> Self {
        Self {
            id: 0,
            name: "KernelTask".into(),
            stack_pointer: 0, 
            state: TaskState::Running,
            stack_bottom: 0,
            stack_top: 0,
        }
    }

    pub fn new(id: Option<u64>, entry_point: u64, name: Option<&str>) -> Self {
        // ID: ha nincs megadva, generálunk egy újat az atomi számlálóból
        let final_id = id.unwrap_or_else(|| NEXT_ID.fetch_add(1, Ordering::SeqCst));
        
        // NÉV: ha nincs megadva, a memóriacím alapján nevezzük el
        let task_name = name.map(|s| s.to_string())
                            .unwrap_or_else(|| format!("task_0x{:x}", entry_point));

        let stack_size = 4096 * 4; // 16 KB stack
        let layout = core::alloc::Layout::from_size_align(stack_size, 16).unwrap();
        
        unsafe {
            let stack_ptr = alloc::alloc::alloc(layout);
            if stack_ptr.is_null() { alloc::alloc::handle_alloc_error(layout); }

            let stack_top = stack_ptr as u64 + stack_size as u64;
            let mut sp = stack_top;

            // Kontextus felépítése a stacken a context_switch számára
            sp -= 8;
            *(sp as *mut u64) = entry_point; // RIP visszatérési címnek

            for _ in 0..6 { // r15, r14, r13, r12, rbp, rbx
                sp -= 8;
                *(sp as *mut u64) = 0;
            }

            Self {
                id: final_id,
                name: task_name,
                stack_pointer: sp,
                state: TaskState::Ready,
                stack_bottom: stack_ptr as u64,
                stack_top,
            }
        }
    }
}



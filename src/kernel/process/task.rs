// src/kernel/process/task.rs

use alloc::alloc::handle_alloc_error;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    Ready,      // Futásra kész
    Running,    // Éppen fut
    Blocked,    // Várakozik
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct TaskContext {
    // Callee-saved regiszterek, amiket mentenünk kell
    r15: u64, r14: u64, r13: u64, r12: u64,
    rbp: u64, rbx: u64,
    rip: u64, // Instruction pointer (hová térünk vissza)
}

pub struct Task {
    pub id: u64,
    pub stack_pointer: u64, // RSP értéke a váltáskor
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
            stack_pointer: 0, // Ezt a context_switch fogja kitölteni az első mentésnél
            state: TaskState::Running,
            stack_bottom: 0, // A fő stack címeit a Limine/Bootloader már beállította
            stack_top: 0,
        }
    }

    pub fn new(id: u64, entry_point: u64) -> Self {
        let stack_size = 4096 * 4;
        let layout = core::alloc::Layout::from_size_align(stack_size, 16).unwrap();
        let stack_ptr = unsafe { alloc::alloc::alloc(layout) };
        if stack_ptr.is_null() { unsafe { handle_alloc_error(layout); } }

        let stack_top = stack_ptr as u64 + stack_size as u64;
        let mut sp = stack_top;

        // --- Ez a rész kritikus ---
        // A context_switch 'ret'-tel indul, de az új tasknak 
        // engedélyeznie kell a megszakításokat.
        
        // 1. RIP (visszatérési cím)
        sp -= 8;
        unsafe { *(sp as *mut u64) = entry_point; }

        // 2. Regiszterek (rbx, rbp, r12, r13, r14, r15)
        for _ in 0..6 {
            sp -= 8;
            unsafe { *(sp as *mut u64) = 0; }
        }

        Self {
            id,
            stack_pointer: sp,
            state: TaskState::Ready,
            stack_bottom: stack_ptr as u64,
            stack_top,
        }
    }
}



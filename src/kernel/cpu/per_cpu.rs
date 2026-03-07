use alloc::vec::Vec;

#[repr(C)] 
pub struct PerCpu {
    pub self_ptr: u64,      
    pub kernel_stack: u64,  
    pub apic_id: u8,
    pub is_bsp: bool,
    pub current_task_id: u32,
    pub ready_queue: Vec<u32>,

    
}

// pub static mut CPUS: [Option<PerCpu>; 256] = {
//     const NONE: Option<PerCpu> = None;
//     [NONE; 256]
// };
pub static mut CPUS: [Option<*mut PerCpu>; 256] = [None; 256];

impl PerCpu {
    pub fn new(apic_id: u8, is_bsp: bool) -> Self {
        Self {
            self_ptr: 0,
            apic_id,
            current_task_id: 0,
            ready_queue: Vec::new(),
            is_bsp,
            kernel_stack: 0,
        }
    }
}

pub fn get_current() -> *mut PerCpu {
    let ptr: u64;
    unsafe {
        core::arch::asm!("mov {}, gs:[0]", out(reg) ptr);
    }
    ptr as *mut PerCpu
}
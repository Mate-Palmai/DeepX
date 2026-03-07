pub mod per_cpu;

use alloc::format;
use crate::arch::x86::cpu::set_gs_base;
use crate::SMP_REQUEST;

pub unsafe fn init_smp() {
    use crate::kernel::acpi::tables::{CPU_COUNT, LAPIC_IDS};
    use crate::kernel::cpu::per_cpu::{PerCpu, CPUS};
    use alloc::boxed::Box;

    let bsp_apic_id = get_bsp_apic_id();

    for i in 0..unsafe { CPU_COUNT } as usize {
        let apic_id = unsafe { LAPIC_IDS[i] };
        let is_bsp = apic_id == bsp_apic_id;
        
        let mut cpu_box = Box::new(PerCpu::new(apic_id, is_bsp));
        
        let stack_size = 32 * 1024;
        let layout = core::alloc::Layout::from_size_align(stack_size, 16).unwrap();
        let stack_ptr = alloc::alloc::alloc(layout);
        
        if stack_ptr.is_null() {
            panic!("Failed to allocate stack for CPU {}", apic_id);
        }

        cpu_box.kernel_stack = stack_ptr as u64 + stack_size as u64;

        let cpu_ptr = Box::into_raw(cpu_box);
        (*cpu_ptr).self_ptr = cpu_ptr as u64;



        unsafe {
            CPUS[apic_id as usize] = Some(cpu_ptr);
        }

        if is_bsp {
            set_gs_base(cpu_ptr as u64);
            crate::kernel::console::LOGGER.ok("CPU: GS base set for BSP.");
        }
    }

    if let Some(smp_response) = SMP_REQUEST.get_response() {
        let bsp_lapic_id = smp_response.bsp_lapic_id(); 

        for cpu_info in smp_response.cpus() {
            if cpu_info.lapic_id == bsp_lapic_id {
                continue;
            }
            
            cpu_info.goto_address.write(ap_main);
        }
    }
    
    crate::kernel::console::LOGGER.ok(&format!("CPU: Detected {} CPU cores.", unsafe { CPU_COUNT }));
}

fn get_bsp_apic_id() -> u8 {
    let res = unsafe { core::arch::x86_64::__cpuid(1) };
    ((res.ebx >> 24) & 0xFF) as u8
}

pub unsafe extern "C" fn ap_main(info: &limine::smp::Cpu) -> ! {
    let apic_id = info.lapic_id as u8;

    crate::arch::x86::idt::load(); 

    if let Some(ptr) = crate::kernel::cpu::per_cpu::CPUS[apic_id as usize] {
        crate::arch::x86::cpu::set_gs_base((*ptr).self_ptr);
    }

    let lapic = crate::arch::x86::apic::LocalApic::new(crate::arch::x86::apic::get_lapic_base());
    lapic.init();

    crate::arch::x86::timer::lapic::init_ap();

    core::arch::asm!("sti");

    loop {
        let mut sched = crate::kernel::process::SCHEDULER.lock();
        sched.schedule();
        drop(sched);
        
        core::arch::asm!("hlt"); 
    }
}

pub fn get_id() -> u8 {
    let res = unsafe { core::arch::x86_64::__cpuid(1) };
    ((res.ebx >> 24) & 0xFF) as u8
}
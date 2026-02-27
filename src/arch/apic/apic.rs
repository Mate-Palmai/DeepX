pub fn has_apic() -> bool {
    let edx: u32;
    unsafe {
        core::arch::asm!(
            "push rbx",  
            "cpuid",        
            "pop rbx",      
            inout("eax") 1 => _,
            out("ecx") _,
            out("edx") edx,
            clobber_abi("C"), 
        );
    }
    (edx & (1 << 9)) != 0

}
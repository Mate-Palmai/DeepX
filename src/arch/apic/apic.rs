pub fn has_apic() -> bool {
    let edx: u32;
    unsafe {
        core::arch::asm!(
            "push rbx",      // Mentjük az rbx értékét a stackre
            "cpuid",         // EAX=1 hívás (alapból 1-nek kell lennie az eax-nek)
            "pop rbx",       // Visszaállítjuk az rbx-et a stackről
            inout("eax") 1 => _,
            out("ecx") _,
            out("edx") edx,
            clobber_abi("C"), // Jelzi, hogy a regiszterek változhatnak
        );
    }
    // Az EDX 9. bitje (0-tól számolva) jelzi az APIC jelenlétét
    (edx & (1 << 9)) != 0

}
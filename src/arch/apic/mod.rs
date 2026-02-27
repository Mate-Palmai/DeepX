pub mod local;
pub mod io;
pub mod apic;

use crate::arch::pic::PICS;
pub use local::LocalApic;
pub use io::IoApic;
pub use self::apic::has_apic;

static mut APIC_ENABLED: bool = false;
const LAPIC_DEFAULT_BASE: usize = 0xFEE00000;

pub fn is_active() -> bool {
    unsafe { APIC_ENABLED }
}

pub fn get_lapic_base() -> usize {
    LAPIC_DEFAULT_BASE
}

pub fn send_eoi() {
    if is_active() {
        let lapic = unsafe { LocalApic::new(get_lapic_base()) };
        lapic.eoi();
    }
}

pub unsafe fn init() {
    {
        let mut pics = PICS.lock();
        pics.disable_pic();
    }

    let lapic = LocalApic::new(LAPIC_DEFAULT_BASE);
    lapic.init();

    let ioapic = IoApic::new(0xFEC00000);
    ioapic.init();

    ioapic.set_irq(0, 32);
    ioapic.set_irq(1, 33);
    ioapic.set_irq(12, 44);

    crate::arch::idt::set_apic_mode(true);
    
    APIC_ENABLED = true;

    crate::kernel::console::LOGGER.ok("APIC initialized and IRQs remapped");
}
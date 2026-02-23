use bitflags::bitflags;
use crate::kernel::mem::pmm;

pub const HHDM_OFFSET: u64 = 0xffff800000000000; 

bitflags! {
    #[derive(Clone, Copy, Debug)]
    pub struct PageTableFlags: u64 {
        const PRESENT = 1 << 0;
        const WRITABLE = 1 << 1;
        const USER_ACCESSIBLE = 1 << 2;
        const WRITE_THROUGH = 1 << 3;
        const NO_CACHE = 1 << 4;
        const ACCESSED = 1 << 5;
        const DIRTY = 1 << 6;
        const HUGE_PAGE = 1 << 7;       // <--- EZ OKOZZA A GONDOT
        const GLOBAL = 1 << 8;
        const NO_EXECUTE = 1 << 63;
    }
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct PageTableEntry(pub u64);

impl PageTableEntry {
    pub const ADDR_MASK: u64 = 0x000f_ffff_ffff_f000;

    pub fn set_addr(&mut self, phys: u64, flags: PageTableFlags) {
        self.0 = (phys & Self::ADDR_MASK) | flags.bits();
    }

    pub fn addr(&self) -> u64 {
        self.0 & Self::ADDR_MASK
    }

    pub fn flags(&self) -> PageTableFlags {
        PageTableFlags::from_bits_truncate(self.0)
    }
}

#[repr(C, align(4096))]
pub struct PageTable {
    pub entries: [PageTableEntry; 512],
}

impl PageTable {
    pub fn zero(&mut self) {
        for entry in self.entries.iter_mut() {
            entry.0 = 0;
        }
    }
}

pub struct VirtAddr(pub u64);

impl VirtAddr {
    pub fn p4_index(&self) -> usize { ((self.0 >> 39) & 0x1ff) as usize }
    pub fn p3_index(&self) -> usize { ((self.0 >> 30) & 0x1ff) as usize }
    pub fn p2_index(&self) -> usize { ((self.0 >> 21) & 0x1ff) as usize }
    pub fn p1_index(&self) -> usize { ((self.0 >> 12) & 0x1ff) as usize }
}

pub struct Mapper {
    p4: &'static mut PageTable,
}

impl Mapper {
    pub unsafe fn new() -> Self {
        let p4_addr: u64;
        core::arch::asm!("mov {}, cr3", out(reg) p4_addr);
        let p4_virt = (p4_addr + HHDM_OFFSET) as *mut PageTable;
        Self { p4: &mut *p4_virt }
    }

    pub fn map_to(&mut self, virt: VirtAddr, phys: u64, flags: PageTableFlags) {
        let p4_idx = virt.p4_index();
        let p3_idx = virt.p3_index();
        let p2_idx = virt.p2_index();
        let p1_idx = virt.p1_index();

        let entry = &mut self.p4.entries[p4_idx];
        let p3 = Self::get_or_create_table_static(entry);

        let entry = &mut p3.entries[p3_idx];
        let p2 = Self::get_or_create_table_static(entry);

        let entry = &mut p2.entries[p2_idx];
        let p1 = Self::get_or_create_table_static(entry);

        p1.entries[p1_idx].set_addr(phys, flags | PageTableFlags::PRESENT);

        unsafe {
            core::arch::asm!("invlpg [{}]", in(reg) virt.0);
        }
    }

    fn get_or_create_table_static(entry: &mut PageTableEntry) -> &'static mut PageTable {
        // HA HUGE_PAGE van itt, kényszerítsük az átalakítást rendes táblává!
        if entry.flags().contains(PageTableFlags::PRESENT) && !entry.flags().contains(PageTableFlags::HUGE_PAGE) {
            if !entry.flags().contains(PageTableFlags::USER_ACCESSIBLE) {
                let new_flags = entry.flags() | PageTableFlags::USER_ACCESSIBLE | PageTableFlags::WRITABLE;
                entry.set_addr(entry.addr(), new_flags);
            }
            unsafe { &mut *((entry.addr() + HHDM_OFFSET) as *mut PageTable) }
        } else {
            // Új táblát foglalunk (akkor is, ha korábban HUGE_PAGE volt ott)
            let new_frame = pmm::alloc_frame().expect("PMM Exhausted");
            let new_table_virt = (new_frame + HHDM_OFFSET) as *mut PageTable;
            unsafe {
                (*new_table_virt).zero();
                // Fontos: a HUGE_PAGE bit itt biztosan nincs beállítva
                entry.set_addr(new_frame, PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE);
                &mut *new_table_virt
            }
        }
    }

    pub fn debug_walk(&self, virt: u64) {
    use alloc::format;
    let v = VirtAddr(virt);
    let log = &crate::kernel::console::LOGGER;

    log.info(&format!("--- Detailed Walk: 0x{:016X} ---", virt));

    // LEVEL 4: PML4
    let p4_entry = self.p4.entries[v.p4_index()];
    log.info(&format!("L4 Index: {} | Entry: 0x{:X} | Flags: {:?}", 
        v.p4_index(), p4_entry.addr(), p4_entry.flags()));
    
    if !p4_entry.flags().contains(PageTableFlags::PRESENT) {
        log.error("L4: Table not present!");
        return;
    }

    // LEVEL 3: PDPT
    let p3 = unsafe { &*((p4_entry.addr() + HHDM_OFFSET) as *const PageTable) };
    let p3_entry = p3.entries[v.p3_index()];
    log.info(&format!("  L3 Index: {} | Entry: 0x{:X} | Flags: {:?}", 
        v.p3_index(), p3_entry.addr(), p3_entry.flags()));

    if !p3_entry.flags().contains(PageTableFlags::PRESENT) {
        log.error("  L3: Table not present!");
        return;
    }

    // LEVEL 2: Page Directory
    let p2 = unsafe { &*((p3_entry.addr() + HHDM_OFFSET) as *const PageTable) };
    let p2_entry = p2.entries[v.p2_index()];
    log.info(&format!("    L2 Index: {} | Entry: 0x{:X} | Flags: {:?}", 
        v.p2_index(), p2_entry.addr(), p2_entry.flags()));

    if p2_entry.flags().contains(PageTableFlags::HUGE_PAGE) {
        log.warn("    L2: HUGE_PAGE (2MB) detected. Stopping walk.");
        return;
    }

    if !p2_entry.flags().contains(PageTableFlags::PRESENT) {
        log.error("    L2: Table not present!");
        return;
    }

    // LEVEL 1: Page Table
    let p1 = unsafe { &*((p2_entry.addr() + HHDM_OFFSET) as *const PageTable) };
    let p1_entry = p1.entries[v.p1_index()];
    
    log.ok(&format!("      L1 Index: {} | Phys Frame: 0x{:X} | Flags: {:?}", 
        v.p1_index(), p1_entry.addr(), p1_entry.flags()));

    // Biztonsági ellenőrzés Ring 3-hoz
    if !p1_entry.flags().contains(PageTableFlags::USER_ACCESSIBLE) {
        log.error("      CRITICAL: USER_ACCESSIBLE bit is MISSING at L1!");
    }
}
}
use alloc::format;
use core::str;

#[repr(C, packed)]
pub struct SdtHeader {
    pub signature: [u8; 4],
    pub length: u32,
    pub revision: u8,
    pub checksum: u8,
    pub oem_id: [u8; 6],
    pub oem_table_id: [u8; 8],
    pub oem_revision: u32,
    pub creator_id: u32,
    pub creator_revision: u32,
}

pub static mut PM1A_CNT_BLK: u32 = 0;
pub static mut PM1B_CNT_BLK: u32 = 0;
pub static mut SMI_CMD: u32 = 0;
pub static mut ACPI_ENABLE: u8 = 0;

// RSDT (32 bit)
pub fn parse_rsdt(rsdt_addr: u64) {
    let header = unsafe { &*(rsdt_addr as *const SdtHeader) };
    let entry_count = (header.length as usize - core::mem::size_of::<SdtHeader>()) / 4;
    let entries_ptr = (rsdt_addr + core::mem::size_of::<SdtHeader>() as u64) as *const u32;

    for i in 0..entry_count {
        let table_ptr = unsafe { *entries_ptr.add(i) } as u64;
        process_table(table_ptr);
    }
}

// XSDT (64 bit)
pub fn parse_xsdt(xsdt_addr: u64) {
    let header = unsafe { &*(xsdt_addr as *const SdtHeader) };
    let entry_count = (header.length as usize - core::mem::size_of::<SdtHeader>()) / 8;
    let entries_ptr = (xsdt_addr + core::mem::size_of::<SdtHeader>() as u64) as *const u64;

    for i in 0..entry_count {
        let table_ptr = unsafe { *entries_ptr.add(i) };
        process_table(table_ptr);
    }
}


fn process_table(ptr: u64) {
    let header = unsafe { &*(ptr as *const SdtHeader) };
    let sig = str::from_utf8(&header.signature).unwrap_or("????");
    
    #[cfg(feature = "dev")]
    crate::kernel::console::LOGGER.info(&format!("ACPI: Found table [{}] at {:#x}", sig, ptr));

    if sig == "FACP" {
        let fadt = unsafe { &*(ptr as *const Fadt) };
        unsafe {
            PM1A_CNT_BLK = fadt.pm1a_control_block;
            PM1B_CNT_BLK = fadt.pm1b_control_block;
            SMI_CMD = fadt.smi_command_port;
            ACPI_ENABLE = fadt.acpi_enable;
            
            #[cfg(feature = "dev")]
            crate::kernel::console::LOGGER.ok(&format!("ACPI: FADT data saved (PM1a: {:#x}, SMI: {:#x})", PM1A_CNT_BLK, SMI_CMD));
        }
    }
}

#[repr(C, packed)]
pub struct Fadt {
    pub header: SdtHeader,
    pub firmware_ctrl: u32,
    pub dsdt: u32,
    _reserved: u8,
    pub preferred_pm_profile: u8,
    pub sci_interrupt: u16,
    pub smi_command_port: u32,
    pub acpi_enable: u8,
    pub acpi_disable: u8,
    pub s4bios_req: u8,
    pub pstate_control: u8,
    pub pm1a_event_block: u32,
    pub pm1b_event_block: u32,
    pub pm1a_control_block: u32, 
    pub pm1b_control_block: u32,
    pub pm2_control_block: u32,
    pub pm_timer_block: u32,
}
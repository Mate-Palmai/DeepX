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

// RSDT (32 bit)
pub fn parse_rsdt(rsdt_addr: u64) {
    let header = unsafe { &*(rsdt_addr as *const SdtHeader) };
    
    // RSDT 4 byte (u32) 
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
    
    // XSDT 8 byte (u64)
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
    crate::kernel::console::LOGGER.info(&format!("ACPI: Found table [{}] at {:#x}", sig, ptr));
}
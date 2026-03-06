pub mod rsdp;
pub mod tables;

pub fn init() {
    let response = crate::RSDP_REQUEST.get_response()
        .expect("ACPI: Bootloader could not find RSDP!");

    let rsdp_ptr = response.address() as *const rsdp::Rsdp;

    unsafe {
        if !(*rsdp_ptr).is_valid() {
            crate::kernel::console::LOGGER.error("ACPI: Invalid RSDP signature!");
            return;
        }

        let revision = (*rsdp_ptr).revision;
        let xsdt_addr = (*rsdp_ptr).xsdt_address;
        let rsdt_addr = (*rsdp_ptr).rsdt_address as u64;

        // Ha van XSDT (revision >= 2) és nem nulla, használjuk azt
        if revision >= 2 && xsdt_addr != 0 {
            crate::kernel::console::LOGGER.info(&alloc::format!(
                "ACPI: Found RSDP v{}, using XSDT at {:#x}", revision, xsdt_addr
            ));
            tables::parse_xsdt(xsdt_addr);
        } else {
            // Különben marad a régi 32 bites RSDT
            crate::kernel::console::LOGGER.info(&alloc::format!(
                "ACPI: Found RSDP v{}, using RSDT at {:#x}", revision, rsdt_addr
            ));
            tables::parse_rsdt(rsdt_addr);
        }
    }
}
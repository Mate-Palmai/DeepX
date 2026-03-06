#[repr(C, packed)]
pub struct Rsdp {
    pub signature: [u8; 8],     // "RSD PTR "
    pub checksum: u8,
    pub oem_id: [u8; 6],
    pub revision: u8,
    pub rsdt_address: u32,      // 32-bit ACPI 1.0
    // ACPI 2.0+:
    pub length: u32,
    pub xsdt_address: u64,      // 64-bit
    pub extended_checksum: u8,
    pub reserved: [u8; 3],
}

impl Rsdp {
    pub fn is_valid(&self) -> bool {
        &self.signature == b"RSD PTR "
    }
}
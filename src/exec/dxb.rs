#[repr(C, packed)]
pub struct DxbHeader {
    pub magic: [u8; 4],
    pub version: u16,
    pub entry_point: u64,
    pub section_count: u16,
    pub checksum: u32,
}

pub fn load_and_verify_dxb(data: &[u8]) -> Result<u64, &'static str> {
    if data.len() < core::mem::size_of::<DxbHeader>() {
        return Err("File too small for DXB header");
    }

    let header = unsafe { &*(data.as_ptr() as *const DxbHeader) };

    if header.magic != [0x7F, b'D', b'X', b'B'] {
        return Err("Invalid DXB magic signature");
    }

    if header.version != 1 {
        return Err("Unsupported DXB version");
    }

    Ok(header.entry_point)
}
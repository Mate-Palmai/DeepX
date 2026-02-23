pub mod frame;
pub mod ids;
pub mod dispatch;
pub mod validate;
pub mod errors;

pub const SYSTUNNEL_VERSION: &str = "v0.0.6";

pub fn init() {
    unsafe {
        crate::kernel::console::LOGGER.ok("Systunnel interface online (DPL3 Trap Gate 0x80)");
    }
}
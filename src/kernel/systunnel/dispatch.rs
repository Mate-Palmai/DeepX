use crate::kernel::systunnel::validate::Validator;
use crate::kernel::systunnel::ids::TunnelID;
use crate::kernel::systunnel::frame::SystunnelFrame;

use alloc::format;

/// Ez a függvény dolgozza fel a Ring 3-ból érkező hívásokat.
/// A frame az összes regisztert tartalmazza, amit az assembly mentett.
#[no_mangle] // Fontos, hogy az assembly elérje ezen a néven!
pub extern "C" fn dispatch(frame: &mut SystunnelFrame) {
    let id = TunnelID::from(frame.rax);

    // A visszatérési értéket (vagy hibaüzenetet) a RAX-ba írjuk vissza
    frame.rax = match id {
        TunnelID::Log => {
            let ptr = frame.rdi;
            let len = frame.rsi;

            // Ellenőrizzük, hogy a memóriaterület egyáltalán olvasható-e
            match Validator::check_buffer(ptr, len) {
                Ok(_) => {
                    unsafe {
                        let s = core::slice::from_raw_parts(ptr as *const u8, len as usize);
                        if let Ok(msg) = core::str::from_utf8(s) {
                            // A kernel Logger-ét használjuk a kiíratáshoz
                            
                            crate::kernel::console::LOGGER.tunnel(msg);
                            // crate::kernel::console::LOGGER.debug(&format!("LOG FROM RING 3: addr=0x{:x}, len={}", ptr, len));
                            // crate::kernel::console::LOGGER.debug(&format!("DEBUG: P: 0x{:X}, L: {}", frame.rdi, frame.rsi));
                            // crate::kernel::console::LOGGER.debug("SYSTUNNEL LOG TRIGGERED!");
                            0 // Siker kód
                        } else {
                            crate::kernel::console::LOGGER.error("SYSTUNNEL: UTF-8 Decode Error");
                            1 // Dekódolási hiba
                        }
                    }
                },
                Err(e) => e as u64, // Például: 2 (Access Violation)
            }
        },
        // A dispatch.rs-en belül:
        // kernel/systunnel/dispatch.rs
        TunnelID::Exit => {
            let status = frame.rdi;
            crate::kernel::console::LOGGER.warn("OS Discovery Exit Triggered");
            // Itt a kernelnek át kell vennie az irányítást!
            crate::prepare_recovery_space_and_jump();
            0 
        },

        // Itt tudsz majd bővíteni (pl. TunnelID::GetKeyboardChar)
        
        _ => {
            unsafe {
                crate::kernel::console::LOGGER.warn("SYSTUNNEL: Unknown Call ID requested.");
            }
            404
        },
    };
}
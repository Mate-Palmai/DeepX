/*
 * DeepX OS Project
 * Copyright (C) 2024-2026 - Máté Pálmai
 *
 * File: /src/kernel/boot/phase_state.rs
 * Description: Boot phase state management.
 */

use core::sync::atomic::{AtomicU8, Ordering};

// === Boot phases ===
#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum BootPhase {
    Early,
    EarlyCpuInit,
    MemoryInit,
    CpuInit,
    VfsInit,
    SystunnelInit,
    DriversInit,
    FsInit,
    UserspaceInit,
    Running,
    Panic,
}

// === Global boot state ===
static BOOT_PHASE: AtomicU8 = AtomicU8::new(BootPhase::Early as u8);

// === API ===
pub fn set_phase(new: BootPhase) {
    
    let old = BOOT_PHASE.swap(new as u8, Ordering::SeqCst);

    if old != new as u8 {
        on_phase_changed(
            unsafe { core::mem::transmute(old) },
            new,
        );

        
    }
}

pub fn get_phase() -> BootPhase {
    unsafe { core::mem::transmute(BOOT_PHASE.load(Ordering::SeqCst)) }
}

fn on_phase_changed(old: BootPhase, new: BootPhase) {
    unsafe {
            let name = match new {
                BootPhase::Early => "Early",
                BootPhase::EarlyCpuInit => "EarlyCpuInit",
                BootPhase::MemoryInit => "MemoryInit",
                BootPhase::CpuInit => "CpuInit",
                BootPhase::VfsInit => "VfsInit",
                BootPhase::SystunnelInit => "SystunnelInit",
                BootPhase::DriversInit => "DriversInit",
                BootPhase::FsInit => "FsInit",
                BootPhase::UserspaceInit => "UserspaceInit",
                BootPhase::Running => "Running",
                BootPhase::Panic => "Panic",
                _ => "^&eUnknown Phase",
            };
            
            // 1. Megnyitjuk a sort (újsor nélkül)
            crate::kernel::console::LOGGER.ok_nl("BootPhase changed: ^&f");
            
            // 2. Hozzáírjuk a színes nevet
            crate::kernel::console::LOGGER.raw(name);
            
            // 3. Manuálisan lezárjuk a sort
            crate::kernel::console::LOGGER.raw("\n");
        }
    
}

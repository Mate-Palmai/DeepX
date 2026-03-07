/*
 * DeepX Project
 * Copyright (C) 2024-2026 - Máté Pálmai
 *
 * File: /src/kernel/boot/phase_state.rs
 * Description: Boot phase state management.
 */

use core::sync::atomic::{AtomicU8, Ordering};

// ---Boot phases--- 
#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum BootPhase {
    Early,
    EarlyCpuInit,
    MemoryInit,
    AcpiInit,
    CpuInit,
    VfsInit,
    SystunnelInit,
    DriversInit,
    FsInit,
    UserspaceInit,
    Running,
    Panic,
}

// ---Global boot state---
static BOOT_PHASE: AtomicU8 = AtomicU8::new(BootPhase::Early as u8);

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


use core::fmt::Write;

fn on_phase_changed(_old: BootPhase, new: BootPhase) {
    let name = match new {
        BootPhase::Early => "Early",
        BootPhase::EarlyCpuInit => "EarlyCpuInit",
        BootPhase::MemoryInit => "MemoryInit",
        BootPhase::AcpiInit => "AcpiInit",
        BootPhase::CpuInit => "CpuInit",
        BootPhase::VfsInit => "VfsInit",
        BootPhase::SystunnelInit => "SystunnelInit",
        BootPhase::DriversInit => "DriversInit",
        BootPhase::FsInit => "FsInit",
        BootPhase::UserspaceInit => "UserspaceInit",
        BootPhase::Running => "Running",
        BootPhase::Panic => "Panic",
    };

    let mut buf = [0u8; 64]; 
    let mut offset = 0;

    let prefix = "BootPhase changed: ^&f";
    
    let p_bytes = prefix.as_bytes();
    buf[..p_bytes.len()].copy_from_slice(p_bytes);
    offset += p_bytes.len();

    let n_bytes = name.as_bytes();
    let to_copy = core::cmp::min(n_bytes.len(), buf.len() - offset);
    buf[offset..offset + to_copy].copy_from_slice(&n_bytes[..to_copy]);
    offset += to_copy;

    if let Ok(final_msg) = core::str::from_utf8(&buf[..offset]) {
        crate::kernel::console::LOGGER.ok(final_msg);
    }
}

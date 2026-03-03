/*
 * DeepX Project
 * Copyright (C) 2024-2026 - Máté Pálmai
 *
 * File: /src/kernel/systunnel/dispatch.rs
 * Description: System tunnel dispatch logic with VFS and Process support.
 */

use crate::kernel::systunnel::validate::Validator;
use crate::kernel::systunnel::ids::TunnelID;
use crate::kernel::systunnel::frame::SystunnelFrame;
use crate::kernel::fs::vfs;

#[no_mangle] 
pub extern "C" fn dispatch(frame: &mut SystunnelFrame) -> u64 {
    let id = TunnelID::from(frame.rax);

    if frame.cs != 0x1B {
        unsafe {
            crate::kernel::console::LOGGER.error(&alloc::format!(
                "CRITICAL: Segment Corruption! CS is 0x{:x} instead of 0x1B", 
                frame.cs
            ));
        }
    }

    if frame.rip < 0x1000 {
        unsafe {
            crate::kernel::console::LOGGER.error(&alloc::format!(
                "CRITICAL: Null or Low RIP! RIP is 0x{:x}", 
                frame.rip
            ));
        }
    }
    
    if id == TunnelID::Execute || id == TunnelID::VfsExists {
        unsafe {
            crate::kernel::console::LOGGER.debug(&alloc::format!(
                "IRETQ PREP -> RIP: 0x{:x}, CS: 0x{:x}, RSP: 0x{:x}, SS: 0x{:x}",
                frame.rip, frame.cs, frame.rsp, frame.ss
            ));
        }
    }

    let result = match id {
        // ID: 0 - Status OK (Ping)
        TunnelID::Ok => 0,

        // ID: 1 - Exit (RDI: exit_code)
        TunnelID::Exit => {
            let status = frame.rdi;
            unsafe {
                crate::kernel::console::LOGGER.warn(&alloc::format!("PROCESS: Exit triggered with code {}", status));
            }
            if status != 0 {
                crate::prepare_recovery_space_and_jump();
            }
            0 
        },

        // ID: 2 - Log (RDI: ptr, RSI: len)
        TunnelID::Log => {
            let ptr = frame.rdi;
            let len = frame.rsi;

            match Validator::check_buffer(ptr, len) {
                Ok(_) => {
                    unsafe {
                        let s = core::slice::from_raw_parts(ptr as *const u8, len as usize);
                        if let Ok(msg) = core::str::from_utf8(s) {
                            crate::kernel::console::LOGGER.tunnel(msg);
                            0
                        } else {
                            crate::kernel::console::LOGGER.error("SYSTUNNEL: UTF-8 Decode Error");
                            1
                        }
                    }
                },
                Err(e) => e as u64, 
            }
        },

        // ID: 9 - Execute (RDI: path_ptr, RSI: len)
        TunnelID::Execute => {
        let ptr = frame.rdi;
        let len = frame.rsi;

        let s = unsafe { core::slice::from_raw_parts(ptr as *const u8, len as usize) };
            if let Ok(path) = core::str::from_utf8(s) {
                let root_lock = crate::kernel::fs::vfs::ROOT_NODE.lock();
                
                if let Some(root) = root_lock.as_ref() {
                    let clean_path = path.strip_prefix('/').unwrap_or(path);

                    if let Ok(node) = root.operations.finddir(clean_path) {
                        let load_virt_addr = 0x4000000 + (node.inode % 0x1000000); 
                        let size = node.size as usize;
                        
                        use crate::kernel::mem::paging::{PageTableFlags, VirtAddr, Mapper};
                        use crate::kernel::mem::pmm;

                        unsafe {
                            let mut mapper = Mapper::new();
                            let flags = PageTableFlags::PRESENT 
                                    | PageTableFlags::WRITABLE 
                                    | PageTableFlags::USER_ACCESSIBLE;

                            for offset in (0..size).step_by(4096) {
                                if let Some(phys_frame) = pmm::alloc_frame() {
                                    mapper.map_to(VirtAddr(load_virt_addr + offset as u64), phys_frame, flags);
                                }
                            }

                            let dest = core::slice::from_raw_parts_mut(load_virt_addr as *mut u8, size);
                            if node.operations.read(0, dest).is_ok() {
                                
                                let stack_base = load_virt_addr + ((size as u64 + 0xFFF) & !0xFFF) + 0x1000;
                                let stack_size = 0x4000; // 16 KB
                                
                                for offset in (0..stack_size).step_by(4096) {
                                    if let Some(phys_frame) = pmm::alloc_frame() {
                                        mapper.map_to(VirtAddr(stack_base + offset), phys_frame, flags);
                                    }
                                }

                                frame.rip = load_virt_addr;
                                frame.rsp = stack_base + stack_size - 8; 

                                0 // RAX = Success
                            } else { 500 }
                        }
                    } else { 404 }
                } else { 503 }
            } else { 400 }
        },

        // ID: 10 - VfsExists (RDI: path_ptr, RSI: len -> RAX: 1/0)
        TunnelID::VfsExists => {
            let ptr = frame.rdi;
            let len = frame.rsi;
            
            if let Ok(_) = Validator::check_buffer(ptr, len) {
                let s = unsafe { core::slice::from_raw_parts(ptr as *const u8, len as usize) };
                if let Ok(path) = core::str::from_utf8(s) {
                    let normalized = if path.starts_with('/') { &path[1..] } else { path };
                    
                    if vfs::exists(path) || vfs::exists(normalized) { 
                        1 
                    } else { 
                        0 
                    }
                } else { 0 }
            } else { 0 }
        },
        
        _ => {
            unsafe {
                crate::kernel::console::LOGGER.warn("SYSTUNNEL: Unknown Call ID requested.");
            }
            404
        },
    };


    frame.rax = result;
    result
}
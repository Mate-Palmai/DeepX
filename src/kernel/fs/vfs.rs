/*
 * DeepX Project
 * Copyright (C) 2024-2026 - Máté Pálmai
 *
 * File: /src/kernel/fs/vfs.rs
 * Description: Virtual File System implementation for the kernel.
 */

use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use alloc::boxed::Box;
use spinning_top::Spinlock;
use limine::request::ModuleRequest;

use crate::MODULE_REQUEST;

pub const VFS_VERSION: &str = "v0.1.0";

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NodeType {
    File,
    Directory,
    CharDevice,
    BlockDevice,
}

#[derive(Debug)]
pub enum VfsError {
    FileNotFound,
    PermissionDenied,
    NotADirectory,
    IoError,
}

pub struct VfsNode {
    pub name: String,
    pub node_type: NodeType,
    pub inode: u64,
    pub size: u64,
    pub operations: Box<dyn VfsOperations + Send + Sync>,
}

pub trait VfsOperations {
    fn read(&self, offset: u64, buffer: &mut [u8]) -> Result<usize, VfsError>;
    fn write(&mut self, offset: u64, buffer: &[u8]) -> Result<usize, VfsError>;
    fn readdir(&self) -> Result<Vec<VfsNode>, VfsError>;
    fn finddir(&self, name: &str) -> Result<VfsNode, VfsError>;
}

pub static ROOT_NODE: Spinlock<Option<VfsNode>> = Spinlock::new(None);

pub fn init_vfs(root: VfsNode) {
    let mut root_lock = ROOT_NODE.lock();
    *root_lock = Some(root);
    unsafe {
        crate::kernel::console::LOGGER.ok("VFS initialized");
    }
}

pub struct RootRamFS;

impl RootRamFS {
    pub fn new_node() -> VfsNode {
        VfsNode {
            name: String::from("/"),
            node_type: NodeType::Directory,
            inode: 0,
            size: 0,
            operations: Box::new(RootRamFS),
        }
    }
}

impl VfsOperations for RootRamFS {
    fn read(&self, _offset: u64, _buffer: &mut [u8]) -> Result<usize, VfsError> {
        Err(VfsError::IoError)
    }

    fn write(&mut self, _offset: u64, _buffer: &[u8]) -> Result<usize, VfsError> {
        Err(VfsError::PermissionDenied)
    }

    fn readdir(&self) -> Result<Vec<VfsNode>, VfsError> {
        let mut entries = Vec::new();

        if let Some(response) = MODULE_REQUEST.get_response() {
            for module in response.modules() {
                let name_str = core::str::from_utf8(module.cmdline())
                    .unwrap_or("unknown")
                    .to_string();

                entries.push(VfsNode {
                    name: name_str,
                    node_type: NodeType::File,
                    inode: module.addr() as u64,
                    size: module.size(),
                    operations: Box::new(ModuleFile {
                        base: module.addr() as u64,
                        size: module.size(),
                    }),
                });
            }
        }
        Ok(entries)
    }

    fn finddir(&self, name: &str) -> Result<VfsNode, VfsError> {
        let entries = self.readdir()?;
        for entry in entries {
            if entry.name == name {
                return Ok(entry);
            }
        }
        Err(VfsError::FileNotFound)
    }
}

unsafe impl Send for RootRamFS {}
unsafe impl Sync for RootRamFS {}

pub struct ModuleFile {
    pub base: u64,
    pub size: u64,
}

impl VfsOperations for ModuleFile {
    fn read(&self, offset: u64, buffer: &mut [u8]) -> Result<usize, VfsError> {
        if offset >= self.size {
            return Ok(0);
        }

        let available = self.size - offset;
        let count = core::cmp::min(buffer.len() as u64, available) as usize;
        
        let ptr = (self.base + offset) as *const u8;

        unsafe {
            core::ptr::copy_nonoverlapping(ptr, buffer.as_mut_ptr(), count);
        }

        Ok(count)
    }

    fn write(&mut self, _offset: u64, _buffer: &[u8]) -> Result<usize, VfsError> {
        Err(VfsError::PermissionDenied)
    }

    fn readdir(&self) -> Result<Vec<VfsNode>, VfsError> {
        Err(VfsError::NotADirectory)
    }

    fn finddir(&self, _name: &str) -> Result<VfsNode, VfsError> {
        Err(VfsError::NotADirectory)
    }
}

unsafe impl Send for ModuleFile {}
unsafe impl Sync for ModuleFile {}

pub fn dump_vfs_at_boot() {
    let root_lock = ROOT_NODE.lock();
    if let Some(root) = root_lock.as_ref() {
        if let Ok(entries) = root.operations.readdir() {
            for entry in entries {
                
                let msg = alloc::format!(
                    "  Found node: {} ({} bytes) at 0x{:016x}", 
                    entry.name, 
                    entry.size,
                    entry.inode
                );

                unsafe {
                    crate::kernel::console::LOGGER.info(&msg);
                }
            }
        }
    }
}
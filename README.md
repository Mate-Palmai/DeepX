# DeepX Kernel

**Current Development Branch:** `v0.1.x (Alpha)`

DeepX is a hobbyist microkernel project written in Rust for the x86_64 architecture. This repository contains the core kernel logic, basic hardware abstraction layers, and a minimal userspace interface.

## Important Disclaimer

- **Not for OS Development:** This project is in its infancy and is NOT suitable for use as a base for other OS development projects or production environments.
- **Known Issues:** I am aware of numerous bugs, stability issues, and architectural shortcuts. These are part of the learning process and will be addressed in future iterations.
- **Documentation:** The codebase currently suffers from a lack of proper documentation. Improving code comments and technical write-ups is a priority for upcoming patches.
- **Hobby Project:** This is a personal learning experiment. Performance and stability may be suboptimal in many areas.

## Project Structure

- **`src/arch/`**: x86_64 specific implementation (GDT, IDT, APIC, PIC, and Task State Segment).
- **`src/kernel/`**: The core logic of the kernel.
  - `mem/`: PMM, VMM (paging), and Heap management.
  - `process/`: Task structures and the Round-Robin scheduler.
  - `fs/`: Virtual File System (VFS) abstraction.
  - `systunnel/`: The system call / IPC interface between kernel and userspace.
  - `console/`: The kernel shell, logging system, and display management.
- **`src/userspace/`**: Minimal applications (Recovery Console, OS Discovery) running as separate binaries.

## Current Features (v0.1)

- **Multitasking:** Basic task switching and life-cycle management (kill/tasks commands).
- **VFS:** Initial support for loading boot modules as files.
- **Kernel Shell:** Interactive CLI for system monitoring and debugging.
- **Panic System:** Custom exception handling with Developer and User modes.

## Future Roadmap

- **Patches:** Improving documentation, fixing race conditions, and cleaning up the HAL.
- **v0.2 Goals:** ACPI table parsing and proper Ring 3 (Userspace) privilege isolation.

## Build & Run

To build and test the DeepX Kernel, you need **Rust Nightly**, **xorriso**, and the **Limine** bootloader installed on your system.

```bash
# To compile the kernel and launch it immediately in QEMU:
./run.sh

# To synchronize the bootloader configuration (limine.conf) with initrd changes:
./update_limine.sh
```

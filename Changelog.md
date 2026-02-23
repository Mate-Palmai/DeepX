# Changelog

## [v0.0.0.3] - 2025-12-26

### Added

- **Global Descriptor Table (GDT) Implementation:** - Designed and loaded a custom 64-bit GDT.

  - Implemented a robust segment reloading mechanism using retfq (Far Return) to ensure proper Code (CS) and Data (DS/SS) segment transitions.

- **Interrupt Descriptor Table (IDT) Framework:** - Established the IDT structure to handle CPU-level exceptions.

  - Implemented a dedicated handler for Division by Zero (Exception 0x00).

- **Custom Visual Panic System:** - Developed a high-visibility "Kernel Panic" screen with a solid red background.

  - Integrated set_fb_debug to pass the raw framebuffer address to the exception handlers for emergency rendering.

### Fixed & Optimized

- **Modern Rust Naked Functions:** Updated exception handlers to use #[unsafe(naked)] and naked_asm! to comply with the latest Rust nightly compiler requirements.

- **Memory-Safe Interrupts:** Refactored handlers to safely borrow the Framebuffer during critical failures, preventing double-faults during panic sequences.

- **ASCII Compatibility:** Optimized all kernel-level art to use standard ASCII characters, ensuring consistent rendering across all VGA-compatible bitmap fonts.

- **Hardware Acceleration (QEMU/KVM):** - Optimized the boot script to support KVM (Kernel-based Virtual Machine) and VirtIO VGA for near-native execution speed and flicker-free rendering.

## [v0.0.0.2] - 2025-12-25

### Added

- **Limine Bootloader Integration:** Transitioned to the Limine boot protocol for modern x86_64 framebuffer initialization and boot services.
- **Modular Kernel Architecture:** Established a scalable directory structure (`arch/`, `kernel/lib/`, `kernel/boot/`) to decouple hardware-specific code from core logic.
- **Advanced Display Driver (display.rs):**
  - Integrated a custom 8x8 VGA bitmap font binary.
  - Implemented a `Console` abstraction to handle automatic cursor tracking and line spacing.
  - Support for multi-colored text segments within a single log line.
- **CPU Identification:** Implemented a CPUID discovery module to fetch and display the processor's raw brand string.
- **Boot Branding:** Added a grey-scaled ASCII art splash screen and a formatted system greeting.

### Fixed & Optimized

- **Memory Safety:** Implemented Rust lifetime specifiers for Framebuffer references to ensure memory integrity within the Console driver.
- **Auto-formatting:** Replaced manual row numbering with a dynamic vertical layout system and raw-string multiline support.

## [v0.0.0.1] - 2025-12-24

- Created a freestanding Rust binary for bare metal execution (no-std).
- Created bootimage with crate bootloader

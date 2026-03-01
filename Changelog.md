# Changelog

## [0.1.1] - 2026-03-01

### Added

- Implemented Hybrid Boot support (BIOS and UEFI).
- Added `limine-uefi-cd.bin` to the ISO structure to support modern UEFI firmware.
- Updated `xorriso` configuration to generate a GPT partition table with an EFI System Partition (ESP).

### Changed

- Unified project branding: Renamed all remaining instances of `DeepX_OS` to `DeepX`.
- Updated build scripts to reflect the new ISO naming convention.
- Incremented system version to 0.1.1 in `Cargo.toml` and internal kernel constants.

### Fixed

- Fixed boot failure on physical hardware (Bare Metal) by providing proper EFI binaries.

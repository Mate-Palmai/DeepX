#!/bin/bash
set -e

# Configuration
ISO_NAME="DeepX.iso"
ISO_ROOT="iso_root"
USERSPACE_DIR="src/userspace"
KERNEL_TARGET="x86_64-DeepX"
USERSPACE_TARGET="x86_64-userspace"

# Check for clean flag
if [[ "$1" == "--clean" || "$1" == "-c" ]]; then
    echo "--- 0. Cleaning Project ---"
    rm -rf target
    rm -rf "$USERSPACE_DIR/target"
    rm -f "$ISO_NAME"
    echo "Clean finished."
fi

echo "--- 1. Compiling Userspace ---"
cd "$USERSPACE_DIR"
cargo build --release --target "${USERSPACE_TARGET}.json"
cd ../..

USERSPACE_OUT="$USERSPACE_DIR/target/$USERSPACE_TARGET/release"

echo "--- 2. Converting to Flat Binaries ---"
# Extracts raw binary sections for the kernel to embed or load
objcopy -O binary -j .text -j .rodata -j .data "$USERSPACE_OUT/os_discovery" src/kernel/os_discovery.bin
objcopy -O binary "$USERSPACE_OUT/recovery_console" src/kernel/recovery.bin
objcopy -O binary "$USERSPACE_OUT/installer" iso_root/initrd/programs/installer.bin

echo "--- 3. Compiling Kernel ---"
cargo build --release --target "${KERNEL_TARGET}.json"

KERNEL_BIN="target/$KERNEL_TARGET/release/DeepX"

echo "--- 4. ISO Building ---"
mkdir -p "$ISO_ROOT/boot"
cp "$KERNEL_BIN" "$ISO_ROOT/boot/kernel.elf"

# Generate bootable ISO using xorriso
xorriso -as mkisofs -R -J \
    -b boot/limine/limine-bios-cd.bin \
    -no-emul-boot -boot-load-size 4 -boot-info-table \
    --efi-boot boot/limine/limine-uefi-cd.bin \
    -efi-boot-part --efi-boot-image \
    --protective-msdos-label \
    "$ISO_ROOT" -o "$ISO_NAME"

# Install Limine BIOS stage
./limine_bin bios-install "$ISO_NAME"

echo "--- 5. Launching QEMU ---"
qemu-system-x86_64 -enable-kvm -cpu host -smp 4 -machine q35 -cdrom "$ISO_NAME" \
    -rtc base=localtime -serial stdio -m 2048M -d cpu_reset
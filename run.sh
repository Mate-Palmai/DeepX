#!/bin/bash
set -e # Megáll, ha bármi hiba történik

ISO_NAME="DeepX_OS.iso"
ISO_ROOT="iso_root"

echo "--- 1. Cleaning up ---"
# rm -rf target
rm -f src/kernel/recovery_bin.raw

echo "--- 2. Compiling Userspace ---"
cd src/userspace
# Kényszerítjük a tiszta fordítást
# cargo clean
cargo build --release --target x86_64-userspace.json
cd ../..

USERSPACE_OUT="src/userspace/target/x86_64-userspace/release"

echo "--- 2.5. Converting to Flat Binaries ---"
# Itt dől el minden: ha nincs kimenet, a script megáll
objcopy -O binary \
    -j .text -j .rodata -j .data \
    src/userspace/target/x86_64-userspace/release/os_discovery \
    src/kernel/os_discovery.bin
# objcopy -S -O binary --only-section=.text src/userspace/target/x86_64-userspace/release/os_discovery src/kernel/os_discovery.bin
objcopy -O binary "$USERSPACE_OUT/recovery_console" src/kernel/recovery.bin

echo "--- 3. Compiling Kernel ---"
# A kernel befordítja a friss .bin fájlokat (pl. include_bytes!-el)
cargo build --release --target x86_64-DeepX-OS.json

KERNEL_BIN="target/x86_64-DeepX-OS/release/DeepX_OS"

echo "--- 4. ISO Building ---"
mkdir -p iso_root/boot
cp "$KERNEL_BIN" iso_root/boot/kernel.elf

xorriso -as mkisofs -R -J -z \
    -b boot/limine/limine-bios-cd.bin \
    -no-emul-boot -boot-load-size 4 -boot-info-table \
    $ISO_ROOT -o $ISO_NAME

./limine_bin bios-install $ISO_NAME

echo "--- 5. Launch ---"
# Tipp: Hozzáadtam a -d int flaget, hogy ha mégis Page Fault lenne, 
# a QEMU logban lásd a pontos processzor állapotot.
qemu-system-x86_64 -enable-kvm -cpu host -cdrom $ISO_NAME -rtc base=localtime -serial stdio -m 2048M -d cpu_reset
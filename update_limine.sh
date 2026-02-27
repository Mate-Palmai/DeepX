#!/bin/bash

CONF_FILE="iso_root/boot/limine.conf"
INITRD_DIR="iso_root/initrd"

cat << EOF > $CONF_FILE
timeout: 3
term_backend: text

/DeepX OS
    protocol: limine
    path: boot():/boot/kernel.elf
    vga: on
EOF

find "$INITRD_DIR" -type f | while read -r full_path; do
    rel_path=${full_path#$INITRD_DIR/}
    
    echo "    module_path: boot():/initrd/$rel_path" >> $CONF_FILE
    
    echo "    module_cmdline: $rel_path" >> $CONF_FILE
done

echo "Limine.conf updated reursively from $INITRD_DIR"
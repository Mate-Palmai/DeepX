# Automatically update limine.conf with the contents of the initrd directory
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

for file in "$INITRD_DIR"/*; do
    if [ -f "$file" ]; then
        filename=$(basename "$file")
        echo "    module_path: boot():/initrd/$filename" >> $CONF_FILE
        echo "    module_cmdline: $filename" >> $CONF_FILE
    fi
done

echo "Limine.conf updated with the following files:"
ls $INITRD_DIR
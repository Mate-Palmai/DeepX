#!/bin/bash
# sign.sh - DeepX Binary Signer

if [ -z "$1" ]; then
    echo "Usage: ./sign.sh <filename.bin>"
    exit 1
fi

INPUT=$1
OUTPUT="${INPUT%.*}.dxb"

MAGIC="\x7F\x44\x58\x42" # \x7F, D, X, B
VERSION="\x01\x00"       # 0x0001
ENTRY="\x00\x00\x00\x00\x00\x00\x00\x00" 
SECTIONS="\x01\x00"    
CHECKSUM="\x00\x00\x00\x00" 

printf "$MAGIC$VERSION$ENTRY$SECTIONS$CHECKSUM" > header.tmp

cat header.tmp "$INPUT" > "$OUTPUT"
rm header.tmp

echo "Successfully signed: $OUTPUT (Header: 20 bytes)"
ENTRY(_start)
PHDRS
{
    text    PT_LOAD FLAGS(5);
    rodata  PT_LOAD FLAGS(4);
    data    PT_LOAD FLAGS(6);
}
SECTIONS
{
    . = 0xffffffff80000000;
    .text : {
        *(.text .text.*)
        *(.userspace)      
        *(.userspace*)
    } :text
    . = ALIGN(0x1000);
    .rodata : {
        *(.rodata .rodata.*)
    } :rodata
    . = ALIGN(0x1000);
    .data : {
        *(.data .data.*)
    } :data
    .bss : {
        *(.bss .bss.*)
        *(COMMON)
    } :data
    /DISCARD/ : {
        *(.eh_frame)
        *(.note .note.*)
    }
}
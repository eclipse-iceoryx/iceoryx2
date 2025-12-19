MEMORY {
    VECTORS : ORIGIN = 0x00000, LENGTH = 0x1000   /* 4 KB for vectors */
    
    /* NOTE: This is a generic memory region for QEMU/emulation only.
     * Real hardware integration requires proper memory map configuration
     * (TCM, SRAM, DRAM, etc.) based on the target platform.
     */
    MEM : ORIGIN = 0x10000, LENGTH = 0x800000 /* 8,388,608 bytes (8 MB) */
}

ENTRY(_start)

SECTIONS {
    /* Vector table at reset address */
    .vectors : {
        KEEP(*(.text.startup))
    } > VECTORS
    
    /* Main code and data sections */
    . = ORIGIN(MEM);
    
    .text : ALIGN(16) {
        *(.text .text*)
    } > MEM
    
    .rodata : ALIGN(16) {
        *(.rodata .rodata*)
    } > MEM
    
    .data : ALIGN(16) {
        *(.data .data*)
    } > MEM
    
    .bss : ALIGN(16) {
        __bss_start = .;
        *(.bss .bss* COMMON)
        __bss_end = .;
    } > MEM
    
    . = ALIGN(16);
    _heap_start = .;
    .heap : {
        . += 0x100000;  /* 1,048,576 bytes (1 MB) */
    } > MEM
    _heap_end = .;
    
    . = ALIGN(16);
    .stack : {
        _stack_bottom = .;
        . += 0x100000;  /* 1,048,576 bytes (1 MB) */
        _stack_top = ALIGN(., 8);
    } > MEM
    
    /DISCARD/ : {
        *(.note .note*)
    }
    
    _end = .;
}

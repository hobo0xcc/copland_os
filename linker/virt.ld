OUTPUT_ARCH( "riscv" )
ENTRY( _entry )

MEMORY
{
    RAM(wxa) : ORIGIN = 0x80000000, LENGTH = 256M
}

SECTIONS
{
    . = ORIGIN(RAM);

    .text : {
        PROVIDE(_text_start = .);
        *(.entry)
        *(.text .text.*)
        . = ALIGN(0x1000);
        *(.trampoline)
        . = ALIGN(0x1000);
        PROVIDE(_text_end = .);
    }

    .rodata : {
        PROVIDE(_rodata_start = .);
        . = ALIGN(16);
        *(.srodata .srodata.*)
        . = ALIGN(16);
        *(.rodata .rodata.*)
        . = ALIGN(0x1000);
        PROVIDE(_rodata_end = .);
    }

    .data : {
        PROVIDE(_data_start = .);
        . = ALIGN(16);
        *(.sdata .sdata.*)
        . = ALIGN(16);
        *(.data .data.*)
        . = ALIGN(0x1000);
        PROVIDE(_data_end = .);
    }

    .bss : {
        PROVIDE(_bss_start = .);
        . = ALIGN(16);
        *(.sbss .sbss.*)
        . = ALIGN(16);
        *(.bss .bss.*)
        . = ALIGN(0x1000);
        PROVIDE(_bss_end = .);
    }

    PROVIDE(_stack_start = .);
    . = . + 0x20000;
    PROVIDE(_stack_end = .);

    PROVIDE(_heap_start = .);
    PROVIDE(_heap_end = ORIGIN(RAM) + LENGTH(RAM));

    /* from qemu/hw/riscv/virt.c
    static const struct MemmapEntry {
        hwaddr base;
        hwaddr size;
    } virt_memmap[] = {
        [VIRT_DEBUG] =       {        0x0,         0x100 },
        [VIRT_MROM] =        {     0x1000,       0x11000 },
        [VIRT_TEST] =        {   0x100000,        0x1000 },
        [VIRT_RTC] =         {   0x101000,        0x1000 },
        [VIRT_CLINT] =       {  0x2000000,       0x10000 },
        [VIRT_PLIC] =        {  0xc000000,     0x4000000 },
        [VIRT_UART0] =       { 0x10000000,         0x100 },
        [VIRT_VIRTIO] =      { 0x10001000,        0x1000 },
        [VIRT_FLASH] =       { 0x20000000,     0x4000000 },
        [VIRT_DRAM] =        { 0x80000000,           0x0 },
        [VIRT_PCIE_MMIO] =   { 0x40000000,    0x40000000 },
        [VIRT_PCIE_PIO] =    { 0x03000000,    0x00010000 },
        [VIRT_PCIE_ECAM] =   { 0x30000000,    0x10000000 },
    };
    */

    /* Peripherals */
    . = 0x2000000;
    PROVIDE(_clint_start = .);
    . = . + 0x10000;
    PROVIDE(_clint_end = .);

    . = 0xc000000;
    PROVIDE(_plic_start = .);
    . = . + 0x4000000;
    PROVIDE(_plic_end = .);

    . = 0x10000000;
    PROVIDE(_uart0_start = .);
    . = . + 0x100;
    PROVIDE(_uart0_end = .);

    . = 0x10001000;
    PROVIDE(_virtio_start = .);
    . = . + 0x8000;
    PROVIDE(_virtio_end = .);
}
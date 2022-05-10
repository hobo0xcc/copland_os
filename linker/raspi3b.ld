OUTPUT_ARCH( "aarch64" )
ENTRY( _entry )

MEMORY {
    RAM(wxa) : ORIGIN = 0x80000, LENGTH = 256M
}

SECTIONS {
    . = ORIGIN(RAM);

    .text : {
        PROVIDE(_text_start = .);
        *(.entry)
        *(.text .text.*)
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
    . = . + 0x10000;
    PROVIDE(_stack_end = .);

    PROVIDE(_heap_start = .);
    PROVIDE(_heap_end = ORIGIN(RAM) + LENGTH(RAM));
}
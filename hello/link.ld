OUTPUT_ARCH(%ARCH%)

ENTRY(_start)

SECTIONS
{
    . = 0xf000000000000;
    _skernel = .;

    .text : ALIGN(4K) {
        _stext = .;
        KEEP(*(.text.head))
        KEEP(*(.text.boot.start))
        KEEP(*(.text.boot .text.boot.*))
        . = ALIGN(4);
        *(.text .text.*);

        . = ALIGN(4K);
        _etext = .;
    } 

    .rodata : ALIGN(4K) {
        _srodata = .;
        *(.rodata .rodata.*)
        *(.srodata .srodata.*)
        *(.sdata2 .sdata2.*)
    } 
   
    .data : ALIGN(4K) {
        _erodata = .;
        _sdata = .;
        *(.data.boot .data.boot.*)
        . = ALIGN(4K);
        *(.data .data.*)
        *(.sdata .sdata.*)
        *(.got .got.*)
    } 

    .tdata : ALIGN(0x10) {
        _stdata = .;
        *(.tdata .tdata.*)
        _etdata = .;
    } 

    .tbss : ALIGN(0x10) {
        _stbss = .;
        *(.tbss .tbss.*)
        *(.tcommon)
        _etbss = .;
    } 

    . = ALIGN(4K);
    _edata = .;

    .bss (NOLOAD) : ALIGN(4K) {
        _sbss = .;
        *(.bss .bss.*)
        *(.sbss .sbss.*)
        *(COMMON)
        . = ALIGN(4K);
        _ebss = .;
    }

    _ekernel = .;
    _kernel_size = _ekernel - _skernel;

    . = ALIGN(4K);
    _stack_bottom = .;
    _stack_top = . + 0x200000;
	/DISCARD/ : {
        *(.comment) *(.gnu*) *(.note*) *(.eh_frame*)
    }
}

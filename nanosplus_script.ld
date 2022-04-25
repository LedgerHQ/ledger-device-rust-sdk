MEMORY
{
  DISCARD (rwx) : ORIGIN = 0xd0000000, LENGTH = 1M

  FLASH   (rx)  : ORIGIN = 0xc0de0000, LENGTH = 400K
  DATA    (r)   : ORIGIN = 0xc0de0000, LENGTH = 400K
  SRAM    (rwx) : ORIGIN = 0xda7a0000, LENGTH = 30K
}

PAGE_SIZE  = 512;
STACK_SIZE = 1500;
END_STACK  = ORIGIN(SRAM) + LENGTH(SRAM);

/*
 * Ensure correct remap cx address
 */
_cx_trampoline     = 0x120001;

SECTIONS
{
  .text :
  {
    _text = .;
    _nvram_start = .;

    *(.boot*)
    *(.text*)
    *(.rodata*) 

    . = ALIGN(64);
    _etext = .;
  } > FLASH = 0x00

  .nvm_data : ALIGN(PAGE_SIZE)
  {
    /* . = ALIGN(PAGE_SIZE); */
    _nvram_data = .;
    *(.nvm_data*)

    /* . = ALIGN(PAGE_SIZE); */
    _envram_data = .;

    _install_parameters = .;
    _nvram_end = .;
  } > FLASH 

  .data :
  {
    . = ALIGN(4);

    _data = .;

    *(vtable)
    *(.data*)

    _edata = .;
  } > SRAM


  ASSERT( (_edata - _data) <= 0, ".data section must be empty" )

   .bss :
  {
    _bss = .;
    *(.bss*)
    _ebss = .;

    . = ALIGN(4);
    app_stack_canary = .;
    PROVIDE(app_stack_canary = .);
    . += 4;
    _stack_validation = .;
    . = _stack_validation + STACK_SIZE;
    _stack = ABSOLUTE(END_STACK) - STACK_SIZE;
    PROVIDE( _stack = ABSOLUTE(END_STACK) - STACK_SIZE);
    _estack = ABSOLUTE(END_STACK);
    PROVIDE( _estack = ABSOLUTE(END_STACK) );
  } > SRAM = 0x00


  .stack_sizes (INFO):
  {
    KEEP(*(.stack_sizes));
  }

  /DISCARD/ :
  {
    libc.a ( * )
    libm.a ( * )
    libgcc.a ( * )
    *(.ARM.exidx* .gnu.linkonce.armexidx.*)
  }
}

PROVIDE(_nvram = ABSOLUTE(_nvram_start));
PROVIDE(_envram = ABSOLUTE(_nvram_end));

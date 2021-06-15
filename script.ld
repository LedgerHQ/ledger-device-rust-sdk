MEMORY
{
  DISCARD (rwx) : ORIGIN = 0xd0000000, LENGTH = 1M

  FLASH   (rx)  : ORIGIN = 0xc0d00000, LENGTH = 400K
  SRAM    (rwx) : ORIGIN = 0x20000200, LENGTH = 4K+512
  CXSRAM  (rwx) : ORIGIN = 0x20001400, LENGTH = 1K
}

PAGE_SIZE  = 64;
STACK_SIZE = DEFINED(stack_size) ? stack_size : 1024;
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

    . = ALIGN(4);
    _etext = .;
  } > FLASH = 0x00

  .nvm_data :
  {
    . = ALIGN(PAGE_SIZE);
    _nvram_data = .;

    *(.nvm_data*)

    . = ALIGN(PAGE_SIZE);
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

  .cx_ram : {
    *(.bss.G_cx)
  } > CXSRAM = 0x00

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

  DEBUG (NOLOAD):
  {
    libc.a ( * )
    libm.a ( * )
    libgcc.a ( * )
    *(.ARM.exidx* .gnu.linkonce.armexidx.*)
  } > DISCARD
}

PROVIDE(_nvram = ABSOLUTE(_nvram_start));
PROVIDE(_envram = ABSOLUTE(_nvram_end));

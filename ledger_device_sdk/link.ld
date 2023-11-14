SECTIONS
{
  .text :
  {
    _text = .;
    _nvram_start = .;

    *(.boot*)
    *(.text*)
    *(.rodata*) 

    . = ALIGN(PAGE_SIZE);
    _etext = .;
  } > FLASH

  .nvm_data : ALIGN(PAGE_SIZE)
  {
    _nvram_data = .;
    *(.nvm_data*)
    . = ALIGN(PAGE_SIZE);
    _envram_data = .;
    _install_parameters = .;
    _nvram_end = .;
  } > FLASH 

  .data : ALIGN(4)
  {
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
    . += 4;
    _stack_validation = .;
    . = _stack_validation + STACK_SIZE;
    _stack = ABSOLUTE(END_STACK) - STACK_SIZE;
    _estack = ABSOLUTE(END_STACK);
  } > SRAM

  .ledger.api_level (INFO): { KEEP(*(.ledger.api_level)) }

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
    *(.debug_info)
  }
}

PROVIDE(_nvram = ABSOLUTE(_nvram_start));
PROVIDE(_envram = ABSOLUTE(_nvram_end));

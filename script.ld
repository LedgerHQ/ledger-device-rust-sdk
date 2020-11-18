MEMORY
{
  DISCARD (rwx) : ORIGIN = 0xd0000000, LENGTH = 1M
  FLASH   (rx)  : ORIGIN = 0xc0d00000, LENGTH = 400K
  SRAM    (rwx) : ORIGIN = 0x20001800, LENGTH = 4K
}

PAGE_SIZE  = 64;
STACK_SIZE = 2048;
END_STACK  = ORIGIN(SRAM) + LENGTH(SRAM);

SECTIONS
{
  .text :
  {
    _text = .;
    PROVIDE(_nvram = .);
    PROVIDE(_setjmp = setjmp);

    *(.boot*)
    *(.text*)
    *(.rodata*) 

    . = ALIGN(4);
    _etext = .;
  } > FLASH

  .nvm_data :
  {
    . = ALIGN(PAGE_SIZE);
    _nvram_data = .;

    *(.nvm_data*)

    . = ALIGN(PAGE_SIZE);
    _envram_data = .;

    _install_parameters = .;
    PROVIDE(N_install_parameters = .);
    PROVIDE(_envram = .);
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

  DEBUG (INFO) :
  {
    libc.a ( * )
    libm.a ( * )
    libgcc.a ( * )
    *(.ARM.exidx* .gnu.linkonce.armexidx.*)
  } > DISCARD

}

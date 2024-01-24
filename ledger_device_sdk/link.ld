PHDRS
{
  headers PT_PHDR FILEHDR PHDRS ;
  text PT_LOAD FLAGS(7); /* RWE */
  text1 PT_LOAD FLAGS(5); /* R_E */
  data PT_LOAD FLAGS(6); /* RW_ */
  bss PT_LOAD FLAGS(6); /* RW_ */
}

SECTIONS
{
  .text :
  {
    _nvram_start = .;
    _text = .;
    *(.boot*)
    *(.text*)
    *(.rodata*)
    _etext = .;
  } > FLASH :text

  .datarelro :
  {
    /* Writable area starts here */
    . = ALIGN(PAGE_SIZE);
    _nvram_data = .;

    /* This section is relocatable data produced by clang */
    _sdatarelro = .;
    *(.data.rel.ro*)
    _edatarelro = .;
  } > FLASH :text

  .got :
  {
    . = ALIGN(PAGE_SIZE);
    _sgot = .;
    *(.got .got*)
    _egot = .;
  } > FLASH :text

  .nvm_data : ALIGN(PAGE_SIZE)
  {
    _snvm_data = .;
    *(.nvm_data*)
    _envm_data = .;
  } > FLASH :text

  .data :
  {
    _data = .;
    *(.data .data.*)
    _edata = .;
  } > SRAM AT> FLASH :data

  .install :
  {
    _envram_data = .;
    _install_parameters = .;
    _nvram_end = .;
  } > FLASH :text1

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
  } > SRAM :bss

  _sidata = LOADADDR(.data);

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

  ledger.target (INFO): { KEEP(*(ledger.target)) }
  ledger.target_id (INFO): { KEEP(*(ledger.target_id)) }
  ledger.target_name (INFO): { KEEP(*(ledger.target_name)) }
  ledger.app_name (INFO): { KEEP(*(ledger.app_name)) }
  ledger.app_version (INFO): { KEEP(*(ledger.app_version)) }
  ledger.api_level (INFO): { KEEP(*(ledger.api_level)) }
  ledger.sdk_version (INFO): { KEEP(*(ledger.sdk_version)) }
  ledger.rust_sdk_version (INFO): { KEEP(*(ledger.rust_sdk_version)) }
  ledger.rust_sdk_name (INFO): { KEEP(*(ledger.rust_sdk_name)) }
  ledger.sdk_name (INFO): { KEEP(*(ledger.sdk_name)) }
  ledger.sdk_hash (INFO): { KEEP(*(ledger.sdk_hash)) }
}

PROVIDE(_nvram = ABSOLUTE(_nvram_start));
PROVIDE(_envram = ABSOLUTE(_nvram_end));

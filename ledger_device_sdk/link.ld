PHDRS
{
  flash0 PT_LOAD ;
  data   PT_LOAD ;
  flash2 PT_LOAD ;
  sram PT_LOAD ;

  headers PT_PHDR PHDRS ;
}

SECTIONS
{
  /* Code, read only, no relocations needed. */
  .text :
  {
    _text = .;
    /* Here begins flash. This symbol is used by the ideompotent `pic`
       function as the lower bound of addressed to relocate. */
    _nvram_start = .;

    *(.boot*)
    *(.text*)
    /* .rodata is moved out so we can update it */

    . = ALIGN(PAGE_SIZE);
    _etext = .;
  } > FLASH :flash0

  /* Relocations, read only, no relocations aginst the relocations themselves
     needed! */
  _reloc_size = SIZEOF(.rel.rodata) + SIZEOF(.rel.data) + SIZEOF(.rel.nvm_data);
  .rel_flash : ALIGN(PAGE_SIZE)
  {
    _relocs = .;

    . += _reloc_size;
    . = ALIGN(PAGE_SIZE);

    _erelocs = .;

    . = ALIGN(PAGE_SIZE);

    /* After this section we have mutable flash. Must be a multiple of PAGE_SIZE from _nvram_start. */
    _nvram_data = .;
  } > FLASH :flash0

  /* Immutable globals, read only during app running proper, but
     relocations are needed. (So not read-only completely.) */
  .rodata : ALIGN(PAGE_SIZE)
  {
    /* Moved here from .text so we can permantly apply relocations to it with
       nvm_write() */
    . = ALIGN(PAGE_SIZE);
    _rodata = .;
    _rodata_src = .;
    *(.rodata*)
    . = ALIGN(PAGE_SIZE);
    _erodata = .;
  } > FLASH :flash0
  _rodata_len = _erodata - _rodata;

  /* Mutable Globals, writable, relocations are needed. */
  .data : ALIGN(4)
  {
    _data = .;
    *(vtable)
    *(.data*)
    . = ALIGN(PAGE_SIZE);
    _edata = .;
  } > SRAM AT> FLASH :data =0xa4a4
  _data_len = SIZEOF(.data);

  ASSERT( (_edata - _data) <= 0, ".data section must be empty" )

  /* Persistent data, read and written during app running proper,
     relocations are also needed. */
  .nvm_data : ALIGN(PAGE_SIZE)
  {
    *(.nvm_data*)

    /* Store _nvram value during link_pass and use this to detect movement of
       _nvram as compared to the previous app execution, and redo the relocations
       if necessary */
    . = ALIGN(4);
    _nvram_prev_run = .;
    LONG(ABSOLUTE(_nvram_start))

    . = ALIGN(PAGE_SIZE);

    /* After this section we no longer have Flash memory at all. */

    /* This symbol is used by the mutable portion of flash calculations. */
    _envram_data = .;
    _install_parameters = .;
    /* This symbol is used by the ideompotent `pic` function as the upper
       bound of addressed to relocate. */
    _nvram_end = .;
  } > FLASH :flash2

  _sidata_src = LOADADDR(.data);

  .bss :
  {
    _bss = .;
    *(.bss*)
    _ebss = .;
    _bss_len = ABSOLUTE(_ebss) - ABSOLUTE(_bss);

    . = ALIGN(4);
    app_stack_canary = .;
    . += 4;
    _stack_validation = .;
    . = _stack_validation + STACK_SIZE;
    _stack = ABSOLUTE(END_STACK) - STACK_SIZE;
    _estack = ABSOLUTE(END_STACK);
  } > SRAM :sram

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

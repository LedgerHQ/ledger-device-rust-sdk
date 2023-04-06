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
    /* Here begins flash. This symbol is used by the ideompotent `pic`
       function as the lower bound of addressed to relocate. */
    _nvram_start = .;

    _text = .;

    *(.boot*)
    *(.text*)
    /* .rodata is moved out so we can update it */
    . = ALIGN(PAGE_SIZE);

    _etext = .;
  } > FLASH :flash0
  _text_len = _etext - _text;

  /* Relocations, read only, no relocations aginst the relocations themselves
     needed! */
  .rel_flash : ALIGN(PAGE_SIZE)
  {
    _relocs = .;

    /* define _reloc_size in the build.rs */
    . += _reloc_size;
    /*(.rel.rodata .rel.rodata*)*/
    /*(.rel.data .rel.data*)*/
    /*(.rel.nvm_data .rel.nvm_data*)*/
    . = ALIGN(PAGE_SIZE);

    _erelocs = .;

    . = ALIGN(PAGE_SIZE);

    /* After this section we have mutable flash. Must be a multiple of PAGE_SIZE from _nvram_start. */
    _nvram_data = .;
  } > FLASH :flash0
  _relocs_len = _erelocs - _relocs;

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
    /**(.rodata*)*/
    . = ALIGN(PAGE_SIZE);
    _erodata = .;
  } > FLASH :flash0
  _rodata_len = _erodata - _rodata;

  /* Mutable Globals, writable, relocations are needed. */
  .data : ALIGN(PAGE_SIZE)
  {
    _data = .;
    *(vtable)
    *(.data*)
    . = ALIGN(PAGE_SIZE);
    _edata = .;
    FILL(0xa4a4);
  } > SRAM AT> FLASH :data =0xa4a4
  _data_len = SIZEOF(.data); /* _edata - _data; */

  /* Persistent data, read and written during app running proper,
     relocations are also needed. */
  .nvm_data : ALIGN(PAGE_SIZE)
  {
    _nvm_data = .;
    _nvm_data_src = .;
    *(.nvm_data*)

    /* Store _nvram value during link_pass and use this to detect movement of
    _nvram as compared to the previous app execution, and redo the relocations
    if necessary */
    . = ALIGN(4);
    _nvram_prev_run = .;
    LONG(ABSOLUTE(_nvram_start))

    . = ALIGN(PAGE_SIZE);
    _envm_data = .;

    /* Debugging: Add some extra buffer space just to make sure we don't cloober install_parameters */
    . += PAGE_SIZE*4;
    /* and make sure we're on a page boundary for them */
    . = ALIGN(PAGE_SIZE);
    _install_parameters = .;

    /* After this section we no longer have Flash memory at all. */

    /* This symbol is used by the mutable portion of flash calculations. */
    _envram_data = .;
    /* This symbol is used by the ideompotent `pic` function as the upper
       bound of addressed to relocate. */
    _nvram_end = .;
  } > FLASH :flash2
  _nvm_data_len = _envm_data - _nvm_data;

  _sidata = LOADADDR(.data);
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
  } :NONE

  /*
  .rel (INFO):
  {
    KEEP(*(.rel.*));
  } :NONE
  */

  /*.text.debug_info (INFO):
  {
	  *(.text.debug_info)
  }*/

  /DISCARD/ :
  {
    libc.a ( * )
    libm.a ( * )
    libgcc.a ( * )
    *(.ARM.exidx* .gnu.linkonce.armexidx.*)
    *(.ARM.extab*)
    /* *(.rel.debug_info)
    *(.rela.debug_info) */
    *(.debug_info)
  }
}

PROVIDE(_nvram = ABSOLUTE(_nvram_start));
PROVIDE(_envram = ABSOLUTE(_nvram_end));

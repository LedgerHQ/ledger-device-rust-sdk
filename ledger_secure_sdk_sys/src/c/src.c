#include <elf.h>
#include <stdbool.h>
#include "exceptions.h"
#include "os_apilevel.h"
#include "string.h"
#include "seproxyhal_protocol.h"
#include "os_id.h"
#include "os_io_usb.h"
#include "os_nvm.h"
#include "os_pic.h"
#include "checks.h"
#ifdef HAVE_BLE
  #include "ledger_ble.h"
#endif

extern void sample_main();

struct SectionSrc;
struct SectionDst;

io_seph_app_t G_io_app;

extern Elf32_Rel _relocs;
extern Elf32_Rel _erelocs;


#ifdef SPECULOS_DEBUGGING
#define PRINTLNC(str) println_c(str)
void println_c(char* str);
#define PRINTHEXC(str, n) printhex_c(str, n)
void printhex_c(char* str, uint32_t m);
#else
#define PRINTLNC(str) while(0)
#define PRINTHEXC(str, n) while(0)
#endif

#ifdef TARGET_NANOS2 // ARM v8
# define SYMBOL_ABSOLUTE_VALUE(DST, SYM) \
  __asm volatile( \
    "movw %[result], #:lower16:" #SYM "\n\t" \
    "movt %[result], #:upper16:" #SYM \
    : [result] "=r" (DST))
#else // ARM v6
# define SYMBOL_ABSOLUTE_VALUE(DST, SYM) \
  __asm volatile( \
    "ldr %[result], =" #SYM \
    : [result] "=r" (DST))
#endif

#ifdef TARGET_NANOS2
# define SYMBOL_SBREL_ADDRESS(DST, SYM) \
  __asm volatile( \
    "movw %[result], #:lower16:" #SYM "(sbrel)\n\t" \
    "movt %[result], #:upper16:" #SYM "(sbrel)\n\t" \
    "add %[result], r9, %[result]" \
    : [result] "=r" (DST))
#elif defined(TARGET_NANOX) || defined(TARGET_STAX) || defined(TARGET_FLEX)
# define SYMBOL_SBREL_ADDRESS(DST, SYM) \
  __asm volatile( \
    "ldr %[result], =" #SYM "(sbrel)\n\t" \
    "add %[result], r9, %[result]" \
    : [result] "=r" (DST))
#elif defined(TARGET_NANOS)
# define SYMBOL_SBREL_ADDRESS(DST, SYM) \
  SYMBOL_ABSOLUTE_VALUE(DST, SYM)
#else
# error "unknown machine"
#endif

void link_pass(
  size_t sec_len,
  struct SectionSrc *sec_src,
  struct SectionDst *sec_dst,
  int nvram_move_amt,
  void* nvram_prev,
  void* envram_prev,
  int dst_ram)
{
#ifdef TARGET_NANOS
  uint32_t buf[16];
#else
  uint32_t buf[128];
#endif

  typedef typeof(*buf) link_addr_t;

  Elf32_Rel* relocs;
  SYMBOL_ABSOLUTE_VALUE(relocs, _relocs);
  Elf32_Rel* erelocs;
  SYMBOL_ABSOLUTE_VALUE(erelocs, _erelocs);


  Elf32_Rel *reloc_start = pic(relocs);
  Elf32_Rel *reloc_end = ((Elf32_Rel*)pic(erelocs-1)) + 1;

  PRINTHEXC("Section base address:", sec_dst);
  PRINTHEXC("Section base address runtime:", pic(sec_dst));
  // Loop over pages of the .rodata section,
  for (size_t i = 0; i < sec_len; i += sizeof(buf)) {
    // We will want to know if we changed each page, to avoid extra write-backs.
    bool is_changed = 0;

    size_t buf_size = sec_len - i < sizeof(buf)
      ? sec_len - i
      : sizeof(buf);

    // Copy over page from *run time* address.
    memcpy(buf, pic(sec_src) + i, buf_size);

    // This is the elf load (*not* elf link or bolos run time!) address of the page
    // we just copied.
    link_addr_t page_link_addr = (link_addr_t)sec_dst + i;

    PRINTHEXC("Chunk base: ", page_link_addr);
    PRINTHEXC("First reloc: ", reloc_start->r_offset);

    // Loop over the rodata entries - we could loop over the
    // correct seciton, but this also works.
    for (Elf32_Rel* reloc = reloc_start; reloc < reloc_end; reloc++) {
      // This is the (absolute) elf *load* address of the relocation.
      link_addr_t abs_offset = reloc->r_offset;

      // This is the relative offset on the current page, in
      // bytes.
      size_t page_offset = abs_offset - page_link_addr;

      // This is the relative offset on the current page, in words.
      //
      // Pointers in word_offset should be aligned to 4-byte
      // boundaries because of alignment, so we can just make it
      // uint32_t directly.
      size_t word_offset = page_offset / sizeof(*buf);

      // This includes word_offset < 0 because uint32_t.
      // Assuming no relocations go behind the end address.
      if (word_offset < sizeof(buf) / sizeof(*buf)) {
        PRINTLNC("Possible reloc");
        void* old = (void*) buf[word_offset];
        // The old ptr should lie within the nvram range of
        // * Link time nvram range
        //   If the link_pass is running for the first time
        //   Or if the link_pass is running for RAM
        // * The previous run's nvram range
        //   If the app has been moved after running the initial link_pass
        if (old >= nvram_prev && old < envram_prev) {
            void* new = old + nvram_move_amt;
            is_changed |= (old != new);
            buf[word_offset] = (uint32_t) new;
        }
      }
    }
    if (dst_ram) {
      PRINTLNC("Chunk to ram");
      memcpy((void*)sec_dst + i, buf, buf_size);
    } else if (is_changed) {
      PRINTLNC("Chunk to flash");
      nvm_write(pic((void *)sec_dst + i), buf, buf_size);
      if (memcmp(pic((void *)sec_dst + i), buf, buf_size)) {
        try_context_set(NULL);
        os_sched_exit(1);
      }
    } else {
      PRINTLNC("Unchanged flash chunk");
    }
  }

  /* PRINTLNC("Ending link pass"); */
}

void get_link_time_nvram_values(
  void** nvram_ptr_p,
  void** envram_ptr_p)
{
#if defined(ST31)
    SYMBOL_ABSOLUTE_VALUE(*nvram_ptr_p, _nvram);
    SYMBOL_ABSOLUTE_VALUE(*envram_ptr_p, _envram);
#elif defined(ST33) || defined(ST33K1M5)
    __asm volatile("ldr %0, =_nvram":"=r"(*nvram_ptr_p));
    __asm volatile("ldr %0, =_envram":"=r"(*envram_ptr_p));
#else
#error "invalid architecture"
#endif
}

void link_pass_ram(
  size_t sec_len,
  struct SectionSrc *sec_src,
  struct SectionDst *sec_dst)
{
    void* nvram_ptr;
    void* envram_ptr;
    get_link_time_nvram_values(&nvram_ptr, &envram_ptr);

    // Value of _nvram in this run
    void* nvram_current = pic(nvram_ptr);

    // Value (in bytes) of change in _nvram
    int nvram_move_amt = nvram_current - nvram_ptr;

    // The nvram_prev and envram_prev are the link time values
    link_pass(sec_len, sec_src, sec_dst, nvram_move_amt, nvram_ptr, envram_ptr, true);
}

void link_pass_nvram(
  size_t sec_len,
  struct SectionSrc *sec_src,
  struct SectionDst *sec_dst)
{
  void* nvram_ptr;
  void* envram_ptr;

  get_link_time_nvram_values(&nvram_ptr, &envram_ptr);

  // Value of _nvram in this run
  void* nvram_current = pic(nvram_ptr);

  void** nvram_prev_link_ptr;
  SYMBOL_ABSOLUTE_VALUE(nvram_prev_link_ptr, _nvram_prev_run);

  // Pointer to the location where nvram_prev's value is stored
  void** nvram_prev_val_ptr = (void**)pic(nvram_prev_link_ptr);

  // Value of _nvram and _envram in previous run
  void* nvram_prev = *nvram_prev_val_ptr;
  void* envram_prev = nvram_prev + (envram_ptr - nvram_ptr);

  void* link_pass_in_progress_tag = (void*) 0x1;
  if (nvram_prev == link_pass_in_progress_tag) {
      // This indicates that the previous link_pass did not complete successfully
      // Abort the app to avoid unexpected behaviour
      // The "fix" for this would be reinstalling the app
      os_sched_exit(1);
  }

  // Value (in bytes) of change in _nvram
  // If the app was moved after the previous run or link time
  int nvram_move_amt = nvram_current - nvram_prev;

  if (nvram_move_amt == 0) {
      // No change in _nvram means that we need not do link_pass again
      return;
  }

  // Add a tag to indicate we are in the middle of executing the link_pass
  nvm_write(nvram_prev_val_ptr, &link_pass_in_progress_tag, sizeof(void*));

  link_pass(sec_len, sec_src, sec_dst, nvram_move_amt, nvram_prev, envram_prev, false);

  // After successful completion of link_pass, clear the link_pass_in_progress_tag
  // And write the proper value of nvram_current
  nvm_write(nvram_prev_val_ptr, &nvram_current, sizeof(void*));
}

#ifdef HAVE_CCID
 #include "usbd_ccid_if.h"
uint8_t G_io_apdu_buffer[260];
#endif

int c_main(void) {
  __asm volatile("cpsie i");

  // Update pointers for pic(), only issuing nvm_write() if we actually changed a pointer in the block.
  // link_pass(&_rodata_len, &_rodata_src, &_rodata);
  size_t rodata_len;
  SYMBOL_ABSOLUTE_VALUE(rodata_len, _rodata_len);
  struct SectionSrc* rodata_src;
  SYMBOL_ABSOLUTE_VALUE(rodata_src, _rodata_src);
  struct SectionDst* rodata;
  SYMBOL_ABSOLUTE_VALUE(rodata, _rodata);

  link_pass_nvram(rodata_len, rodata_src, rodata);

  size_t data_len;
  SYMBOL_ABSOLUTE_VALUE(data_len, _data_len);
  struct SectionSrc* sidata_src;
  SYMBOL_ABSOLUTE_VALUE(sidata_src, _sidata_src);
  struct SectionDst* data;
  __asm volatile("mov %[result],r9" : [result] "=r" (data));

  link_pass_ram(data_len, sidata_src, data);

  size_t bss_len;
  SYMBOL_ABSOLUTE_VALUE(bss_len, _bss_len);
  struct SectionDst* bss;
  SYMBOL_SBREL_ADDRESS(bss, _bss);
  memset(bss, 0, bss_len);

  // formerly known as 'os_boot()'
  try_context_set(NULL);

  for(;;) {
    BEGIN_TRY {
      TRY {
        // below is a 'manual' implementation of `io_seproxyhal_init`
    #ifdef HAVE_MCU_PROTECT
        unsigned char c[4];
        c[0] = SEPROXYHAL_TAG_MCU;
        c[1] = 0;
        c[2] = 1;
        c[3] = SEPROXYHAL_TAG_MCU_TYPE_PROTECT;
        io_seproxyhal_spi_send(c, 4);
    #ifdef HAVE_BLE
        unsigned int plane = G_io_app.plane_mode;
    #endif
    #endif
        memset(&G_io_app, 0, sizeof(G_io_app));

    #ifdef HAVE_BLE
        G_io_app.plane_mode = plane;
    #endif
        G_io_app.apdu_state = APDU_IDLE;
        G_io_app.apdu_length = 0;
        G_io_app.apdu_media = IO_APDU_MEDIA_NONE;

        G_io_app.ms = 0;
        io_usb_hid_init();

        USB_power(0);
        USB_power(1);
    #ifdef HAVE_CCID
        io_usb_ccid_set_card_inserted(1);
    #endif

    #ifdef HAVE_BLE
        LEDGER_BLE_init();
    #endif

    #if !defined(HAVE_BOLOS) && defined(HAVE_PENDING_REVIEW_SCREEN)
        check_audited_app();
    #endif // !defined(HAVE_BOLOS) && defined(HAVE_PENDING_REVIEW_SCREEN)

        sample_main();
      }
      CATCH(EXCEPTION_IO_RESET) {
        continue;
      }
      CATCH_ALL {
        break;
      }
      FINALLY {
      }
    }
    END_TRY;
  }
  return 0;
}

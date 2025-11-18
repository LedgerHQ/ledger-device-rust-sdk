#include <elf.h>
#include <stdbool.h>
#include "exceptions.h"
#include "os_apilevel.h"
#include "string.h"
#include "os_io.h"
#include "os_id.h"
#include "os_nvm.h"
#include "os_pic.h"
#include "checks.h"

#ifdef HAVE_IO_USB
#include "usbd_ledger.h"
#endif  // HAVE_IO_USB

#ifdef HAVE_BLE
#include "ble_ledger.h"
#include "ble_ledger_profile_apdu.h"
#endif  // HAVE_BLE

extern void sample_main(int arg0);
extern void heap_init();

struct SectionSrc;
struct SectionDst;

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
#elif defined(TARGET_NANOX) || defined(TARGET_STAX) || defined(TARGET_FLEX) || defined(TARGET_APEX_P)
# define SYMBOL_SBREL_ADDRESS(DST, SYM) \
  __asm volatile( \
    "ldr %[result], =" #SYM "(sbrel)\n\t" \
    "add %[result], r9, %[result]" \
    : [result] "=r" (DST))
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
  uint32_t buf[128];

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
    // correct section, but this also works.
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
#if defined(ST33) || defined(ST33K1M5)
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

void c_reset_bss() {
  size_t bss_len;
  SYMBOL_ABSOLUTE_VALUE(bss_len, _bss_len);
  struct SectionDst* bss;
  SYMBOL_SBREL_ADDRESS(bss, _bss);
  memset(bss, 0, bss_len);
}

bolos_ux_params_t G_ux_params = {0};

void c_boot_std() {

    // Warn UX layer of io reset to avoid unwanted pin lock
    memset(&G_ux_params, 0, sizeof(G_ux_params));
    G_ux_params.ux_id = BOLOS_UX_IO_RESET;

    // If the app has just been booted from the UX, multiple os_ux calls may be necessary
    // to ensure UX layer has take the BOLOS_UX_IO_RESET instruction into account.
    for (uint8_t i = 0; i < 2; i++) {
        os_ux(&G_ux_params);
        if (os_sched_last_status(TASK_BOLOS_UX) == BOLOS_UX_OK) {
            break;
        }
    }

    os_io_init_t init_io;

    init_io.usb.pid        = 0;
    init_io.usb.vid        = 0;
    init_io.usb.class_mask = 0;
    memset(init_io.usb.name, 0, sizeof(init_io.usb.name));
#ifdef HAVE_IO_USB
    init_io.usb.class_mask = USBD_LEDGER_CLASS_HID;
#ifdef HAVE_WEBUSB
    init_io.usb.class_mask |= USBD_LEDGER_CLASS_WEBUSB;
#endif  // HAVE_WEBUSB
#ifdef HAVE_IO_U2F
    init_io.usb.class_mask |= USBD_LEDGER_CLASS_HID_U2F;

    init_io.usb.hid_u2f_settings.protocol_version            = 2;
    init_io.usb.hid_u2f_settings.major_device_version_number = 0;
    init_io.usb.hid_u2f_settings.minor_device_version_number = 1;
    init_io.usb.hid_u2f_settings.build_device_version_number = 0;
    init_io.usb.hid_u2f_settings.capabilities_flag = 0;
#endif  // HAVE_IO_U2F
#endif  // !HAVE_IO_USB

    init_io.ble.profile_mask = 0;
#ifdef HAVE_BLE
    init_io.ble.profile_mask = BLE_LEDGER_PROFILE_APDU;
#endif  // !HAVE_BLE

    os_io_init(&init_io);
    os_io_start();

    heap_init();
}

int c_main(int arg0) {
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
  
  // if libcall, does not reset bss as it is shared with the calling app
  if (arg0 == 0)
    c_reset_bss();

  // formerly known as 'os_boot()'
  try_context_set(NULL);

  for(;;) {
    BEGIN_TRY {
      TRY {
        // if libcall, does not start io and memory allocator
        if (arg0 == 0)
          c_boot_std();
        sample_main(arg0);
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

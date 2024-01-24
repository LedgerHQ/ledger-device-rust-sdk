#include "exceptions.h"
#include "os_apilevel.h"
#include "string.h"
#include "seproxyhal_protocol.h"
#include "os_id.h"
#include "os_io_usb.h"
#include "checks.h"
#include "os_pic.h"
#include "os_nvm.h"
#ifdef HAVE_BLE
  #include "ledger_ble.h"
#endif

void *pic_internal(void *link_address);

extern void sample_main();

io_seph_app_t G_io_app;

extern unsigned int _bss;
extern unsigned int _data;
extern unsigned int _edata;
extern unsigned int _sidata;
extern unsigned int _sdatarelro;
extern unsigned int _edatarelro;
extern unsigned int _sgot;
extern unsigned int _egot;
extern unsigned int _text;

#if defined(TARGET_NANOS)
#define NVM_PAGE_SIZE 64
#elif defined(TARGET_NANOX)
#define NVM_PAGE_SIZE 256
#elif defined(TARGET_NANOS2)
#define NVM_PAGE_SIZE 512
#endif
#define NVM_PAGE_SIZE_INT  NVM_PAGE_SIZE / 4

uint32_t read_r9() {
  uint32_t reg;
  __asm volatile ("mov %0, r9" : "=r" (reg));
  return reg;
}

// These assume that the code and data sections have distinctive bit patterns in
// at the top, so that they can be easily told apart from lengths
#define IS_TEXT(x) (((x & 0xff000000) == ((uint32_t)(&_text) & 0xff000000)))
#define IS_BSS(x) (((x & 0xffff0000) == ((uint32_t)(&_bss) & 0xffff0000)))

void relocate(uint32_t _real_bss) {
    uint32_t nvm_page_buffer[NVM_PAGE_SIZE_INT];
    uint32_t _real_text = (uint32_t)pic_internal(&_text);
    int _text_offset = _real_text - (uint32_t)(&_text);
    int _bss_offset = _real_bss - (uint32_t)(&_data);

    uint32_t * end = (uint32_t*)pic_internal(&_egot);
    uint32_t * p = (uint32_t*)pic_internal(&_sgot);
    while (p < end) {
      size_t length = MIN(NVM_PAGE_SIZE_INT, end-p);
      size_t i = 0;
      for (;i < length; i++) {
        if IS_BSS(p[i]) {
          nvm_page_buffer[i] = p[i] + _bss_offset;
        } else if IS_TEXT(p[i]) {
          nvm_page_buffer[i] = p[i] + _text_offset;
        } else {
          nvm_page_buffer[i] = p[i];
        }
      }
      // no need to pad the page_buffer with 0 or original data
      // nvm_write() will handle that on its own
      nvm_write(p, nvm_page_buffer, length*4);
      p += NVM_PAGE_SIZE_INT;
    }

    end = (uint32_t*)&_edatarelro;
    p = (uint32_t*)&_sdatarelro;
    while (p < end) {
      size_t length = MIN(NVM_PAGE_SIZE_INT, end-p);
      for (size_t i = 0; i < length; i++) {
        if IS_BSS(p[i]) {
          nvm_page_buffer[i] = p[i] + _bss_offset;
        } else if IS_TEXT(p[i]) {
          nvm_page_buffer[i] = p[i] + _text_offset;
        } else {
          nvm_page_buffer[i] = p[i];
        }
      }
      nvm_write(p, nvm_page_buffer, length*4);
      p += NVM_PAGE_SIZE_INT;
    }
}

#ifdef HAVE_CCID
 #include "usbd_ccid_if.h"
uint8_t G_io_apdu_buffer[260];
#endif

int c_main(void) {
  __asm volatile("cpsie i");

  // point r9 to .got section

  // save ram address for future usage
  // volatile uint32_t _ram_addr = read_r9();
  __asm volatile ("mov r8, r9");

  // Use asm to fetch got's link address and
  // apply pic_internal manually before setting
  // r9 to it
  __asm volatile ("ldr r0, =_sgot");
  __asm volatile ("bl pic_internal");
  __asm volatile ("mov r9, r0");
  __asm volatile ("mov r0, r8");
  __asm volatile ("bl relocate");

  // formerly known as 'os_boot()'
  try_context_set(NULL);

  // TODO: add a way to detect when the app was moved in flash (uninstall of another app)
  // with some marker in nvm data that would conditionally trigger `relocate()`

  // copy .data section
  uint32_t * src = (uint32_t*)&_sidata;
  uint32_t * dst = (uint32_t*)&_data;
  uint32_t * end = (uint32_t*)&_edata;
  while (dst < end) {
    *dst++ = *src++;
  }

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

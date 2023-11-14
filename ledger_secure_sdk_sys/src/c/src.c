#include "exceptions.h"
#include "os_apilevel.h"
#include "string.h"
#include "seproxyhal_protocol.h"
#include "os_id.h"
#include "os_io_usb.h"
#include "checks.h"
#ifdef HAVE_BLE
  #include "ledger_ble.h"
#endif

extern void sample_main();

io_seph_app_t G_io_app;

#ifdef HAVE_CCID
 #include "usbd_ccid_if.h"
uint8_t G_io_apdu_buffer[260];
#endif

int c_main(void) {
  __asm volatile("cpsie i");

  // formerly known as 'os_boot()'
  try_context_set(NULL);

  for(;;) {
    BEGIN_TRY {
      TRY {
        // below is a 'manual' implementation of `io_seproxyhal_init`
        check_api_level(CX_COMPAT_APILEVEL);
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

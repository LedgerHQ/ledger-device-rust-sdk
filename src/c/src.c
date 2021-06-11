#include "os.h"
#include "os_io_seproxyhal.h"




extern void sample_main();

void os_longjmp(unsigned int exception) {
  longjmp(try_context_get()->jmp_buf, exception);
}

io_seph_app_t G_io_app;

int c_main(void) {
  __asm volatile("cpsie i");
  unsigned int r9_reg = pic_internal(0xc0d00000);
  __asm volatile("mov r9, %0":"=r"(r9_reg)::"r9");

  // formerly known as 'os_boot()'
  try_context_set(NULL);

  for(;;) {
    BEGIN_TRY {
      TRY {
        // below is a 'manual' implementation of `io_seproxyhal_init`
        check_api_level(CX_COMPAT_APILEVEL);

        memset(&G_io_app, 0, sizeof(G_io_app));

        G_io_app.apdu_state = APDU_IDLE;
        G_io_app.apdu_length = 0;
        G_io_app.apdu_media = IO_APDU_MEDIA_NONE;

        G_io_app.ms = 0;
        io_usb_hid_init();

        USB_power(0);
        USB_power(1);
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
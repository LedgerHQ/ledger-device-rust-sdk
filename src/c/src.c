#include "os.h"
#include "os_io_seproxyhal.h"

extern void sample_main();

unsigned char G_io_seproxyhal_spi_buffer[128] = {0};
io_seph_app_t G_io_app;

int c_main(void) {
  __asm volatile("cpsie i");
  unsigned int r9_reg = pic_internal(0xc0d00000);
  __asm volatile("mov r9, %0":"=r"(r9_reg)::"r9");

  os_boot();
  for(;;) {
    BEGIN_TRY {
      TRY {
        io_seproxyhal_init();
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
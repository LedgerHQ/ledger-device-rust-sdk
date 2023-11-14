#include "bolos_target.h"

//  Definitions common to both `cc` and `bindgen`
#define HAVE_LOCAL_APDU_BUFFER
#define IO_HID_EP_LENGTH 64
#define USB_SEGMENT_SIZE 64
#define OS_IO_SEPROXYHAL
#define HAVE_IO_USB
#define HAVE_L4_USBLIB
#define HAVE_USB_APDU
#define __IO volatile
#define IO_USB_MAX_ENDPOINTS 6
#define IO_SEPROXYHAL_BUFFER_SIZE_B 128

#if defined(TARGET_NANOX)
#define HAVE_BLE
#define HAVE_BLE_APDU
#endif

// #define HAVE_USB_CLASS_CCID
// #define HAVE_CCID

#if defined(TARGET_NANOX) || defined(TARGET_NANOS2)
#define HAVE_SEPROXYHAL_MCU
#define HAVE_MCU_PROTECT
#define HAVE_MCU_SEPROXYHAL
#define HAVE_MCU_SERIAL_STORAGE
#define HAVE_SE_BUTTON
#define HAVE_BAGL
#define HAVE_SE_SCREEN
#endif

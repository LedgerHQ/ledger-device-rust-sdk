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
#define main _start

#define HAVE_SEPROXYHAL_MCU
#define HAVE_MCU_PROTECT
#define HAVE_MCU_SEPROXYHAL
#define HAVE_MCU_SERIAL_STORAGE
#define HAVE_SE_BUTTON
#define HAVE_BAGL
#define HAVE_SE_SCREEN

#define HAVE_BLE
#define HAVE_BLE_APDU

#if defined(TARGET_STAX)
#define HAVE_NBGL
#define NBGL_USE_CASE
#endif

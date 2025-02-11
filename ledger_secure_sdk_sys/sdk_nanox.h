// Standard Defines
#define IO_HID_EP_LENGTH 64
#define HAVE_SPRINTF
#define HAVE_SNPRINTF_FORMAT_U
#define HAVE_IO_USB
#define HAVE_L4_USBLIB
#define IO_USB_MAX_ENDPOINTS 4
#define HAVE_USB_APDU
#define USB_SEGMENT_SIZE 64
#define OS_IO_SEPROXYHAL
#define HAVE_LOCAL_APDU_BUFFER
#define IO_SEPROXYHAL_BUFFER_SIZE_B 300
#define __IO volatile
#define main _start

#define HAVE_SE_BUTTON
#define HAVE_SE_SCREEN
#define HAVE_FONTS

#define BAGL_HEIGHT 64
#define BAGL_WIDTH 128
#define HAVE_BAGL_ELLIPSIS
#define HAVE_BAGL_FONT_OPEN_SANS_REGULAR_11PX
#define HAVE_BAGL_FONT_OPEN_SANS_EXTRABOLD_11PX
#define HAVE_BAGL_FONT_OPEN_SANS_LIGHT_16PX
#define SCREEN_SIZE_NANO

#define HAVE_SEPROXYHAL_MCU
#define HAVE_MCU_PROTECT

// BLE SUPPORT  
#define HAVE_BLE
#define HAVE_BLE_APDU
#define BLE_COMMAND_TIMEOUT_MS 2000
#define BLE_SEGMENT_SIZE 32
#define HAVE_INAPP_BLE_PAIRING

// WEB USB (not supported in Rust SDK)
//#define HAVE_WEBUSB 
//#define WEBUSB_URL_SIZE_B 
//#define WEBUSB_URL

// APP STORAGE (feature dependent)
//#define HAVE_APP_STORAGE

// NBGL KEYBOARD (feature dependent)
//#define NBGL_KEYBOARD

// NBGL KEYPAD (feature dependent)
//#define NBGL_KEYPAD
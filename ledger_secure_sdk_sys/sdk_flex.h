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

// NBGL
#define HAVE_BAGL_FONT_INTER_REGULAR_28PX
#define HAVE_BAGL_FONT_INTER_SEMIBOLD_28PX
#define HAVE_BAGL_FONT_INTER_MEDIUM_36PX
#define HAVE_INAPP_BLE_PAIRING
#define HAVE_NBGL
#define HAVE_PIEZO_SOUND
#define HAVE_SE_TOUCH
#define HAVE_SE_EINK_DISPLAY
#define NBGL_PAGE
#define NBGL_USE_CASE
#define SCREEN_SIZE_WALLET
#define HAVE_FAST_HOLD_TO_APPROVE

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

// NFC SUPPORT (feature dependent)
//#define HAVE_NFC
//#define HAVE_NFC_READER

// APP STORAGE (feature dependent)
//#define HAVE_APP_STORAGE

// NBGL QRCODE (feature dependent)
#define NBGL_QRCODE

// NBGL KEYBOARD (feature dependent)
//#define NBGL_KEYBOARD

// NBGL KEYPAD (feature dependent)
//#define NBGL_KEYPAD

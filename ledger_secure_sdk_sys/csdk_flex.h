////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// Makefile.standard_app
//////////////////////////////////////////////////////////////////////////////////////////////////////////////////////// 
// BLUETOOTH
#define HAVE_BLE
#define HAVE_BLE_APDU
#define BLE_COMMAND_TIMEOUT_MS 2000
#define BLE_SEGMENT_SIZE 32
// NFC SUPPORT (feature dependent)
//#define HAVE_NFC
//#define HAVE_NFC_READER
// APP STORAGE (feature dependent)
//#define HAVE_APP_STORAGE
// IO SEPROXY BUFFER SIZE
#define IO_SEPROXYHAL_BUFFER_SIZE_B 300
// NBGL QRCODE (feature dependent)
#define NBGL_QRCODE
// NBGL KEYBOARD (feature dependent)
//#define NBGL_KEYBOARD
// NBGL KEYPAD (feature dependent)
//#define NBGL_KEYPAD
// STANDARD DEFINES
#define IO_HID_EP_LENGTH 64
#define HAVE_SPRINTF
#define HAVE_SNPRINTF_FORMAT_U
#define HAVE_IO_USB
#define HAVE_L4_USBLIB
#define IO_USB_MAX_ENDPOINTS 4
#define HAVE_USB_APDU
#define USB_SEGMENT_SIZE 64
//#define HAVE_WEBUSB 
//#define WEBUSB_URL_SIZE_B 
//#define WEBUSB_URL
#define OS_IO_SEPROXYHAL
////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// Makefile.defines
////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
#define gcc
#define __IO volatile
// Flex
#define HAVE_BAGL_FONT_INTER_REGULAR_28PX
#define HAVE_BAGL_FONT_INTER_SEMIBOLD_28PX
#define HAVE_BAGL_FONT_INTER_MEDIUM_36PX
#define HAVE_INAPP_BLE_PAIRING
#define HAVE_NBGL
#define HAVE_PIEZO_SOUND
#define HAVE_SE_TOUCH
#define HAVE_SE_EINK_DISPLAY
//#define HAVE_HW_TOUCH_SWIPE
#define NBGL_PAGE
#define NBGL_USE_CASE
#define SCREEN_SIZE_WALLET
#define HAVE_FAST_HOLD_TO_APPROVE

#define HAVE_LEDGER_PKI

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// Misc
////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
#define HAVE_LOCAL_APDU_BUFFER
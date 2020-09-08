#include "glyphs.h"
unsigned int const C_icon_down_colors[] = {
  0x00000000,
  0x00ffffff,
};

unsigned char const C_icon_down_bitmap[] = {
  0x41, 0x11, 0x05, 0x01,
};
#ifdef OS_IO_SEPROXYHAL
        #include "os_io_seproxyhal.h"
        const bagl_icon_details_t C_icon_down = { GLYPH_icon_down_WIDTH, GLYPH_icon_down_HEIGHT, 1, C_icon_down_colors, C_icon_down_bitmap };
        #endif // OS_IO_SEPROXYHAL
#include "glyphs.h"
unsigned int const C_icon_left_colors[] = {
  0x00000000,
  0x00ffffff,
};

unsigned char const C_icon_left_bitmap[] = {
  0x48, 0x12, 0x42, 0x08,
};
#ifdef OS_IO_SEPROXYHAL
        #include "os_io_seproxyhal.h"
        const bagl_icon_details_t C_icon_left = { GLYPH_icon_left_WIDTH, GLYPH_icon_left_HEIGHT, 1, C_icon_left_colors, C_icon_left_bitmap };
        #endif // OS_IO_SEPROXYHAL
#include "glyphs.h"
unsigned int const C_icon_right_colors[] = {
  0x00000000,
  0x00ffffff,
};

unsigned char const C_icon_right_bitmap[] = {
  0x21, 0x84, 0x24, 0x01,
};
#ifdef OS_IO_SEPROXYHAL
        #include "os_io_seproxyhal.h"
        const bagl_icon_details_t C_icon_right = { GLYPH_icon_right_WIDTH, GLYPH_icon_right_HEIGHT, 1, C_icon_right_colors, C_icon_right_bitmap };
        #endif // OS_IO_SEPROXYHAL
#include "glyphs.h"
unsigned int const C_icon_up_colors[] = {
  0x00000000,
  0x00ffffff,
};

unsigned char const C_icon_up_bitmap[] = {
  0x08, 0x8a, 0x28, 0x08,
};
#ifdef OS_IO_SEPROXYHAL
        #include "os_io_seproxyhal.h"
        const bagl_icon_details_t C_icon_up = { GLYPH_icon_up_WIDTH, GLYPH_icon_up_HEIGHT, 1, C_icon_up_colors, C_icon_up_bitmap };
        #endif // OS_IO_SEPROXYHAL

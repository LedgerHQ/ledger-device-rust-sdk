#ifdef HAVE_RIPEMD160

#ifndef CX_RIPEMD160_H
#define CX_RIPEMD160_H

#include "lcx_ripemd160.h"
#include <stdbool.h>
#include <stddef.h>

cx_err_t cx_ripemd160_update(cx_ripemd160_t *ctx, const uint8_t *data, size_t len);
void cx_ripemd160_final(cx_ripemd160_t *ctx, uint8_t *digest);
bool   cx_ripemd160_validate_context(const cx_ripemd160_t *ctx);

#endif // HAVE_RIPEMD160

#endif // CX_RIPEMD160_H

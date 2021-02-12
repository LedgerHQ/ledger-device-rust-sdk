#if defined(HAVE_SHA512) || defined(HAVE_SHA384)

#ifndef CX_SHA512_H
#define CX_SHA512_H

#include <stdbool.h>
#include <stddef.h>
#include "lcx_sha512.h"

cx_err_t cx_sha512_update(cx_sha512_t *ctx, const uint8_t *data, size_t len);
void cx_sha512_final(cx_sha512_t *ctx, uint8_t *digest);
bool   cx_sha512_validate_context(const cx_sha512_t *ctx);

#endif // CX_SHA512_H

#endif // defined(HAVE_SHA512) || defined(HAVE_SHA384)

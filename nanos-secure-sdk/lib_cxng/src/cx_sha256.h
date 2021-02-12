#if defined(HAVE_SHA256) || defined(HAVE_SHA224)

#ifndef CX_SHA256_H
#define CX_SHA256_H

#include "lcx_sha256.h"
#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>

cx_err_t cx_sha256_update(cx_sha256_t *ctx, const uint8_t *data, size_t len);
void cx_sha256_final(cx_sha256_t *ctx, uint8_t *digest);
bool   cx_sha256_validate_context(const cx_sha256_t *ctx);

#endif // CX_SHA256_H
#endif // HAVE_SHA256

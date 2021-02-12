
/*******************************************************************************
*   Ledger Nano S - Secure firmware
*   (c) 2021 Ledger
*
*  Licensed under the Apache License, Version 2.0 (the "License");
*  you may not use this file except in compliance with the License.
*  You may obtain a copy of the License at
*
*      http://www.apache.org/licenses/LICENSE-2.0
*
*  Unless required by applicable law or agreed to in writing, software
*  distributed under the License is distributed on an "AS IS" BASIS,
*  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
*  See the License for the specific language governing permissions and
*  limitations under the License.
********************************************************************************/

#ifdef HAVE_HASH

#ifndef CX_HASH_H
#define CX_HASH_H

#include "lcx_hash.h"
#include <stddef.h>

#ifdef HAVE_RIPEMD160
#define RIPEMD_BLOCK_SIZE 64
#endif

#if defined(HAVE_SHA256) || defined(HAVE_SHA224)
#define SHA256_BLOCK_SIZE 64
#endif

#if defined(HAVE_SHA512) || defined(HAVE_SHA384)
#define SHA512_BLOCK_SIZE 128
#endif

//#warning reduce MAX_HASH_SIZE and MAX_HASH_BLOCK_SIZE according to HAVE_xxx
#define MAX_HASH_SIZE 128
#define MAX_HASH_BLOCK_SIZE 128

#ifdef HAVE_GROESTL
cx_err_t cx_groestl(cx_hash_t *hash, uint32_t mode, const uint8_t *in, size_t in_len, uint8_t *out, size_t out_len);
#endif

#ifdef HAVE_BLAKE2
cx_err_t cx_blake2b(cx_hash_t *hash, uint32_t mode, const uint8_t *in, size_t in_len, uint8_t *out, size_t out_len);
#endif

/* Generic API */
typedef struct {
  cx_md_t md_type;
  size_t  output_size;
  size_t  block_size;
  void (*init_func)(cx_hash_t *ctx);
  cx_err_t (*update_func)(cx_hash_t *ctx, const uint8_t *data, size_t len);
  void (*finish_func)(cx_hash_t *ctx, uint8_t *digest);
  cx_err_t (*init_ex_func)(cx_hash_t *ctx, size_t output_size);
  bool (*validate_context)(const cx_hash_t *ctx);
  size_t (*output_size_func)(const cx_hash_t *ctx);
} cx_hash_info_t;

const cx_hash_info_t *cx_hash_get_info(cx_md_t md_type);

#endif

#endif // HAVE_HASH

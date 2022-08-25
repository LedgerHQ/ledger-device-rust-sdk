
/*******************************************************************************
*   Ledger Nano S - Secure firmware
*   (c) 2022 Ledger
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
#if defined(HAVE_CHACHA_POLY)
#if defined(HAVE_POLY1305) && defined(HAVE_CHACHA)

/**
 * @file    lcx_chacha_poly.h
 * @brief   CHACHA20_POLY1305 Authenticated Encryption with Additional Data (AEAD)
 *
 * @section Description
 *
 * @author  Ledger
 * @version 1.0
 **/


#ifndef CX_CHACHA_POLY_H
#define CX_CHACHA_POLY_H

#include "lcx_aead.h"
#include "lcx_chacha.h"
#include "lcx_chacha_poly.h"
#include "cx_poly1305.h"
#include "ox.h"
#include <stddef.h>

extern const cx_aead_info_t cx_chacha20_poly1305_info;

void cx_chachapoly_init(cx_chachapoly_context_t *ctx);

cx_err_t cx_chachapoly_set_key(cx_chachapoly_context_t *ctx,
                              const uint8_t *key, size_t key_len);

cx_err_t cx_chachapoly_start(cx_chachapoly_context_t *ctx, uint32_t mode,
                               const uint8_t *iv, size_t iv_len);

cx_err_t cx_chachapoly_update_aad(cx_chachapoly_context_t *ctx,
                                   const uint8_t *aad,
                                   size_t aad_len);

cx_err_t cx_chachapoly_update(cx_chachapoly_context_t *ctx,
                               const uint8_t *input,
                               uint8_t *output, size_t len);

cx_err_t cx_chachapoly_finish(cx_chachapoly_context_t *ctx,
                               uint8_t *tag, size_t tag_len);

cx_err_t cx_chachapoly_encrypt_and_tag(cx_chachapoly_context_t *ctx,
                                       const uint8_t *input, size_t len,
                                       const uint8_t *iv, size_t iv_len,
                                       const uint8_t *aad, size_t aad_len,
                                       uint8_t *output, uint8_t *tag, size_t tag_len);

cx_err_t cx_chachapoly_decrypt_and_auth(cx_chachapoly_context_t *ctx,
                                    const uint8_t *input, size_t len,
                                    const uint8_t *iv, size_t iv_len,
                                    const uint8_t *aad, size_t aad_len,
                                    uint8_t *output, const uint8_t *tag, size_t tag_len);

#endif /* CX_CHACHA_POLY_H */
#endif // HAVE_POLY1305 && HAVE_CHACHA
#endif // HAVE_CHACHA_POLY

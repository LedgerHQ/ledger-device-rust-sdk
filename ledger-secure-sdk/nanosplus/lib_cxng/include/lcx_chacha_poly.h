
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


#ifndef LCX_CHACHA_POLY_H
#define LCX_CHACHA_POLY_H

#include "lcx_chacha.h"
#include "lcx_poly1305.h"
#include "ox.h"
#include <stddef.h>

typedef struct {
    cx_chacha_context_t   chacha20_ctx;   ///< The ChaCha20 context.
    cx_poly1305_context_t poly1305_ctx;   ///< The Poly1305 context.
    size_t                aad_len;        ///< The length in bytes of the Additional Authenticated Data.
    size_t                ciphertext_len; ///< The length in bytes of the ciphertext.
    uint32_t              state;          ///< The current state of the context.
    uint32_t              mode;           ///< Cipher mode (encrypt or decrypt).
} cx_chachapoly_context_t;

#endif /* LCX_CHACHA_POLY_H */
#endif // HAVE_POLY1305 && HAVE_CHACHA
#endif // HAVE_CHACHA_POLY

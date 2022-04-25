
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

/**
 * @file    lcx_aes_gcm.h
 * @brief   AES in Galois/Counter Mode (AES-GCM)
 *
 * @section Description
 *
 * The Galois/Counter Mode (GCM) is an authenticated encryption algorithm
 * designed to provide both data authenticity (integrity) and confidentiality.
 * Refer to SP 800-38D for more details.
 *
 * @author  Ledger
 * @version 1.0
 **/

#if defined(HAVE_AES) && (HAVE_AES_GCM)

#ifndef LCX_AES_GCM_H
#define LCX_AES_GCM_H

#include "ox.h"
#include <stddef.h>

/**
 * @brief AES-GCM context
 */
typedef struct {
  cx_aes_key_t key;           ///< AES key
  size_t       len;           ///< Input length
  size_t       aad_len;       ///< Additional data length
  uint8_t      enc_block[16]; ///< First encrypted block used to compute the tag
  uint8_t      J0[16];        ///< Counter
  uint8_t      processed[16]; ///< Processed data
  uint8_t      hash_key[16];  ///< Ghash key
  uint32_t     mode;          ///< Encrypt or decrypt
  uint8_t      flag;          ///< Indicates either the IV has already been processed or not
} cx_aes_gcm_context_t;


#endif
#endif // HAVE_AES && HAVE_AES_GCM

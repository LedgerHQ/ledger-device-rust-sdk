
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

#ifdef HAVE_ECSCHNORR

#include "cx_rng.h"
#include "cx_ecfp.h"
#include "cx_eddsa.h"
#include "cx_hash.h"
#include "cx_utils.h"
#include "cx_ram.h"

#include <string.h>

/* ----------------------------------------------------------------------- */
/*                                                                         */
/* ----------------------------------------------------------------------- */
// const char kr[] = {0xe5, 0xa8, 0xd1, 0xd5, 0x29, 0x97, 0x1c, 0x10, 0xca, 0x2a, 0xf3, 0x78, 0x44, 0x4f, 0xb5, 0x44,
// 0xa2, 0x11, 0x70, 0x78, 0x92, 0xc8, 0x89, 0x8f, 0x91, 0xdc, 0xb1, 0x71, 0x58, 0x4e, 0x3d, 0xb9};

cx_err_t cx_ecschnorr_sign_no_throw(const cx_ecfp_private_key_t *pv_key,
                           uint32_t                     mode,
                           cx_md_t                      hashID,
                           const uint8_t *              msg,
                           size_t                       msg_len,
                           uint8_t *                    sig,
                           size_t *                     sig_len) {
#define CX_MAX_TRIES 100
#define H G_cx.sha256

  size_t                        size;
  cx_ecpoint_t                  Q;
  cx_bn_t                       bn_k, bn_d, bn_r, bn_s, bn_n;
  uint8_t                       R[33];
  uint8_t                       S[32];
  int                           odd;
  uint8_t                       tries;
  cx_err_t                      error;
  int                           diff;

  CX_CHECK(cx_ecdomain_parameters_length(pv_key->curve, &size));

  // WARN: only accept weierstrass 256 bits curve for now
  if (hashID != CX_SHA256 || size != 32 ||
      !CX_CURVE_RANGE(pv_key->curve, WEIERSTRASS) ||
      *sig_len < (6 + 2 * (size + 1)) || pv_key->d_len != size) {
    return CX_INVALID_PARAMETER;
  }

  CX_CHECK(cx_bn_lock(size, 0));
  CX_CHECK(cx_bn_alloc(&bn_n, size));
  CX_CHECK(cx_ecdomain_parameter_bn(pv_key->curve, CX_CURVE_PARAM_Order, &bn_n));
  CX_CHECK(cx_bn_alloc(&bn_k, size));
  CX_CHECK(cx_bn_alloc(&bn_d, size));
  CX_CHECK(cx_bn_alloc(&bn_r, size));
  CX_CHECK(cx_bn_alloc(&bn_s, size));
  CX_CHECK(cx_ecpoint_alloc(&Q, pv_key->curve));

  // generate random
  tries = 0;
 RETRY:
  if (tries == CX_MAX_TRIES) {
    goto end;
  }

  switch (mode & CX_MASK_RND) {
  case CX_RND_PROVIDED:
    if (tries) {
      goto end;
    }
    CX_CHECK(cx_bn_init(&bn_d, sig, size));
    break;

  case CX_RND_TRNG:
    CX_CHECK(cx_bn_rand(&bn_d));
    break;

  default:
    error = CX_INVALID_PARAMETER;
    goto end;
  }
  CX_CHECK(cx_bn_reduce(&bn_k, &bn_d, &bn_n));
  CX_CHECK(cx_bn_export(&bn_k, sig, size));

  // sign
  tries++;
 RETRY2:
  CX_CHECK(cx_ecdomain_generator_bn(pv_key->curve, &Q));
  CX_CHECK(cx_ecpoint_scalarmul(&Q, sig, size));

  switch (mode & CX_MASK_EC) {
  case CX_ECSCHNORR_ISO14888_XY:
  case CX_ECSCHNORR_ISO14888_X:
    // 1. Generate a random k from [1, ..., order-1]
    // 2. Q = G*k
    // 3. r = H(Q.x||Q.y||M)
    // 4. s = (k+r*pv_key.d)%n
    cx_sha256_init_no_throw(&H);
    CX_CHECK(cx_ecpoint_export(&Q, sig, size, NULL, 0));
    cx_hash_no_throw((cx_hash_t *)&H, 0, sig, size, NULL, 0);
    if ((mode & CX_MASK_EC) == CX_ECSCHNORR_ISO14888_XY) {
      CX_CHECK(cx_ecpoint_export(&Q, NULL, 0, sig, size));
      cx_hash_no_throw((cx_hash_t *)&H, 0, sig, size, NULL, 0);
    }
    cx_hash_no_throw((cx_hash_t *)&H, CX_LAST | CX_NO_REINIT, msg, msg_len, R, sizeof(R));

    CX_CHECK(cx_bn_init(&bn_d, R, 32));
    CX_CHECK(cx_bn_reduce(&bn_r, &bn_d, &bn_n));
    CX_CHECK(cx_bn_cmp_u32(&bn_r, 0, &diff));
    if (diff == 0) {
      cx_bn_unlock();
      goto RETRY;
    }

    CX_CHECK(cx_bn_init(&bn_d, pv_key->d, pv_key->d_len));
    CX_CHECK(cx_bn_mod_mul(&bn_s, &bn_d, &bn_r, &bn_n));
    CX_CHECK(cx_bn_mod_add(&bn_s, &bn_k, &bn_s, &bn_n));

    CX_CHECK(cx_bn_cmp_u32(&bn_s, 0, &diff));
    if (diff == 0) {
      goto RETRY;
    }
    CX_CHECK(cx_bn_export(&bn_s, S, 32));
    break;

  case CX_ECSCHNORR_BSI03111:
    // 1. Q = G*k
    // 2. r = H((msg+xQ), and r%n != 0
    // 3. s = (k-r*pv_key.d)%n
    // r = H((msg+xQ), and r%n != 0
    cx_sha256_init_no_throw(&H);
    cx_hash_no_throw((cx_hash_t *)&H, 0, msg, msg_len, NULL, 0);
    CX_CHECK(cx_ecpoint_export(&Q, sig, size, NULL, 0));
    cx_hash_no_throw((cx_hash_t *)&H, CX_LAST | CX_NO_REINIT, sig, size, R, sizeof(R));

    CX_CHECK(cx_bn_init(&bn_d, R, CX_SHA256_SIZE));
    CX_CHECK(cx_bn_reduce(&bn_r, &bn_d, &bn_n));
    CX_CHECK(cx_bn_cmp_u32(&bn_r, 0, &diff));
    if (diff == 0) {
      goto RETRY;
    }

    // s = (k-r*pv_key.d)%n
    CX_CHECK(cx_bn_init(&bn_d, pv_key->d, pv_key->d_len));
    CX_CHECK(cx_bn_mod_mul(&bn_s, &bn_d, &bn_r, &bn_n));
    CX_CHECK(cx_bn_mod_sub(&bn_s, &bn_k, &bn_s, &bn_n));

    CX_CHECK(cx_bn_cmp_u32(&bn_s, 0, &diff));
    if (diff == 0) {
      goto RETRY;
    }
    CX_CHECK(cx_bn_export(&bn_s, S, 32));
    break;

  case CX_ECSCHNORR_Z:
    // https://github.com/Zilliqa/Zilliqa/blob/master/src/libCrypto/Schnorr.cpp#L580
    // https://docs.zilliqa.com/whitepaper.pdf
    // 1. Generate a random k from [1, ..., order-1]
    // 2. Compute the commitment Q = kG, where  G is the base point
    // 3. Compute the challenge r = H(Q, kpub, m) [CME: mod n according to pdf/code, Q and kpub compressed "02|03 x"
    // according to code)
    // 4. If r = 0 mod(order), goto 1
    // 4. Compute s = k - r*kpriv mod(order)
    // 5. If s = 0 goto 1.
    // 5  Signature on m is (r, s)

    // Q
    cx_sha256_init_no_throw(&H);
    CX_CHECK(cx_ecpoint_export(&Q, NULL, 0, sig, size));
    odd = sig[size - 1] & 1;
    CX_CHECK(cx_ecpoint_export(&Q, sig + 1, size, NULL, 0));
    sig[0] = odd ? 0x03 : 0x02;
    cx_hash_no_throw((cx_hash_t *)&H, 0, sig, 1 + size, NULL, 0); // Q
    // kpub
    CX_CHECK(cx_ecdomain_generator_bn(pv_key->curve, &Q));
    CX_CHECK(cx_ecpoint_scalarmul(&Q, pv_key->d, pv_key->d_len));
    CX_CHECK(cx_ecpoint_export(&Q, NULL, 0, sig, size));
    odd = sig[size - 1] & 1;
    CX_CHECK(cx_ecpoint_export(&Q, sig + 1, size, NULL, 0));
    sig[0] = odd ? 0x03 : 0x02;
    cx_hash_no_throw((cx_hash_t *)&H, 0, sig, 1 + size, NULL, 0); // Q
    // m
    cx_hash_no_throw((cx_hash_t *)&H, CX_LAST | CX_NO_REINIT, msg, msg_len, R, sizeof(R));

    // Compute the challenge r = H(Q, kpub, m)
    //[CME: mod n according to pdf/code, Q and kpub compressed "02|03 x" according to code)
    CX_CHECK(cx_bn_init(&bn_d, R, CX_SHA256_SIZE));
    CX_CHECK(cx_bn_reduce(&bn_r, &bn_d, &bn_n));
    CX_CHECK(cx_bn_cmp_u32(&bn_r, 0, &diff));
    if (diff == 0) {
      goto RETRY;
    }
    CX_CHECK(cx_bn_export(&bn_r, R, 32));

    CX_CHECK(cx_bn_init(&bn_d, pv_key->d, pv_key->d_len));
    CX_CHECK(cx_bn_mod_mul(&bn_s, &bn_d, &bn_r, &bn_n));
    CX_CHECK(cx_bn_mod_sub(&bn_s, &bn_k, &bn_s, &bn_n));
    CX_CHECK(cx_bn_cmp_u32(&bn_s, 0, &diff));
    if (diff == 0) {
      goto RETRY;
    }
    CX_CHECK(cx_bn_export(&bn_s, S, 32));
    break;

  case CX_ECSCHNORR_LIBSECP:
    // Inputs: 32-byte message m, 32-byte scalar key x (!=0), 32-byte scalar nonce k (!=0)
    // 1. Compute point R = k * G. Reject nonce if R's y coordinate is odd (or negate nonce).
    // 2. Compute 32-byte r, the serialization of R's x coordinate.
    // 3. Compute scalar h = Hash(r || m). Reject nonce if h == 0 or h >= order.
    // 4. Compute scalar s = k - h * x.
    // 5. The signature is (r, s).
    // Q = G*k
    CX_CHECK(cx_ecpoint_export(&Q, NULL, 0, sig, size));
    odd = sig[size - 1] & 1;
    if (odd) {
      // if y is odd, k <- -k mod n = n-k,  and retry
      CX_CHECK(cx_bn_mod_sub(&bn_k, &bn_n, &bn_k, &bn_n));
      CX_CHECK(cx_bn_export(&bn_k, sig, size));
      goto RETRY2;
    }
    // r = xQ
    CX_CHECK(cx_ecpoint_export(&Q, R, size, NULL, 0));
    CX_CHECK(cx_bn_init(&bn_d, R, size));
    CX_CHECK(cx_bn_reduce(&bn_r, &bn_d, &bn_n));
    CX_CHECK(cx_bn_cmp_u32(&bn_r, 0, &diff));
    if (diff == 0) {
      goto RETRY;
    }
    // h = Hash(r || m).
    cx_sha256_init_no_throw(&H);
    cx_hash_no_throw((cx_hash_t *)&H, 0, R, size, NULL, 0);
    cx_hash_no_throw((cx_hash_t *)&H, CX_LAST | CX_NO_REINIT, msg, msg_len, sig, sizeof(S));
    // Reject nonce if h == 0 or h >= order.
    CX_CHECK(cx_bn_init(&bn_r, sig, 32));
    CX_CHECK(cx_bn_cmp_u32(&bn_r, 0, &diff));
    if (diff == 0) {
      goto RETRY;
    }
    CX_CHECK(cx_bn_cmp(&bn_r, &bn_n, &diff));
    if (diff >= 0) {
      goto RETRY;
    }
    // s = k - h * x.
    CX_CHECK(cx_bn_init(&bn_d, pv_key->d, pv_key->d_len));
    CX_CHECK(cx_bn_mod_mul(&bn_s, &bn_d, &bn_r, &bn_n));
    CX_CHECK(cx_bn_mod_sub(&bn_s, &bn_k, &bn_s, &bn_n));
    CX_CHECK(cx_bn_cmp_u32(&bn_s, 0, &diff));
    if (diff == 0) {
      goto RETRY;
    }
    CX_CHECK(cx_bn_export(&bn_s, S, 32));
    break;

  default:
    error = CX_INVALID_PARAMETER;
    goto end;
  }

 end:
  cx_bn_unlock();
  if (error == CX_OK) {
    // encoding
    *sig_len = cx_ecfp_encode_sig_der(sig, *sig_len, R, size, S, size);
  }
  return error;
}

/* ----------------------------------------------------------------------- */
/*                                                                         */
/* ----------------------------------------------------------------------- */
bool cx_ecschnorr_verify(const cx_ecfp_public_key_t *pu_key,
                         uint32_t                    mode,
                         cx_md_t                     hashID,
                         const uint8_t *             msg,
                         size_t                      msg_len,
                         const uint8_t *             sig,
                         size_t                      sig_len) {
  size_t                  size;
  uint8_t *               r, *s;
  size_t                  r_len, s_len;
  cx_ecpoint_t            R, P, Q;
  uint8_t                 x[33];
  bool                    odd;
  volatile int            verified;
  cx_err_t                error;
  int                     diff;

#define H G_cx.sha256
  cx_bn_t bn_d, bn_r, bn_s, bn_n;

  CX_CHECK(cx_ecdomain_parameters_length(pu_key->curve, &size));

  if (!CX_CURVE_RANGE(pu_key->curve, WEIERSTRASS) ||
      hashID != CX_SHA256 || size != 32 ||
      pu_key->W_len != 1 + 2 * size) {
    return false;
  }

  if (!cx_ecfp_decode_sig_der(sig, sig_len, size, &r, &r_len, &s, &s_len)) {
    return false;
  }

  CX_CHECK(cx_bn_lock(size, 0));
  verified = false;

  CX_CHECK(cx_bn_alloc(&bn_n, size));
  CX_CHECK(cx_ecdomain_parameter_bn(pu_key->curve, CX_CURVE_PARAM_Order, &bn_n));
  CX_CHECK(cx_bn_alloc_init(&bn_r, size, r, r_len));
  CX_CHECK(cx_bn_alloc_init(&bn_s, size, s, s_len));
  CX_CHECK(cx_bn_alloc(&bn_d, size));

  CX_CHECK(cx_bn_cmp_u32(&bn_r, 0, &diff));
  if (diff == 0) {
    goto end;
  }
  CX_CHECK(cx_bn_cmp_u32(&bn_s, 0, &diff));
  if (diff == 0) {
    goto end;
  }

  CX_CHECK(cx_ecpoint_alloc(&P, pu_key->curve));
  CX_CHECK(cx_ecpoint_alloc(&Q, pu_key->curve));
  CX_CHECK(cx_ecpoint_alloc(&R, pu_key->curve));
  CX_CHECK(cx_ecdomain_generator_bn(pu_key->curve, &P));
  CX_CHECK(cx_ecpoint_init(&Q, &pu_key->W[1], size, &pu_key->W[1 + size], size));

  switch (mode & CX_MASK_EC) {
  case CX_ECSCHNORR_ISO14888_XY:
  case CX_ECSCHNORR_ISO14888_X:
    // 1. check...
    // 2. Q = [s]G - [r]W
    //   If Q = 0, output Error and terminate.
    //3. v = H(Qx||Qy||M).
    //4. Output True if v = r, and False otherwise.

    // 1.
    CX_CHECK(cx_bn_cmp_u32(&bn_r, 0, &diff));
    if (diff == 0) {
      break;
    }
    CX_CHECK(cx_bn_cmp(&bn_n, &bn_s, &diff));
    if (diff <= 0) {
      break;
    }
    CX_CHECK(cx_bn_cmp_u32(&bn_s, 0, &diff));
    if (diff ==0 ) {
      break;
    }
    CX_CHECK(cx_bn_cmp(&bn_n, &bn_s, &diff));
    if (diff <= 0) {
      break;
    }

    // 2.
    CX_CHECK(cx_ecpoint_scalarmul(&P, s, s_len)); // sG
    CX_CHECK(cx_ecpoint_scalarmul(&Q, r, r_len)); // rW
    CX_CHECK(cx_ecpoint_neg(&Q));
    cx_ecpoint_add(&R, &P, &Q) ;        // sG-rW
    // 3.
    cx_sha256_init_no_throw(&H);
    CX_CHECK(cx_ecpoint_export(&R, x, size, NULL, 0));
    cx_hash_no_throw((cx_hash_t *)&H, 0, x, size, NULL, 0);
    if ((mode & CX_MASK_EC) == CX_ECSCHNORR_ISO14888_XY) {
      CX_CHECK(cx_ecpoint_export(&R, NULL, 0, x, size));
      cx_hash_no_throw((cx_hash_t *)&H, 0, x, size, NULL, 0);
    }
    cx_hash_no_throw((cx_hash_t *)&H, CX_LAST | CX_NO_REINIT, msg, msg_len, x, sizeof(x));
    // 4.
    CX_CHECK(cx_bn_init(&bn_s, x, CX_SHA256_SIZE));
    CX_CHECK(cx_bn_cmp(&bn_r, &bn_s, &diff));
    if (diff == 0) {
      verified = true;
    }
    break;

  case CX_ECSCHNORR_BSI03111:
    // 1. Verify that r in {0, . . . , 2**t - 1} and s in {1, 2, . . . , n - 1}.
    //   If the check fails, output False and terminate.
    // 2. Q = [s]G + [r]W
    //   If Q = 0, output Error and terminate.
    // 3. v = H(M||Qx)
    // 4. Output True if v = r, and False otherwise.

    // 1.
    CX_CHECK(cx_bn_cmp_u32(&bn_r, 0, &diff));
    if (diff == 0) {
      break;
    }
    CX_CHECK(cx_bn_cmp_u32(&bn_s, 0, &diff));
    if (diff == 0) {
      break;
    }
    CX_CHECK(cx_bn_cmp(&bn_n, &bn_s, &diff));
    if (diff <= 0) {
      break;
    }

    // 2.
    CX_CHECK(cx_ecpoint_scalarmul(&P, s, s_len)); // sG
    CX_CHECK(cx_ecpoint_scalarmul(&Q, r, r_len)); // rW
    CX_CHECK(cx_ecpoint_add(&R, &P, &Q));         // sG+rW
    // 3.
    cx_sha256_init_no_throw(&H);
    cx_hash_no_throw((cx_hash_t *)&H, 0, msg, msg_len, NULL, 0);
    CX_CHECK(cx_ecpoint_export(&R, x, size, NULL, 0));
    cx_hash_no_throw((cx_hash_t *)&H, CX_LAST | CX_NO_REINIT, x, size, x, sizeof(x));
    // 4.
    CX_CHECK(cx_bn_init(&bn_s, x, CX_SHA256_SIZE));
    CX_CHECK(cx_bn_cmp(&bn_r, &bn_s, &diff));
    if (diff == 0) {
      verified = true;
    }
    break;

  case CX_ECSCHNORR_Z:
    // The algorithm to check the signature (r, s) on a message m using a public
    // key kpub is as follows
    // 1. Check if r,s is in [1, ..., order-1]
    // 2. Compute Q = sG + r*kpub
    // 3. If Q = O (the neutral point), return 0;
    // 4. r' = H(Q, kpub, m) [CME: mod n and Q and kpub compressed "02|03 x" according to pdf/code]
    // 5. return r' == r

    // r,s is in [1, ..., order-1]
    CX_CHECK(cx_bn_cmp_u32(&bn_r, 0, &diff));
    if (diff == 0) {
      break;
    }
    CX_CHECK(cx_bn_cmp(&bn_r, &bn_n, &diff));
    if (diff >= 0) {
      break;
    }
    CX_CHECK(cx_bn_cmp_u32(&bn_s, 0, &diff));
    if (diff == 0) {
      break;
    }
    CX_CHECK(cx_bn_cmp(&bn_s, &bn_n, &diff));
    if (diff >= 0) {
      break;
    }

    //  Q = sG + r*kpub
    CX_CHECK(cx_ecpoint_scalarmul(&P, s, s_len)); // sG
    CX_CHECK(cx_ecpoint_scalarmul(&Q, r, r_len)); // rW
    CX_CHECK(cx_ecpoint_add(&R, &P, &Q));         // sG+rW
    // r' = H(Q, kpub, m)
    cx_sha256_init_no_throw(&H);
    // Q
    CX_CHECK(cx_ecpoint_export(&R, NULL, 0, x, size));
    odd = x[size - 1] & 1;
    CX_CHECK(cx_ecpoint_export(&R, x + 1, size, NULL, 0));
    x[0] = odd ? 0x03 : 0x02;
    cx_hash_no_throw((cx_hash_t *)&H, 0, x, 1 + size, NULL, 0); // Q
    // kpub
    memmove(x + 1, &pu_key->W[1], size);
    x[0] = (pu_key->W[1 + 2 * size - 1] & 1) ? 0x03 : 0x02;
    cx_hash_no_throw((cx_hash_t *)&H, 0, x, 1 + size, NULL, 0); // kpub
    // m
    cx_hash_no_throw((cx_hash_t *)&H, CX_LAST | CX_NO_REINIT, msg, msg_len, x, sizeof(x)); // m

    CX_CHECK(cx_bn_init(&bn_d, x, CX_SHA256_SIZE));
    CX_CHECK(cx_bn_reduce(&bn_s, &bn_d, &bn_n));
    CX_CHECK(cx_bn_cmp(&bn_r, &bn_s, &diff));
    if (diff == 0) {
      verified = true;
    }
    break;

  case CX_ECSCHNORR_LIBSECP:
    // Verification:
    // Inputs: 32-byte message m, public key point Q, signature: (32-byte r, scalar s)
    // 1. Signature is invalid if s >= order.
    // 2. Signature is invalid if r >= p.
    // 3. Compute scalar h = Hash(r || m). Signature is invalid if h == 0 or h >= order.
    // 4. Option 1 (faster for single verification):
    // 5. Compute point R = h * Q + s * G. Signature is invalid if R is infinity or R's y coordinate is odd.
    // 6. Signature is valid if the serialization of R's x coordinate equals r.
    // s < order and r < field.

    //1. 2.
    CX_CHECK(cx_bn_cmp_u32(&bn_r, 0, &diff));
    if (diff == 0) {
      verified = false;
      break;
    }
    CX_CHECK(cx_bn_cmp_u32(&bn_s, 0, &diff));
    if (diff == 0) {
      verified = false;
      break;
    }
    CX_CHECK(cx_bn_cmp(&bn_n, &bn_s, &diff));
    if (diff <= 0) {
      verified = false;
      break;
    }
    // h = Hash(r||m), and h!=0, and h<order
    cx_sha256_init_no_throw(&H);
    CX_CHECK(cx_bn_export(&bn_r, x, size));
    cx_hash_no_throw((cx_hash_t *)&H, 0, x, size, NULL, 0);
    cx_hash_no_throw((cx_hash_t *)&H, CX_LAST | CX_NO_REINIT, msg, msg_len, x, sizeof(x));
    CX_CHECK(cx_bn_init(&bn_s, x, CX_SHA256_SIZE));
    CX_CHECK(cx_bn_cmp_u32(&bn_s, 0, &diff));
    if (diff == 0) {
      break;
    }
    CX_CHECK(cx_bn_cmp(&bn_s, &bn_n, &diff));
    if (diff >= 0) {
      break;
    }
    // R = h*W + s*G, and Ry is NOT odd, and Rx=r
    CX_CHECK(cx_ecpoint_scalarmul(&P, s, s_len));           // sG
    CX_CHECK(cx_ecpoint_scalarmul(&Q, x, CX_SHA256_SIZE));  // rW
    CX_CHECK(cx_ecpoint_add(&R, &P, &Q));                   // R = R+Q=sG+hW
    CX_CHECK(cx_ecpoint_export_bn(&R, &bn_s, &bn_d));
    CX_CHECK(cx_bn_is_odd(&bn_d, &odd));
    if (odd)  {
      break;
    }
    CX_CHECK(cx_bn_cmp(&bn_r, &bn_s, &diff));
    if (diff == 0) {
      verified = true;
    }
    break;

  default:
    error = CX_INVALID_PARAMETER;
    goto end;
  }

 end:
  cx_bn_unlock();
  return error == CX_OK && verified;
}

#endif // HAVE_ECSCHNORR


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

#ifdef HAVE_GROESTL

#include "cx_utils.h"
#include "cx_ram.h"

#include <string.h>

#include "cx_Groestl-ref.h"

static HashReturn Init(hashState *, int);
static void Update(hashState *, const BitSequence *, DataLength);
static void Final(hashState *, BitSequence *);

cx_err_t cx_groestl_init_no_throw(cx_groestl_t *hash, size_t size) {
  memset(hash, 0, sizeof(cx_groestl_t));
  switch (size) {
  case 224:
  case 256:
  case 384:
  case 512:
    hash->output_size = size >> 3;
    break;
  default:
    return CX_INVALID_PARAMETER;
  }

  if (Init(&hash->ctx, size) != SUCCESS) {
    return CX_INVALID_PARAMETER;
  }
  return CX_OK;
}

cx_err_t cx_groestl(cx_groestl_t *hash, uint32_t mode, const uint8_t *in, size_t len, uint8_t *out, size_t out_len) {
  size_t sz = 0;
  Update(&((cx_groestl_t *)hash)->ctx, in, len);
  if (mode & CX_LAST) {
    sz = ((cx_groestl_t *)hash)->output_size;
    if (out && (out_len < sz)) {
      return CX_INVALID_PARAMETER;
    }
    Final(&((cx_groestl_t *)hash)->ctx, out);
  }
  return CX_OK;
}

cx_err_t cx_groestl_update(cx_groestl_t *ctx, const uint8_t *data, size_t len) {
  if (ctx == NULL) {
    return CX_INVALID_PARAMETER;
  }
  if (data == NULL) {
    return len == 0 ? CX_OK : CX_INVALID_PARAMETER;
  }
  Update(&ctx->ctx, data, len);
  return CX_OK;
}

cx_err_t cx_groestl_final(cx_groestl_t *ctx, uint8_t *digest) {
  Final(&ctx->ctx, digest);
  return CX_OK;
}

size_t cx_groestl_get_output_size(const cx_groestl_t *ctx) {
  return ctx->output_size;
}

/* Groestl-ref.c     January 2011
 * Reference ANSI C code
 * Authors: Soeren S. Thomsen
 *          Krystian Matusiewicz
 *
 * This code is placed in the public domain
 */

#include "cx_Groestl-ref.h"

/* S-box */
static const u8 S[256] = {
    0x63, 0x7c, 0x77, 0x7b, 0xf2, 0x6b, 0x6f, 0xc5, 0x30, 0x01, 0x67, 0x2b, 0xfe, 0xd7, 0xab, 0x76, 0xca, 0x82, 0xc9,
    0x7d, 0xfa, 0x59, 0x47, 0xf0, 0xad, 0xd4, 0xa2, 0xaf, 0x9c, 0xa4, 0x72, 0xc0, 0xb7, 0xfd, 0x93, 0x26, 0x36, 0x3f,
    0xf7, 0xcc, 0x34, 0xa5, 0xe5, 0xf1, 0x71, 0xd8, 0x31, 0x15, 0x04, 0xc7, 0x23, 0xc3, 0x18, 0x96, 0x05, 0x9a, 0x07,
    0x12, 0x80, 0xe2, 0xeb, 0x27, 0xb2, 0x75, 0x09, 0x83, 0x2c, 0x1a, 0x1b, 0x6e, 0x5a, 0xa0, 0x52, 0x3b, 0xd6, 0xb3,
    0x29, 0xe3, 0x2f, 0x84, 0x53, 0xd1, 0x00, 0xed, 0x20, 0xfc, 0xb1, 0x5b, 0x6a, 0xcb, 0xbe, 0x39, 0x4a, 0x4c, 0x58,
    0xcf, 0xd0, 0xef, 0xaa, 0xfb, 0x43, 0x4d, 0x33, 0x85, 0x45, 0xf9, 0x02, 0x7f, 0x50, 0x3c, 0x9f, 0xa8, 0x51, 0xa3,
    0x40, 0x8f, 0x92, 0x9d, 0x38, 0xf5, 0xbc, 0xb6, 0xda, 0x21, 0x10, 0xff, 0xf3, 0xd2, 0xcd, 0x0c, 0x13, 0xec, 0x5f,
    0x97, 0x44, 0x17, 0xc4, 0xa7, 0x7e, 0x3d, 0x64, 0x5d, 0x19, 0x73, 0x60, 0x81, 0x4f, 0xdc, 0x22, 0x2a, 0x90, 0x88,
    0x46, 0xee, 0xb8, 0x14, 0xde, 0x5e, 0x0b, 0xdb, 0xe0, 0x32, 0x3a, 0x0a, 0x49, 0x06, 0x24, 0x5c, 0xc2, 0xd3, 0xac,
    0x62, 0x91, 0x95, 0xe4, 0x79, 0xe7, 0xc8, 0x37, 0x6d, 0x8d, 0xd5, 0x4e, 0xa9, 0x6c, 0x56, 0xf4, 0xea, 0x65, 0x7a,
    0xae, 0x08, 0xba, 0x78, 0x25, 0x2e, 0x1c, 0xa6, 0xb4, 0xc6, 0xe8, 0xdd, 0x74, 0x1f, 0x4b, 0xbd, 0x8b, 0x8a, 0x70,
    0x3e, 0xb5, 0x66, 0x48, 0x03, 0xf6, 0x0e, 0x61, 0x35, 0x57, 0xb9, 0x86, 0xc1, 0x1d, 0x9e, 0xe1, 0xf8, 0x98, 0x11,
    0x69, 0xd9, 0x8e, 0x94, 0x9b, 0x1e, 0x87, 0xe9, 0xce, 0x55, 0x28, 0xdf, 0x8c, 0xa1, 0x89, 0x0d, 0xbf, 0xe6, 0x42,
    0x68, 0x41, 0x99, 0x2d, 0x0f, 0xb0, 0x54, 0xbb, 0x16};

/* Shift values for short/long variants */
static const u8 Shift[2][2][ROWS] = {{{0, 1, 2, 3, 4, 5, 6, 7}, {1, 3, 5, 7, 0, 2, 4, 6}},
                                      {{0, 1, 2, 3, 4, 5, 6, 11}, {1, 3, 5, 11, 0, 2, 4, 6}}};

/* AddRoundConstant xors a round-dependent constant to the state */
static void AddRoundConstant(u8 x[ROWS][COLS1024], int columns, u8 round, Variant v) {
  int i, j;
  if ((v & 1) == 0) {
    for (i = 0; i < columns; i++)
      x[0][i] ^= (i << 4) ^ round;
  } else {
    for (i = 0; i < columns; i++)
      for (j = 0; j < ROWS - 1; j++)
        x[j][i] ^= 0xff;
    for (i = 0; i < columns; i++)
      x[ROWS - 1][i] ^= (i << 4) ^ 0xff ^ round;
  }
}

/* SubBytes replaces each byte by a value from the S-box */
static void SubBytes(u8 x[ROWS][COLS1024], int columns) {
  int i, j;

  for (i = 0; i < ROWS; i++)
    for (j = 0; j < columns; j++)
      x[i][j] = S[x[i][j]];
}

/* ShiftBytes cyclically shifts each row to the left by a number of
   positions */
static void ShiftBytes(u8 x[ROWS][COLS1024], int columns, Variant v) {
  const u8 *R = Shift[v / 2][v & 1];
  int        i, j;
  u8         temp[COLS1024];

  for (i = 0; i < ROWS; i++) {
    for (j = 0; j < columns; j++) {
      temp[j] = x[i][(j + R[i]) % columns];
    }
    for (j = 0; j < columns; j++) {
      x[i][j] = temp[j];
    }
  }
}

/* MixBytes reversibly mixes the bytes within a column */
static void MixBytes(u8 x[ROWS][COLS1024], int columns) {
  int i, j;
  u8  temp[ROWS];

  for (i = 0; i < columns; i++) {
    for (j = 0; j < ROWS; j++) {
      temp[j] = mul2(x[(j + 0) % ROWS][i]) ^ mul2(x[(j + 1) % ROWS][i]) ^ mul3(x[(j + 2) % ROWS][i]) ^
                mul4(x[(j + 3) % ROWS][i]) ^ mul5(x[(j + 4) % ROWS][i]) ^ mul3(x[(j + 5) % ROWS][i]) ^
                mul5(x[(j + 6) % ROWS][i]) ^ mul7(x[(j + 7) % ROWS][i]);
    }
    for (j = 0; j < ROWS; j++) {
      x[j][i] = temp[j];
    }
  }
}

/* apply P-permutation to x */
static void P(hashState *ctx, u8 x[ROWS][COLS1024]) {
  u8      i;
  Variant v = ctx->columns == 8 ? P512 : P1024;
  for (i = 0; i < ctx->rounds; i++) {
    AddRoundConstant(x, ctx->columns, i, v);
    SubBytes(x, ctx->columns);
    ShiftBytes(x, ctx->columns, v);
    MixBytes(x, ctx->columns);
  }
}

/* apply Q-permutation to x */
static void Q(hashState *ctx, u8 x[ROWS][COLS1024]) {
  u8      i;
  Variant v = ctx->columns == 8 ? Q512 : Q1024;
  for (i = 0; i < ctx->rounds; i++) {
    AddRoundConstant(x, ctx->columns, i, v);
    SubBytes(x, ctx->columns);
    ShiftBytes(x, ctx->columns, v);
    MixBytes(x, ctx->columns);
  }
}

/* digest (up to) msglen bytes */
static void Transform(hashState *ctx, const BitSequence *input, u32 msglen) {
  unsigned int i, j;
/*
u8 temp1[ROWS][COLS1024], temp2[ROWS][COLS1024];
*/
#define temp1 G_cx.groestl.temp1
#define temp2 G_cx.groestl.temp2

  /* digest one message block at the time */
  for (; msglen >= ctx->statesize; msglen -= ctx->statesize, input += ctx->statesize) {
    /* store message block (m) in temp2, and xor of chaining (h) and
       message block in temp1 */
    for (i = 0; i < ROWS; i++) {
      for (j = 0; j < ctx->columns; j++) {
        temp1[i][j] = ctx->chaining[i][j] ^ input[j * ROWS + i];
        temp2[i][j] = input[j * ROWS + i];
      }
    }

    P(ctx, temp1); /* P(h+m) */
    Q(ctx, temp2); /* Q(m) */

    /* xor P(h+m) and Q(m) onto chaining, yielding P(h+m)+Q(m)+h */
    for (i = 0; i < ROWS; i++) {
      cx_memxor(ctx->chaining[i], temp1[i], ctx->columns);
      cx_memxor(ctx->chaining[i], temp2[i], ctx->columns);
      // for (j = 0; j < ctx->columns; j++) {
      //   ctx->chaining[i][j] ^= temp1[i][j] ^ temp2[i][j];
      // }
    }

    /* increment block counter */
    ctx->block_counter++;
  }
#undef temp1
#undef temp2
}

/* do output transformation, P(h)+h */
static void OutputTransformation(hashState *ctx) {
  unsigned int i, j;
/*
u8 temp[ROWS][COLS1024];
*/
// OK because no used at the same time than tmp in Transform
#define temp G_cx.groestl.temp1

  /* store chaining ("h") in temp */
  for (i = 0; i < ROWS; i++) {
    for (j = 0; j < ctx->columns; j++) {
      temp[i][j] = ctx->chaining[i][j];
    }
  }

  /* compute P(temp) = P(h) */
  P(ctx, temp);

  /* feed chaining forward, yielding P(h)+h */
  for (i = 0; i < ROWS; i++) {
    cx_memxor(ctx->chaining[i], temp[i], ctx->columns);
    // for (j = 0; j < ctx->columns; j++) {
    //   ctx->chaining[i][j] ^= temp[i][j];
    // }
  }
#undef temp
}

/* initialise context */
static HashReturn Init(hashState *ctx, int hashbitlen) {
  unsigned int i, j;

  if (hashbitlen <= 0 || (hashbitlen % 8) || hashbitlen > 512)
    return BAD_HASHLEN;

  if (hashbitlen <= 256) {
    ctx->rounds    = ROUNDS512;
    ctx->columns   = COLS512;
    ctx->statesize = SIZE512;
  } else {
    ctx->rounds    = ROUNDS1024;
    ctx->columns   = COLS1024;
    ctx->statesize = SIZE1024;
  }

  /* zeroise chaining variable */
  for (i = 0; i < ROWS; i++) {
    for (j = 0; j < ctx->columns; j++) {
      ctx->chaining[i][j] = 0;
    }
  }

  /* store hashbitlen and set initial value */
  ctx->hashlen = hashbitlen / 8u;
  for (i = ROWS - sizeof(int); i < ROWS; i++) {
    ctx->chaining[i][ctx->columns - 1] = (u8)(hashbitlen >> (8 * (7 - i)));
  }

  /* initialise other variables */
  ctx->buf_ptr       = 0;
  ctx->block_counter = 0;

  return SUCCESS;
}

static void Update(hashState *ctx, const BitSequence *input, DataLength msglen) {
  unsigned int index = 0;

  /* if the buffer contains data that still needs to be digested */
  if (ctx->buf_ptr) {
    /* copy data into buffer until buffer is full, or there is no more
       data */
    for (index = 0; ctx->buf_ptr < ctx->statesize && index < msglen; index++, ctx->buf_ptr++) {
      ctx->buffer[ctx->buf_ptr] = input[index];
    }

    if (ctx->buf_ptr < ctx->statesize) {
      /* this chunk of message does not fill the buffer */
      return;
    }

    /* the buffer is full, digest */
    ctx->buf_ptr = 0;
    Transform(ctx, ctx->buffer, ctx->statesize);
  }

  /* digest remainder of data modulo the block size */
  Transform(ctx, input + index, msglen - index);
  index += ((msglen - index) / ctx->statesize) * ctx->statesize;

  /* copy remaining data to buffer */
  for (; index < msglen; index++, ctx->buf_ptr++) {
    ctx->buffer[ctx->buf_ptr] = input[index];
  }
  return;
}

static void Final(hashState *ctx, BitSequence *output) {
  unsigned int zeroise;
  unsigned int i, j;

  /* 100... padding */
  ctx->buffer[ctx->buf_ptr++] = 0x80;

  if (ctx->buf_ptr > ctx->statesize - LENGTHFIELDLEN) {
    /* padding requires two blocks */
    while (ctx->buf_ptr < ctx->statesize) {
      ctx->buffer[ctx->buf_ptr++] = 0;
    }
    Transform(ctx, ctx->buffer, ctx->statesize);
    ctx->buf_ptr = 0;
  }
  while (ctx->buf_ptr < ctx->statesize - LENGTHFIELDLEN) {
    ctx->buffer[ctx->buf_ptr++] = 0;
  }

  /* length padding */
  ctx->block_counter++;
  ctx->buf_ptr = ctx->statesize;
  while (ctx->buf_ptr > ctx->statesize - LENGTHFIELDLEN) {
    ctx->buffer[--ctx->buf_ptr] = (u8)ctx->block_counter;
    ctx->block_counter >>= 8;
  }

  /* digest (last) padding block */
  Transform(ctx, ctx->buffer, ctx->statesize);
  /* output transformation */
  OutputTransformation(ctx);

  /* store hash output */
  if (output) {
    zeroise = 1;
  } else {
    zeroise = 0;
    output  = ctx->buffer;
  }
  j = 0;
  for (i = ctx->statesize - ctx->hashlen; i < ctx->statesize; i++, j++) {
    output[j] = ctx->chaining[i % ROWS][i / ROWS];
  }
  if (zeroise == 0) {
    zeroise = j;
  }

  /* zeroise */
  for (i = 0; i < ROWS; i++) {
    for (j = 0; j < ctx->columns; j++) {
      ctx->chaining[i][j] = 0;
    }
  }

  for (i = zeroise; i < ctx->statesize; i++) {
    ctx->buffer[i] = 0;
  }
}

#endif // HAVE_GROESTL

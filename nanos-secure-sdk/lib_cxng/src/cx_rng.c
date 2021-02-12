
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

#ifdef HAVE_RNG

#include "cx_ram.h"
#include "cx_utils.h"
#include "cx_crc.h"
#include "cx_ades.h"
#include "lcx_rng.h"
#include "errors.h"
#include "exceptions.h"

#include "os_halt.h"
#include "os_math.h"
#include "os_random.h"
#include <string.h>

#if defined(HAVE_RNG_SONY)
uint32_t G_cx_rng_sony;
#endif // CX_RNG_SONY

/* ----------------------------------------------------------------------- */
/*                                                                         */
/* ----------------------------------------------------------------------- */
void cx_rng_no_throw(uint8_t *buffer, size_t len) {
  cx_err_t error;

  error = cx_get_random_bytes(buffer, len);
  if (error) {
    /* XXX: calling halt() add a dependency to THROW (os_longjmp). */
    while (1);
  }
}

uint32_t cx_rng_u32(void) {
  uint32_t r;
  cx_rng_no_throw((uint8_t *)&r, sizeof(uint32_t));
  return r;
}

uint8_t cx_rng_u8(void) {
  uint8_t r;
  cx_rng_no_throw((uint8_t *)&r, sizeof(uint8_t));
  return r;
}

/* ----------------------------------------------------------------------- */
/*                                                                         */
/* ----------------------------------------------------------------------- */
uint32_t cx_rng_u32_range_func(uint32_t a, uint32_t b, cx_rng_u32_range_randfunc_t randfunc) {
  uint32_t range = b - a;
  uint32_t r;

  if ((range & (range - 1)) == 0) {  // special case: range is a power of 2
    r = randfunc();
    return a + r % range;
  }

  uint32_t chunk_size = UINT32_MAX / range;
  uint32_t last_chunk_value = chunk_size * range;
  r = randfunc();
  while (r >= last_chunk_value) {
    r = randfunc();
  }
  return a + r / chunk_size;
}

/* ----------------------------------------------------------------------- */
/*                                                                         */
/* ----------------------------------------------------------------------- */
uint32_t cx_rng_u32_range(uint32_t a, uint32_t b) {
  return cx_rng_u32_range_func(a, b, (cx_rng_u32_range_randfunc_t)cx_rng_u32);
}

#endif // HAVE_RNG


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
#include "cx_rng.h"

#if defined(HAVE_RNG_SONY)
extern unsigned int G_cx_rng_sony;
#endif // CX_RNG_SONY

void cx_init() {
#ifdef HAVE_RNG
// init Sony RNG
#if defined(HAVE_RNG_SONY)
  G_cx_rng_sony = 0x42;
#endif // CX_RNG_SONY

#endif // HAVE_RNG
}

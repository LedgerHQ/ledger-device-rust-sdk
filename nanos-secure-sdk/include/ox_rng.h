
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

#ifndef OX_RNG_H
#define OX_RNG_H

#include <stdint.h>

#include "decorators.h"

SYSCALL void cx_trng_get_random_data(uint8_t *buf PLENGTH(size), size_t size);
SYSCALL void cx_trng_selftest(void);
SYSCALL void cx_trng_init(void);

#endif // OX_RNG_H
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

#if !defined(HAVE_BOLOS) && defined(HAVE_PENDING_REVIEW_SCREEN)

#include <stddef.h>

#include "checks.h"
#include "os_helpers.h"
#include "os_types.h"

// This label ultimately comes from the application link.
extern unsigned int const _install_parameters;
extern void sample_pending();

// This function is called at the end of the seph initialization.
// It checks the install parameters of the application to be run, and if this area contains the
// CHECK_NOT_AUDITED_TLV_TAG tag with the CHECK_NOT_AUDITED_TLV_VAL value, a specific display
// is triggered before the actual application's splash screen.
void check_audited_app(void) {
  unsigned char     data = BOLOS_FALSE;
  unsigned char*    buffer = &data;
  unsigned int      length = os_parse_bertlv((unsigned char*)(&_install_parameters),
                                             CHECK_NOT_AUDITED_MAX_LEN,
                                             NULL,
                                             CHECK_NOT_AUDITED_TLV_TAG,
                                             0x00,
                                             (void**)&buffer,
                                             sizeof(data));

  // We trigger the associated behaviour only when the tag was present and the value corresponds to
  // the expected one.
  if ((length)
      && (CHECK_NOT_AUDITED_TLV_VAL == data))
  {
    sample_pending();
  }
}

#endif // !defined(HAVE_BOLOS)  && defined(HAVE_PENDING_REVIEW_SCREEN)

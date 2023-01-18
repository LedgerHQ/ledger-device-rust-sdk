
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

#include "exceptions.h"
#include "lcx_rng.h"
#include "os_helpers.h"
#include "os_io.h"
#include "os_utils.h"
#include <string.h>

char os_secure_memcmp(const void *src1, const void* src2, size_t length) {
#define SRC1 ((unsigned char const *)src1)
#define SRC2 ((unsigned char const *)src2)
  unsigned int l = length;
  unsigned char xoracc=0;
  // don't || to ensure all condition are evaluated
  while(!(!length && !l)) {
    length--;
    xoracc |= SRC1[length] ^ SRC2[length];
    l--;
  }
  // WHAT ??? glitch detected ?
  if (*(volatile unsigned int*)&l!=*(volatile unsigned int*)&length) {
    THROW(EXCEPTION);
  }
  return xoracc;
}

// BER encoded
// <tag> <length> <value>
// tag: 1 byte only
// length: 1 byte if little than 0x80, else 1 byte of length encoding (0x8Y, with Y the number of following bytes the length is encoded on) and then Y bytes of BE encoded total length
// value: no encoding, raw data
unsigned int os_parse_bertlv(unsigned char* mem, unsigned int mem_len,
                             unsigned int * tlvoffset, unsigned int tag, unsigned int offset, void** buffer, unsigned int maxlength) {
  unsigned int ret, tlvoffset_in;
  unsigned int check_equals_buffer = offset & OS_PARSE_BERTLV_OFFSET_COMPARE_WITH_BUFFER;
  unsigned int get_address = offset & OS_PARSE_BERTLV_OFFSET_GET_LENGTH;
  offset &= ~(OS_PARSE_BERTLV_OFFSET_COMPARE_WITH_BUFFER|OS_PARSE_BERTLV_OFFSET_GET_LENGTH);

  // nothing to be read
  if (mem_len == 0 || buffer == NULL || (!get_address && *buffer == NULL)) {
    return 0;
  }

  // the tlv start address
  unsigned char* tlv = (unsigned char*) mem;
  unsigned int remlen = mem_len;
  ret = 0;

  // account for a shift in the tlv list before parsing
  tlvoffset_in = 0;
  if (tlvoffset) {
    tlvoffset_in = *tlvoffset;
  }

  // parse tlv until some tag to parse
  while(remlen>=2) {
    // tag matches
    unsigned int tlvtag = *tlv++;
    remlen--;
    unsigned int tlvlen = *tlv++;
    remlen--;
    if (remlen == 0) {
      goto retret;
    }
    if (tlvlen >= 0x80) {
      // invalid encoding
      if (tlvlen == 0x80) {
        goto retret;
      }
      unsigned int tlvlenlen_ = tlvlen & 0x7F;
      tlvlen = 0;
      while(tlvlenlen_--) {
        // BE encoded
        tlvlen = (tlvlen << 8) | ((*tlv++)&0xFF);
        remlen--;
        if (remlen == 0) {
          goto retret;
        }
      }
    }
    // check if tag matches
    if (tlvtag == (tag&0xFF)) {
      if (tlvoffset) {
        unsigned int o = (unsigned int) tlv - (unsigned int)mem;
        // compute the current position in the tlv bytes
        *tlvoffset = o;

        // skip the tag if the requested tlvoffset has not been matched yet.
        if (tlvoffset_in>o) {
          goto next_tlv;
        }
      }
      // avoid OOB
      if (offset > tlvlen || offset > remlen) {
        goto retret;
      }

      // check maxlength is respected for equality
      if (check_equals_buffer && (tlvlen-offset) != maxlength) {
        // buffer to check the complete given length
        goto retret;
      }

      maxlength = MIN(maxlength, MIN(tlvlen-offset, remlen));
      // robustness check to avoid memory dumping, only allowing data space dumps
      if (
        offset > mem_len
        || maxlength > mem_len
        || offset+maxlength > mem_len
        // don't rely only on provided app bounds to avoid address forgery
        || (unsigned int)tlv < (unsigned int)mem
        || (unsigned int)tlv+offset < (unsigned int)mem
        || (unsigned int)tlv+offset+maxlength < (unsigned int)mem
        || (unsigned int)tlv > (unsigned int)mem+mem_len
        || (unsigned int)tlv+offset > (unsigned int)mem+mem_len
        || (unsigned int)tlv+offset+maxlength > (unsigned int)mem+mem_len) {
        goto retret;
      }

      // retrieve the tlv's data content at the requested offset, and return the total data length
      if (get_address) {
        *buffer = tlv+offset;
        // return the tlv's total length from requested offset
        ret = MIN(tlvlen-offset, remlen);
        goto retret;
      }

      if (!check_equals_buffer) {
        memmove(*buffer, tlv+offset, maxlength);
      }
      else {
        ret = os_secure_memcmp(*buffer, tlv+offset, maxlength) == 0;
        goto retret;
      }
      ret = maxlength;
      goto retret;
    }
  next_tlv:
    // skip to next tlv
    tlv += tlvlen;
    remlen-=MIN(remlen, tlvlen);
  }
retret:
  return ret;
}

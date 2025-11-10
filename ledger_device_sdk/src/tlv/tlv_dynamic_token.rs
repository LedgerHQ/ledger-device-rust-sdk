//! Dynamic Token TLV Parser
//!
//! This module implements the following cross-application specification:
//! <https://ledgerhq.atlassian.net/wiki/spaces/TA/pages/5603262535/Token+Dynamic+Descriptor>
//!
//! Please refer to [TLV Generic](crate::tlv::tlv_generic) for documentation on how to write your own use-case if it
//! does not follow the above specification.
//!
//! The goal of this TLV use case is to parse dynamic information about a token for clear signing
//! purposes.
//!
//! The trusted information comes from the Ledger CAL and is forwarded by Ledger Wallet.
//! TLV data are signed by Ledger PKI infrastructure and the signature is verified using
//! the [PKI module](crate::pki).
//!
//! A PKI certificate with the appropriate usage must have been received and installed beforehand.
//! A sample application implementing this use-case is provide as part of the SDK
//! in the `examples` folder along with sample PKI certificate and TLV payload APDUs.

use super::tlv_generic::*;
use crate::ecc::CurvesId;
use crate::hash::sha2::Sha2_256;
use crate::hash::HashInit;
use crate::pki::pki_check_signature;
use crate::tag_to_flag_u64;
use ledger_secure_sdk_sys::CERTIFICATE_PUBLIC_KEY_USAGE_COIN_META;
extern crate alloc;
use alloc::string::String;
use alloc::{vec, vec::Vec};

/// Dynamic Token TLV Tags
const TAG_STRUCTURE_TYPE: Tag = 0x01;
const TAG_VERSION: Tag = 0x02;
const TAG_COIN_TYPE: Tag = 0x03;
const TAG_APP: Tag = 0x04;
const TAG_TICKER: Tag = 0x05;
const TAG_MAGNITUDE: Tag = 0x06;
const TAG_TUID: Tag = 0x07;
const TAG_SIGNATURE: Tag = 0x08;

// Generate the tag_to_flag_u64 function using the macro
tag_to_flag_u64!(
    TAG_STRUCTURE_TYPE,
    TAG_VERSION,
    TAG_COIN_TYPE,
    TAG_APP,
    TAG_TICKER,
    TAG_MAGNITUDE,
    TAG_TUID,
    TAG_SIGNATURE
);

/// Dynamic Token Output type
#[derive(Default, Debug)]
pub struct DynamicTokenOut {
    /// Version of the dynamic token structure
    pub version: u8,
    /// Coin type
    pub coin_type: Vec<u8>,
    /// Application name
    pub app_name: String,
    /// Ticker symbol
    pub ticker: String,
    /// Magnitude
    pub magnitude: u8,
    /// Token unique identifier
    pub tuid: Vec<u8>,
}

#[derive(Default)]
struct DynamicTokenExtracted {
    structure_type: u8,
    dynamic_token_out: DynamicTokenOut,
    signature: Vec<u8>,
    hash_ctx: Sha2_256,
}

// Handlers
fn on_structure_type(d: &TlvData<'_>, out: &mut DynamicTokenExtracted) -> Result<bool> {
    let v: u64 = d.as_u64_be()?;
    out.structure_type = v as u8;
    Ok(true)
}
fn on_version(d: &TlvData<'_>, out: &mut DynamicTokenExtracted) -> Result<bool> {
    let v: u64 = d.as_u64_be()?;
    out.dynamic_token_out.version = v as u8;
    Ok(true)
}
fn on_coin_type(d: &TlvData<'_>, out: &mut DynamicTokenExtracted) -> Result<bool> {
    out.dynamic_token_out.coin_type = d.as_bytes().to_vec();
    Ok(true)
}
fn on_app_name(d: &TlvData<'_>, out: &mut DynamicTokenExtracted) -> Result<bool> {
    out.dynamic_token_out.app_name =
        String::from(core::str::from_utf8(d.as_bytes()).map_err(|_| TlvError::LengthOverflow)?);
    Ok(true)
}
fn on_ticker(d: &TlvData<'_>, out: &mut DynamicTokenExtracted) -> Result<bool> {
    out.dynamic_token_out.ticker =
        String::from(core::str::from_utf8(d.as_bytes()).map_err(|_| TlvError::LengthOverflow)?);
    Ok(true)
}
fn on_magnitude(d: &TlvData<'_>, out: &mut DynamicTokenExtracted) -> Result<bool> {
    let v: u64 = d.as_u64_be()?;
    out.dynamic_token_out.magnitude = v as u8;
    Ok(true)
}
fn on_tuid(d: &TlvData<'_>, out: &mut DynamicTokenExtracted) -> Result<bool> {
    out.dynamic_token_out.tuid = d.as_bytes().to_vec();
    Ok(true)
}
fn on_signature(d: &TlvData<'_>, out: &mut DynamicTokenExtracted) -> Result<bool> {
    out.signature = d.as_bytes().to_vec();
    Ok(true)
}
fn on_common(d: &TlvData<'_>, out: &mut DynamicTokenExtracted) -> Result<bool> {
    if d.tag != TAG_SIGNATURE {
        let result = out.hash_ctx.update(d.raw);
        if result.is_err() {
            return Err(TlvError::HandlerFailed);
        }
    }
    Ok(true)
}

// Static handler table
static HANDLERS: &[Handler<DynamicTokenExtracted>] = &[
    Handler {
        tag: TAG_STRUCTURE_TYPE,
        unique: true,
        func: Some(on_structure_type),
    },
    Handler {
        tag: TAG_VERSION,
        unique: true,
        func: Some(on_version),
    },
    Handler {
        tag: TAG_COIN_TYPE,
        unique: true,
        func: Some(on_coin_type),
    },
    Handler {
        tag: TAG_APP,
        unique: true,
        func: Some(on_app_name),
    },
    Handler {
        tag: TAG_TICKER,
        unique: true,
        func: Some(on_ticker),
    },
    Handler {
        tag: TAG_MAGNITUDE,
        unique: true,
        func: Some(on_magnitude),
    },
    Handler {
        tag: TAG_TUID,
        unique: true,
        func: Some(on_tuid),
    },
    Handler {
        tag: TAG_SIGNATURE,
        unique: true,
        func: Some(on_signature),
    },
];

/// Parse Dynamic Token TLV-encoded data
/// # Arguments
/// * `payload` - The TLV-encoded data to parse.
/// * `out` - The output structure to fill with parsed data.
/// # Returns
/// Returns `Ok(())` if parsing was successful, or a `TlvError` otherwise.
pub fn parse_dynamic_token_tlv(payload: &[u8], out: &mut DynamicTokenOut) -> Result<()> {
    let mut extracted = DynamicTokenExtracted::default();
    extracted.hash_ctx = Sha2_256::new();

    let mut received = Received::new(tag_to_flag_u64);

    let mut cfg = ParseCfg::new(HANDLERS);
    cfg.common = Some(on_common);

    parse(&cfg, payload, &mut extracted, &mut received)?;

    // Check that mandatory TAGs were received
    let mandatory_tags = tag_to_flag_u64(TAG_STRUCTURE_TYPE)
        | tag_to_flag_u64(TAG_VERSION)
        | tag_to_flag_u64(TAG_COIN_TYPE)
        | tag_to_flag_u64(TAG_APP)
        | tag_to_flag_u64(TAG_TICKER)
        | tag_to_flag_u64(TAG_MAGNITUDE)
        | tag_to_flag_u64(TAG_TUID)
        | tag_to_flag_u64(TAG_SIGNATURE);
    if received.flags & mandatory_tags != mandatory_tags {
        return Err(TlvError::MissingMandatoryTag);
    }

    // At this point, all TLV fields have been processed and the signature needs to be verified
    // Step 1: finalize the hash
    let hash_size = extracted.hash_ctx.get_size();
    let mut hash = vec![0u8; hash_size];
    let res = extracted.hash_ctx.finalize(&mut hash);
    if res.is_err() {
        return Err(TlvError::SignatureVerificationFailed);
    }

    // Step 2: verify the signature of TLV payload
    // with PKI certificate.
    // In test mode, skip signature verification
    #[cfg(not(test))]
    {
        let res = pki_check_signature(
            &mut hash,
            CERTIFICATE_PUBLIC_KEY_USAGE_COIN_META,
            CurvesId::Secp256k1,
            &mut extracted.signature,
        );
        if res.is_err() {
            return Err(TlvError::SignatureVerificationFailed);
        }
    }

    // Copy the extracted dynamic token output
    *out = extracted.dynamic_token_out;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::assert_eq_err as assert_eq;
    use crate::testing::TestType;
    use crate::tlv::{parse_dynamic_token_tlv, DynamicTokenOut};
    use testmacro::test_item as test;

    const TLV_PAYLOAD: &[u8] = &[
        0x01, 0x01, 0x90, 0x02, 0x01, 0x01, 0x03, 0x02, 0x01, 0xf5, 0x04, 0x06, 0x53, 0x6f, 0x6c,
        0x61, 0x6e, 0x61, 0x05, 0x04, 0x47, 0x4f, 0x52, 0x4b, 0x06, 0x01, 0x06, 0x07, 0x33, 0x10,
        0x01, 0x00, 0x11, 0x2c, 0x33, 0x38, 0x50, 0x67, 0x7a, 0x70, 0x4a, 0x59, 0x75, 0x32, 0x48,
        0x6b, 0x69, 0x59, 0x76, 0x56, 0x38, 0x71, 0x65, 0x50, 0x46, 0x61, 0x6b, 0x42, 0x38, 0x74,
        0x75, 0x6f, 0x62, 0x50, 0x64, 0x47, 0x6d, 0x32, 0x46, 0x46, 0x45, 0x6e, 0x37, 0x44, 0x70,
        0x75, 0x6d, 0x70, 0x12, 0x00, 0x08, 0x48, 0x30, 0x46, 0x02, 0x21, 0x00, 0x91, 0xd2, 0x54,
        0xb2, 0xf9, 0xe8, 0xaa, 0xd9, 0x27, 0x54, 0xfc, 0x62, 0x75, 0xb4, 0xd2, 0x18, 0x22, 0x7c,
        0xe8, 0x93, 0x70, 0xcb, 0x72, 0x43, 0x48, 0xd5, 0xc4, 0x42, 0x6a, 0x9e, 0x95, 0xd6, 0x02,
        0x21, 0x00, 0xd1, 0x18, 0x6c, 0x28, 0x81, 0x1f, 0x33, 0x48, 0x48, 0x86, 0x7e, 0xfa, 0x10,
        0xb0, 0x61, 0x1a, 0xd2, 0xdf, 0xdf, 0x6b, 0x50, 0xfc, 0xec, 0xdd, 0x55, 0xc1, 0x1d, 0x57,
        0x22, 0x41, 0xb4, 0x37,
    ];

    #[test]
    fn test_parse_dynamic_token_tlv() {
        let mut out = DynamicTokenOut::default();
        let res = parse_dynamic_token_tlv(TLV_PAYLOAD, &mut out);
        assert_eq!(res, Ok(()));
    }

    #[test]
    fn test_parse_dynamic_token_tlv_missing_tag() {
        let mut out = DynamicTokenOut::default();
        let res = parse_dynamic_token_tlv(&TLV_PAYLOAD[3..], &mut out);
        assert_eq!(res, Err(crate::tlv::TlvError::MissingMandatoryTag));
    }

    #[test]
    fn test_parse_dynamic_token_tlv_unexpected_eof() {
        let mut out = DynamicTokenOut::default();
        let res = parse_dynamic_token_tlv(&TLV_PAYLOAD[..12], &mut out);
        assert_eq!(res, Err(crate::tlv::TlvError::UnexpectedEof));
    }

    #[test]
    fn test_parse_dynamic_token_tlv_invalid_der_length() {
        let mut out = DynamicTokenOut::default();
        let mut invalid_payload = TLV_PAYLOAD.to_vec();
        invalid_payload[1] = 0xFF; // Invalid length for tag 0x01
        let res = parse_dynamic_token_tlv(&invalid_payload, &mut out);
        assert_eq!(res, Err(crate::tlv::TlvError::InvalidDerLength));
    }

    #[test]
    fn test_parse_dynamic_token_tlv_length_overflow() {
        let mut out = DynamicTokenOut::default();
        let mut invalid_payload = TLV_PAYLOAD.to_vec();
        invalid_payload[1] = 0x0A; // Invalid length for tag 0x01
        let res = parse_dynamic_token_tlv(&invalid_payload, &mut out);
        assert_eq!(res, Err(crate::tlv::TlvError::LengthOverflow));
    }

    #[test]
    fn test_parse_dynamic_token_tlv_unknown_tag() {
        let mut out = DynamicTokenOut::default();
        let mut invalid_payload = TLV_PAYLOAD.to_vec();
        invalid_payload[0] = 0x09; // Unknown tag
        let res = parse_dynamic_token_tlv(&invalid_payload, &mut out);
        assert_eq!(res, Err(crate::tlv::TlvError::UnknownTag));
    }

    #[test]
    fn test_parse_dynamic_token_tlv_dupplicate_unique_tag() {
        let mut out = DynamicTokenOut::default();
        let mut invalid_payload = TLV_PAYLOAD.to_vec();
        // Duplicate tag 0x01
        invalid_payload.extend_from_slice(&[0x01, 0x01, 0x90]);
        let res = parse_dynamic_token_tlv(&invalid_payload, &mut out);
        assert_eq!(res, Err(crate::tlv::TlvError::DuplicateUniqueTag));
    }
}

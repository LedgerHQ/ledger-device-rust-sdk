//! Trusted Name TLV Parser
//!
//! This module implements the following cross-application specification:
//! <https://ledgerhq.atlassian.net/wiki/spaces/TrustServices/pages/3736863735/LNS+Arch+Nano+Trusted+Names+Descriptor+Format+APIs>
//!
//! Please refer to [TLV Generic](crate::tlv::tlv_generic) for documentation on how to write your own use-case if it
//! does not follow the above specification.
//!
//! The goal of this TLV use case is to associate a blockchain address to a trusted domain name.
//!
//! The trusted information comes from the Ledger CAL and is forwarded by Ledger Wallet.
//! TLV data are signed by Ledger PKI infrastructure and the signature is verified using
//! the [PKI module](crate::pki).
//!
//! A PKI certificate with the appropriate usage must have been received and installed beforehand.
//! A sample application implementing this use-case is provided as part of the SDK
//! in the `examples` folder along with sample PKI certificate and TLV payload APDUs.

use super::tlv_generic::*;
use crate::ecc::CurvesId;
use crate::hash::ripemd::Ripemd160;
use crate::hash::sha2::{Sha2_256, Sha2_512};
use crate::hash::sha3::{Keccak256, Sha3_256};
use crate::hash::HashInit;
use crate::pki::pki_check_signature;
use crate::tag_to_flag_u64;
use ledger_secure_sdk_sys::CERTIFICATE_PUBLIC_KEY_USAGE_TRUSTED_NAME;
extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;

enum TlvTrustedNameSignerAlgorithm {
    TlvTrustedNameSignerAlgorithmEcdsaSha256 = 0x01,
    TlvTrustedNameSignerAlgorithmEcdsaSha3_256 = 0x02,
    TlvTrustedNameSignerAlgorithmEcdsaKeccak256 = 0x03,
    TlvTrustedNameSignerAlgorithmEcdsaRipemd160 = 0x04,
    TlvTrustedNameSignerAlgorithmEcdsaSha512 = 0x16,
    TlvTrustedNameSignerAlgorithmEddsaKeccak256 = 0x17,
    TlvTrustedNameSignerAlgorithmEddsaSha3_256 = 0x18,
}

/// Trusted Name TLV Tags
const TAG_STRUCTURE_TYPE: Tag = 0x01;
const TAG_VERSION: Tag = 0x02;
const TAG_TRUSTED_NAME_TYPE: Tag = 0x70;
const TAG_TRUSTED_NAME_SOURCE: Tag = 0x71;
const TAG_TRUSTED_NAME: Tag = 0x20;
const TAG_CHAIN_ID: Tag = 0x23;
const TAG_ADDRESS: Tag = 0x22;
const TAG_NFT_ID: Tag = 0x72;
const TAG_SOURCE_CONTRACT: Tag = 0x73;
const TAG_CHALLENGE: Tag = 0x12;
const TAG_NOT_VALID_AFTER: Tag = 0x10;
const TAG_SIGNER_KEY_ID: Tag = 0x13;
const TAG_SIGNER_ALGO: Tag = 0x14;
const TAG_DER_SIGNATURE: Tag = 0x15;

// Generate the tag_to_flag_u64 function using the macro
tag_to_flag_u64!(
    TAG_STRUCTURE_TYPE,
    TAG_VERSION,
    TAG_TRUSTED_NAME_TYPE,
    TAG_TRUSTED_NAME_SOURCE,
    TAG_TRUSTED_NAME,
    TAG_CHAIN_ID,
    TAG_ADDRESS,
    TAG_NFT_ID,
    TAG_SOURCE_CONTRACT,
    TAG_CHALLENGE,
    TAG_NOT_VALID_AFTER,
    TAG_SIGNER_KEY_ID,
    TAG_SIGNER_ALGO,
    TAG_DER_SIGNATURE
);

// Hash contexts for multiple algorithms
// Used to compute hashes in parallel while parsing TLV data
// for signature verification.
// We will know the correct format to use at reception of the signer_algo tag.
// We don't try to be clever and just calculate all hash until then.
// Performance hit is unnoticeable. Memory footprint is negligeable.
#[derive(Default)]
struct MultipleHashContext {
    hash_sha2_256: Sha2_256,
    hash_sha2_512: Sha2_512,
    hash_sha3_256: Sha3_256,
    hash_keccak_256: Keccak256,
    hash_ripemd_160: Ripemd160,
}

/// Trusted Name Output type
#[derive(Default, Debug)]
pub struct TrustedNameOut {
    /// Version of the Trusted Name structure
    pub version: u8,
    /// Type of the Trusted Name
    pub trusted_name_type: u8,
    /// Source of the Trusted Name
    pub trusted_name_source: u8,
    /// The Trusted Name itself
    pub trusted_name: String,
    /// Chain ID associated with the Trusted Name
    pub chain_id: u64,
    /// Address associated with the Trusted Name
    pub address: String,
    /// NFT ID associated with the Trusted Name (optional)
    pub nft_id: Option<Vec<u8>>,
    /// Source contract associated with the Trusted Name (optional)
    pub source_contract: Option<String>,
    /// Challenge associated with the Trusted Name (optional)
    pub challenge: Option<u32>,
    /// Not valid after timestamp associated with the Trusted Name (optional)
    pub not_valid_after: Option<u64>,
}

#[derive(Default)]
struct TrustedNameExtracted {
    structure_type: u8,
    trusted_name_out: TrustedNameOut,
    signer_key_id: u8,
    signer_algorithm: u8,
    signature: Vec<u8>,
    hash_ctx: MultipleHashContext,
}

// Handlers
fn on_structure_type(d: &TlvData<'_>, out: &mut TrustedNameExtracted) -> Result<bool> {
    let v: u64 = d.as_u64_be()?;
    out.structure_type = v as u8;
    Ok(true)
}
fn on_version(d: &TlvData<'_>, out: &mut TrustedNameExtracted) -> Result<bool> {
    let v: u64 = d.as_u64_be()?;
    out.trusted_name_out.version = v as u8;
    Ok(true)
}

fn on_trusted_name_type(d: &TlvData<'_>, out: &mut TrustedNameExtracted) -> Result<bool> {
    let v: u64 = d.as_u64_be()?;
    out.trusted_name_out.trusted_name_type = v as u8;
    Ok(true)
}

fn on_trusted_name_source(d: &TlvData<'_>, out: &mut TrustedNameExtracted) -> Result<bool> {
    let v: u64 = d.as_u64_be()?;
    out.trusted_name_out.trusted_name_source = v as u8;
    Ok(true)
}

fn on_trusted_name(d: &TlvData<'_>, out: &mut TrustedNameExtracted) -> Result<bool> {
    out.trusted_name_out.trusted_name =
        String::from(core::str::from_utf8(d.as_bytes()).map_err(|_| TlvError::LengthOverflow)?);
    Ok(true)
}

fn on_chain_id(d: &TlvData<'_>, out: &mut TrustedNameExtracted) -> Result<bool> {
    out.trusted_name_out.chain_id = d.as_u64_be()?;
    Ok(true)
}

fn on_address(d: &TlvData<'_>, out: &mut TrustedNameExtracted) -> Result<bool> {
    out.trusted_name_out.address =
        String::from(core::str::from_utf8(d.as_bytes()).map_err(|_| TlvError::LengthOverflow)?);
    Ok(true)
}

fn on_nft_id(d: &TlvData<'_>, out: &mut TrustedNameExtracted) -> Result<bool> {
    out.trusted_name_out.nft_id = Some(d.as_bytes().to_vec());
    Ok(true)
}

fn on_source_contract(d: &TlvData<'_>, out: &mut TrustedNameExtracted) -> Result<bool> {
    out.trusted_name_out.source_contract = Some(String::from(
        core::str::from_utf8(d.as_bytes()).map_err(|_| TlvError::LengthOverflow)?,
    ));
    Ok(true)
}

fn on_challenge(d: &TlvData<'_>, out: &mut TrustedNameExtracted) -> Result<bool> {
    let v: u64 = d.as_u64_be()?;
    out.trusted_name_out.challenge = Some(v as u32);
    Ok(true)
}

fn on_not_valid_after(d: &TlvData<'_>, out: &mut TrustedNameExtracted) -> Result<bool> {
    let v: u64 = d.as_u64_be()?;
    out.trusted_name_out.not_valid_after = Some(v);
    Ok(true)
}

fn on_signer_key_id(d: &TlvData<'_>, out: &mut TrustedNameExtracted) -> Result<bool> {
    let v: u64 = d.as_u64_be()?;
    out.signer_key_id = v as u8;
    Ok(true)
}

fn on_signer_algorithm(d: &TlvData<'_>, out: &mut TrustedNameExtracted) -> Result<bool> {
    let v: u64 = d.as_u64_be()?;
    out.signer_algorithm = v as u8;
    Ok(true)
}

fn on_signature(d: &TlvData<'_>, out: &mut TrustedNameExtracted) -> Result<bool> {
    out.signature = d.as_bytes().to_vec();
    Ok(true)
}

fn on_common(d: &TlvData<'_>, out: &mut TrustedNameExtracted) -> Result<bool> {
    if d.tag != TAG_DER_SIGNATURE {
        let hash_updates = [
            out.hash_ctx.hash_sha2_256.update(d.raw),
            out.hash_ctx.hash_sha2_512.update(d.raw),
            out.hash_ctx.hash_sha3_256.update(d.raw),
            out.hash_ctx.hash_keccak_256.update(d.raw),
            out.hash_ctx.hash_ripemd_160.update(d.raw),
        ];

        for result in hash_updates {
            if result.is_err() {
                return Err(TlvError::HandlerFailed);
            }
        }
    }

    Ok(true)
}

// Static handler table
static HANDLERS: &[Handler<TrustedNameExtracted>] = &[
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
        tag: TAG_TRUSTED_NAME_TYPE,
        unique: true,
        func: Some(on_trusted_name_type),
    },
    Handler {
        tag: TAG_TRUSTED_NAME_SOURCE,
        unique: true,
        func: Some(on_trusted_name_source),
    },
    Handler {
        tag: TAG_TRUSTED_NAME,
        unique: true,
        func: Some(on_trusted_name),
    },
    Handler {
        tag: TAG_CHAIN_ID,
        unique: true,
        func: Some(on_chain_id),
    },
    Handler {
        tag: TAG_ADDRESS,
        unique: true,
        func: Some(on_address),
    },
    Handler {
        tag: TAG_NFT_ID,
        unique: true,
        func: Some(on_nft_id),
    },
    Handler {
        tag: TAG_SOURCE_CONTRACT,
        unique: true,
        func: Some(on_source_contract),
    },
    Handler {
        tag: TAG_CHALLENGE,
        unique: true,
        func: Some(on_challenge),
    },
    Handler {
        tag: TAG_NOT_VALID_AFTER,
        unique: true,
        func: Some(on_not_valid_after),
    },
    Handler {
        tag: TAG_SIGNER_KEY_ID,
        unique: true,
        func: Some(on_signer_key_id),
    },
    Handler {
        tag: TAG_SIGNER_ALGO,
        unique: true,
        func: Some(on_signer_algorithm),
    },
    Handler {
        tag: TAG_DER_SIGNATURE,
        unique: true,
        func: Some(on_signature),
    },
];

/// Parse Trusted Name TLV-encoded data
/// # Arguments
/// * `payload` - The TLV-encoded input data
/// * `out` - The output TrustedNameOut structure to be filled
/// # Returns
/// * `Result<()>` - Ok(()) if parsing and verification succeed, Err(TlvError) otherwise
pub fn parse_trusted_name_tlv(payload: &[u8], out: &mut TrustedNameOut) -> Result<()> {
    let mut extracted = TrustedNameExtracted::default();

    extracted.hash_ctx = MultipleHashContext {
        hash_sha2_256: Sha2_256::new(),
        hash_sha2_512: Sha2_512::new(),
        hash_sha3_256: Sha3_256::new(),
        hash_keccak_256: Keccak256::new(),
        hash_ripemd_160: Ripemd160::new(),
    };

    let mut received = Received::new(tag_to_flag_u64);

    let mut cfg = ParseCfg::new(HANDLERS);
    cfg.common = Some(on_common);

    parse(&cfg, payload, &mut extracted, &mut received)?;

    // Check that mandatory TAGs were received
    let mandatory_tags = tag_to_flag_u64(TAG_STRUCTURE_TYPE)
        | tag_to_flag_u64(TAG_VERSION)
        | tag_to_flag_u64(TAG_TRUSTED_NAME_TYPE)
        | tag_to_flag_u64(TAG_TRUSTED_NAME_SOURCE)
        | tag_to_flag_u64(TAG_TRUSTED_NAME)
        | tag_to_flag_u64(TAG_CHAIN_ID)
        | tag_to_flag_u64(TAG_ADDRESS)
        | tag_to_flag_u64(TAG_SIGNER_KEY_ID)
        | tag_to_flag_u64(TAG_SIGNER_ALGO)
        | tag_to_flag_u64(TAG_DER_SIGNATURE);
    if received.flags & mandatory_tags != mandatory_tags {
        return Err(TlvError::MissingMandatoryTag);
    }

    // At this point, all TLV fields have been processed and the signature needs to be verified
    // Step 1: finalize the hash according to the signer_algorithm
    let mut hash = [0u8; 64];
    let mut hash_size = 0usize;
    let mut curve: CurvesId = CurvesId::Invalid;
    finalize_hashes(
        &mut extracted.hash_ctx,
        extracted.signer_algorithm,
        &mut hash,
        &mut hash_size,
        &mut curve,
    )?;

    // Step 2: verify the signature
    // Check signature with PKI certificate
    // In test mode, skip signature verification
    #[cfg(not(test))]
    {
        let res = pki_check_signature(
            &mut hash[..hash_size],
            CERTIFICATE_PUBLIC_KEY_USAGE_TRUSTED_NAME,
            curve,
            &mut extracted.signature,
        );
        if res.is_err() {
            return Err(TlvError::SignatureVerificationFailed);
        }
    }

    // Copy the extracted trusted name output
    *out = extracted.trusted_name_out;

    Ok(())
}

// Helper macro to reduce boilerplate for hash finalization
macro_rules! finalize_hash {
    ($hash_ctx:expr, $curve_id:expr, $hash:expr, $hash_size:expr, $curve:expr) => {{
        *$hash_size = $hash_ctx.get_size();
        *$curve = $curve_id;
        $hash_ctx
            .finalize($hash)
            .map_err(|_| TlvError::SignatureVerificationFailed)?;
    }};
}

fn finalize_hashes(
    hash_ctx: &mut MultipleHashContext,
    signer_algorithm: u8,
    hash: &mut [u8],
    hash_size: &mut usize,
    curve: &mut CurvesId,
) -> Result<()> {
    match signer_algorithm {
        x if x == TlvTrustedNameSignerAlgorithm::TlvTrustedNameSignerAlgorithmEcdsaSha256 as u8 => {
            finalize_hash!(
                hash_ctx.hash_sha2_256,
                CurvesId::Secp256k1,
                hash,
                hash_size,
                curve
            );
        }
        x if x
            == TlvTrustedNameSignerAlgorithm::TlvTrustedNameSignerAlgorithmEcdsaSha3_256 as u8 =>
        {
            finalize_hash!(
                hash_ctx.hash_sha3_256,
                CurvesId::Secp256k1,
                hash,
                hash_size,
                curve
            );
        }
        x if x
            == TlvTrustedNameSignerAlgorithm::TlvTrustedNameSignerAlgorithmEcdsaKeccak256 as u8 =>
        {
            finalize_hash!(
                hash_ctx.hash_keccak_256,
                CurvesId::Secp256k1,
                hash,
                hash_size,
                curve
            );
        }
        x if x
            == TlvTrustedNameSignerAlgorithm::TlvTrustedNameSignerAlgorithmEcdsaRipemd160 as u8 =>
        {
            finalize_hash!(
                hash_ctx.hash_ripemd_160,
                CurvesId::Secp256k1,
                hash,
                hash_size,
                curve
            );
        }
        x if x == TlvTrustedNameSignerAlgorithm::TlvTrustedNameSignerAlgorithmEcdsaSha512 as u8 => {
            finalize_hash!(
                hash_ctx.hash_sha2_512,
                CurvesId::Secp256k1,
                hash,
                hash_size,
                curve
            );
        }
        x if x
            == TlvTrustedNameSignerAlgorithm::TlvTrustedNameSignerAlgorithmEddsaKeccak256 as u8 =>
        {
            finalize_hash!(
                hash_ctx.hash_keccak_256,
                CurvesId::Ed25519,
                hash,
                hash_size,
                curve
            );
        }
        x if x
            == TlvTrustedNameSignerAlgorithm::TlvTrustedNameSignerAlgorithmEddsaSha3_256 as u8 =>
        {
            finalize_hash!(
                hash_ctx.hash_sha3_256,
                CurvesId::Ed25519,
                hash,
                hash_size,
                curve
            );
        }
        _ => return Err(TlvError::SignatureVerificationFailed),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::assert_eq_err as assert_eq;
    use crate::testing::TestType;
    use crate::tlv::{parse_trusted_name_tlv, TrustedNameOut};
    use testmacro::test_item as test;

    const TLV_PAYLOAD: &[u8] = &[
        0x01, 0x01, 0x03, 0x02, 0x01, 0x02, 0x70, 0x01, 0x06, 0x71, 0x01, 0x06, 0x20, 0x2c, 0x46,
        0x7a, 0x39, 0x6e, 0x70, 0x59, 0x4a, 0x47, 0x58, 0x6b, 0x38, 0x48, 0x75, 0x53, 0x4b, 0x77,
        0x64, 0x33, 0x52, 0x32, 0x48, 0x42, 0x57, 0x64, 0x64, 0x4d, 0x39, 0x4b, 0x7a, 0x37, 0x7a,
        0x63, 0x79, 0x46, 0x4c, 0x4c, 0x33, 0x67, 0x31, 0x32, 0x75, 0x50, 0x65, 0x65, 0x23, 0x01,
        0x65, 0x22, 0x2c, 0x41, 0x78, 0x6d, 0x55, 0x46, 0x33, 0x71, 0x6b, 0x64, 0x7a, 0x31, 0x7a,
        0x73, 0x31, 0x35, 0x31, 0x51, 0x35, 0x57, 0x74, 0x74, 0x56, 0x4d, 0x6b, 0x46, 0x70, 0x46,
        0x47, 0x51, 0x50, 0x77, 0x67, 0x68, 0x5a, 0x73, 0x34, 0x64, 0x31, 0x6d, 0x77, 0x59, 0x35,
        0x35, 0x64, 0x73, 0x2b, 0x4a, 0x55, 0x50, 0x79, 0x69, 0x77, 0x72, 0x59, 0x4a, 0x46, 0x73,
        0x6b, 0x55, 0x50, 0x69, 0x48, 0x61, 0x37, 0x68, 0x6b, 0x65, 0x52, 0x38, 0x56, 0x55, 0x74,
        0x41, 0x65, 0x46, 0x6f, 0x53, 0x59, 0x62, 0x4b, 0x65, 0x64, 0x5a, 0x4e, 0x73, 0x44, 0x76,
        0x43, 0x4e, 0x12, 0x04, 0xae, 0x96, 0x5c, 0x07, 0x13, 0x01, 0x00, 0x14, 0x01, 0x01, 0x15,
        0x47, 0x30, 0x45, 0x02, 0x21, 0x00, 0xf8, 0x19, 0xe2, 0xc2, 0xe6, 0x1b, 0x72, 0xa9, 0x7c,
        0xa5, 0x1a, 0x1e, 0x44, 0x0a, 0xdd, 0x22, 0xa9, 0x82, 0x35, 0x2b, 0x25, 0x60, 0x06, 0x1b,
        0x71, 0x88, 0xf0, 0x86, 0xd6, 0x21, 0x04, 0xf9, 0x02, 0x20, 0x62, 0x18, 0x3a, 0x32, 0x49,
        0x4a, 0xae, 0x8b, 0x41, 0x27, 0xfa, 0xf2, 0x1b, 0x75, 0xce, 0xc3, 0x8d, 0x49, 0xb4, 0x8d,
        0x07, 0x69, 0xa6, 0x42, 0x56, 0x66, 0xe7, 0xee, 0x14, 0x3e, 0xc9, 0xc2,
    ];

    #[test]
    fn test_parse_trusted_name_tlv() {
        let mut out = TrustedNameOut::default();
        let res = parse_trusted_name_tlv(TLV_PAYLOAD, &mut out);
        assert_eq!(res, Ok(()));
    }

    #[test]
    fn test_parse_trusted_name_tlv_missing_tag() {
        let mut out = TrustedNameOut::default();
        let res = parse_trusted_name_tlv(&TLV_PAYLOAD[3..], &mut out);
        assert_eq!(res, Err(crate::tlv::TlvError::MissingMandatoryTag));
    }

    #[test]
    fn test_parse_trusted_name_tlv_unexpected_eof() {
        let mut out = TrustedNameOut::default();
        let res = parse_trusted_name_tlv(&TLV_PAYLOAD[..15], &mut out);
        assert_eq!(res, Err(crate::tlv::TlvError::UnexpectedEof));
    }

    #[test]
    fn test_parse_trusted_name_tlv_invalid_der_length() {
        let mut out = TrustedNameOut::default();
        let mut invalid_payload = TLV_PAYLOAD.to_vec();
        invalid_payload[1] = 0xFF; // Invalid length for tag 0x01
        let res = parse_trusted_name_tlv(&invalid_payload, &mut out);
        assert_eq!(res, Err(crate::tlv::TlvError::InvalidDerLength));
    }

    #[test]
    fn test_parse_trusted_name_tlv_length_overflow() {
        let mut out = TrustedNameOut::default();
        let mut invalid_payload = TLV_PAYLOAD.to_vec();
        invalid_payload[1] = 0x0A; // Invalid length for tag 0x01
        let res = parse_trusted_name_tlv(&invalid_payload, &mut out);
        assert_eq!(res, Err(crate::tlv::TlvError::LengthOverflow));
    }

    #[test]
    fn test_parse_trusted_name_tlv_unknown_tag() {
        let mut out = TrustedNameOut::default();
        let mut invalid_payload = TLV_PAYLOAD.to_vec();
        invalid_payload[0] = 0x09; // Unknown tag
        let res = parse_trusted_name_tlv(&invalid_payload, &mut out);
        assert_eq!(res, Err(crate::tlv::TlvError::UnknownTag));
    }

    #[test]
    fn test_parse_trusted_name_tlv_duplicate_unique_tag() {
        let mut out = TrustedNameOut::default();
        let mut invalid_payload = TLV_PAYLOAD.to_vec();
        // Duplicate tag 0x01
        invalid_payload.extend_from_slice(&[0x01, 0x01, 0x03]);
        let res = parse_trusted_name_tlv(&invalid_payload, &mut out);
        assert_eq!(res, Err(crate::tlv::TlvError::DuplicateUniqueTag));
    }
}

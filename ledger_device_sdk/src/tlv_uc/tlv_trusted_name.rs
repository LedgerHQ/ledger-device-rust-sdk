use super::*;
use crate::ecc::CurvesId;
use crate::hash::ripemd::Ripemd160;
use crate::hash::sha2::{Sha2_256, Sha2_512};
use crate::hash::sha3::{Keccak256, Sha3_256};
use crate::hash::HashInit;
use crate::pki::pki_check_signature;
use crate::tag_to_flag_u64;
use ledger_secure_sdk_sys::CERTIFICATE_PUBLIC_KEY_USAGE_TRUSTED_NAME;
extern crate alloc;
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
const TAG_0: Tag = 0x01;
const TAG_1: Tag = 0x02;
const TAG_2: Tag = 0x70;
const TAG_3: Tag = 0x71;
const TAG_4: Tag = 0x20;
const TAG_5: Tag = 0x23;
const TAG_6: Tag = 0x22;
const TAG_7: Tag = 0x72;
const TAG_8: Tag = 0x73;
const TAG_9: Tag = 0x12;
const TAG_10: Tag = 0x10;
const TAG_11: Tag = 0x13;
const TAG_12: Tag = 0x14;
const TAG_13: Tag = 0x15;

// Generate the tag_to_flag_u64 function using the macro
tag_to_flag_u64!(
    TAG_0, TAG_1, TAG_2, TAG_3, TAG_4, TAG_5, TAG_6, TAG_7, TAG_8, TAG_9, TAG_10, TAG_11, TAG_12,
    TAG_13
);

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
    pub version: u8,
    pub trusted_name_type: u8,
    pub trusted_name_source: u8,
    pub trusted_name: Vec<u8>,
    pub chain_id: u64,
    pub address: Vec<u8>,
    pub nft_id: Option<Vec<u8>>,
    pub source_contract: Option<Vec<u8>>,
    pub challenge: Option<u32>,
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
    out.trusted_name_out.trusted_name = d.as_bytes().to_vec();
    Ok(true)
}

fn on_chain_id(d: &TlvData<'_>, out: &mut TrustedNameExtracted) -> Result<bool> {
    out.trusted_name_out.chain_id = d.as_u64_be()?;
    Ok(true)
}

fn on_address(d: &TlvData<'_>, out: &mut TrustedNameExtracted) -> Result<bool> {
    out.trusted_name_out.address = d.as_bytes().to_vec();
    Ok(true)
}

fn on_nft_id(d: &TlvData<'_>, out: &mut TrustedNameExtracted) -> Result<bool> {
    out.trusted_name_out.nft_id = Some(d.as_bytes().to_vec());
    Ok(true)
}

fn on_source_contract(d: &TlvData<'_>, out: &mut TrustedNameExtracted) -> Result<bool> {
    out.trusted_name_out.source_contract = Some(d.as_bytes().to_vec());
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
    if d.tag != TAG_13 {
        let hash_updates = [
            out.hash_ctx.hash_sha2_256.update(d.raw),
            out.hash_ctx.hash_sha2_512.update(d.raw),
            out.hash_ctx.hash_sha3_256.update(d.raw),
            out.hash_ctx.hash_keccak_256.update(d.raw),
            out.hash_ctx.hash_ripemd_160.update(d.raw),
        ];

        for result in hash_updates {
            if result.is_err() {
                return Err(Error::HandlerFailed);
            }
        }
    }

    Ok(true)
}

// Static handler table
static HANDLERS: &[Handler<TrustedNameExtracted>] = &[
    Handler {
        tag: TAG_0,
        unique: true,
        func: Some(on_structure_type),
    },
    Handler {
        tag: TAG_1,
        unique: true,
        func: Some(on_version),
    },
    Handler {
        tag: TAG_2,
        unique: true,
        func: Some(on_trusted_name_type),
    },
    Handler {
        tag: TAG_3,
        unique: true,
        func: Some(on_trusted_name_source),
    },
    Handler {
        tag: TAG_4,
        unique: true,
        func: Some(on_trusted_name),
    },
    Handler {
        tag: TAG_5,
        unique: true,
        func: Some(on_chain_id),
    },
    Handler {
        tag: TAG_6,
        unique: true,
        func: Some(on_address),
    },
    Handler {
        tag: TAG_7,
        unique: true,
        func: Some(on_nft_id),
    },
    Handler {
        tag: TAG_8,
        unique: true,
        func: Some(on_source_contract),
    },
    Handler {
        tag: TAG_9,
        unique: true,
        func: Some(on_challenge),
    },
    Handler {
        tag: TAG_10,
        unique: true,
        func: Some(on_not_valid_after),
    },
    Handler {
        tag: TAG_11,
        unique: true,
        func: Some(on_signer_key_id),
    },
    Handler {
        tag: TAG_12,
        unique: true,
        func: Some(on_signer_algorithm),
    },
    Handler {
        tag: TAG_13,
        unique: true,
        func: Some(on_signature),
    },
];

/// Parse Trusted Name TLV-encoded data
pub fn parse_trusted_name_tlv(payload: &[u8], out: &mut TrustedNameOut) -> Result<()> {
    let mut extracted = TrustedNameExtracted::default();

    extracted.hash_ctx = MultipleHashContext {
        hash_sha2_256: Sha2_256::new(),
        hash_sha2_512: Sha2_512::new(),
        hash_sha3_256: Sha3_256::new(),
        hash_keccak_256: Keccak256::new(),
        hash_ripemd_160: Ripemd160::new(),
    };

    let mut cfg = ParseCfg::new(HANDLERS, tag_to_flag_u64);
    cfg.common = Some(on_common);

    parse(&cfg, payload, &mut extracted)?;

    // At this point, all TLV fields have been processed and the signature needs to be verified
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

    // Check signature with PKI certificate
    let res = pki_check_signature(
        &mut hash[..hash_size],
        CERTIFICATE_PUBLIC_KEY_USAGE_TRUSTED_NAME,
        curve,
        &mut extracted.signature,
    );
    if res.is_err() {
        return Err(Error::SignatureVerificationFailed);
    }

    // Copy the extracted trusted name output
    *out = extracted.trusted_name_out;

    Ok(())
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
            *hash_size = hash_ctx.hash_sha2_256.get_size();
            *curve = CurvesId::Secp256k1;
            let res = hash_ctx.hash_sha2_256.finalize(hash);
            if res.is_err() {
                return Err(Error::SignatureVerificationFailed);
            }
        }
        x if x
            == TlvTrustedNameSignerAlgorithm::TlvTrustedNameSignerAlgorithmEcdsaSha3_256 as u8 =>
        {
            *hash_size = hash_ctx.hash_sha3_256.get_size();
            *curve = CurvesId::Secp256k1;
            let res = hash_ctx.hash_sha3_256.finalize(hash);
            if res.is_err() {
                return Err(Error::SignatureVerificationFailed);
            }
        }
        x if x
            == TlvTrustedNameSignerAlgorithm::TlvTrustedNameSignerAlgorithmEcdsaKeccak256 as u8 =>
        {
            *hash_size = hash_ctx.hash_keccak_256.get_size();
            *curve = CurvesId::Secp256k1;
            let res = hash_ctx.hash_keccak_256.finalize(hash);
            if res.is_err() {
                return Err(Error::SignatureVerificationFailed);
            }
        }
        x if x
            == TlvTrustedNameSignerAlgorithm::TlvTrustedNameSignerAlgorithmEcdsaRipemd160 as u8 =>
        {
            *hash_size = hash_ctx.hash_ripemd_160.get_size();
            *curve = CurvesId::Secp256k1;
            let res = hash_ctx.hash_ripemd_160.finalize(hash);
            if res.is_err() {
                return Err(Error::SignatureVerificationFailed);
            }
        }
        x if x == TlvTrustedNameSignerAlgorithm::TlvTrustedNameSignerAlgorithmEcdsaSha512 as u8 => {
            *hash_size = hash_ctx.hash_sha2_512.get_size();
            *curve = CurvesId::Secp256k1;
            let res = hash_ctx.hash_sha2_512.finalize(hash);
            if res.is_err() {
                return Err(Error::SignatureVerificationFailed);
            }
        }
        x if x
            == TlvTrustedNameSignerAlgorithm::TlvTrustedNameSignerAlgorithmEddsaKeccak256 as u8 =>
        {
            *hash_size = hash_ctx.hash_keccak_256.get_size();
            *curve = CurvesId::Ed25519;
            let res = hash_ctx.hash_keccak_256.finalize(hash);
            if res.is_err() {
                return Err(Error::SignatureVerificationFailed);
            }
        }
        x if x
            == TlvTrustedNameSignerAlgorithm::TlvTrustedNameSignerAlgorithmEddsaSha3_256 as u8 =>
        {
            *hash_size = hash_ctx.hash_sha3_256.get_size();
            *curve = CurvesId::Ed25519;
            let res = hash_ctx.hash_sha3_256.finalize(hash);
            if res.is_err() {
                return Err(Error::SignatureVerificationFailed);
            }
        }
        _ => return Err(Error::SignatureVerificationFailed),
    }
    Ok(())
}

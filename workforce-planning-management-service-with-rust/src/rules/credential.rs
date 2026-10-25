//! Verifiable credentials out (WPM-R157, WPM-D109). DB-free and clock-free.
//!
//! A completed, verified CPD entry is issued as an **Open Badges 3.0** credential (1EdTech `OpenBadgeCredential`, a W3C Verifiable
//! Credential Data Model 2.0 credential), secured as a **JWT with `EdDSA` (Ed25519)**, the `vc+jwt` form of Verifiable Credentials JOSE
//! (RFC 7515 for the JWS, RFC 8037 for `EdDSA`). Revocation is a **Bitstring Status List** (W3C Recommendation, 15 May 2025):
//! a 131 072-bit list, left-most bit first, GZIP-compressed (RFC 1952) and written as multibase base64url without padding.
//!
//! Pinned: Open Badges context `context-3.0.3.json`, VC Data Model context `https://www.w3.org/ns/credentials/v2`.
//! The credential names **no person**: the worker holds it and shows it; the issuer is the employer.
//!
//! The gzip written here uses *stored* (uncompressed) blocks, which RFC 1952 and RFC 1951 allow; a list that is mostly zeroes is
//! still 16 KiB, which is the minimum the specification asks for, so nothing is gained by compressing it.

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde_json::{Value, json};
use sha2::Digest;
use uuid::Uuid;

/// The Open Badges 3.0 context this module builds to.
pub const OB_CONTEXT: &str = "https://purl.imsglobal.org/spec/ob/v3p0/context-3.0.3.json";
/// The Verifiable Credentials Data Model 2.0 context.
pub const VC_CONTEXT: &str = "https://www.w3.org/ns/credentials/v2";
/// The size of a status list in bits, the minimum the specification asks for.
pub const STATUS_BITS: usize = 131_072;

/// Base64url without padding.
#[must_use]
pub fn b64(data: &[u8]) -> String {
    URL_SAFE_NO_PAD.encode(data)
}

fn unb64(text: &str) -> Result<Vec<u8>, String> {
    URL_SAFE_NO_PAD
        .decode(text.trim())
        .map_err(|_| "not base64url".to_string())
}

/// CRC-32 (IEEE, as RFC 1952 uses).
#[must_use]
pub fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFF_u32;
    for byte in data {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            crc = if crc & 1 == 1 { (crc >> 1) ^ 0xEDB8_8320 } else { crc >> 1 };
        }
    }
    !crc
}

/// Wrap `data` in a gzip member made of stored blocks.
#[must_use]
pub fn gzip_stored(data: &[u8]) -> Vec<u8> {
    let mut out = vec![0x1f, 0x8b, 8, 0, 0, 0, 0, 0, 0, 0xff];
    let mut chunks = data.chunks(65_535).peekable();
    if data.is_empty() {
        out.extend_from_slice(&[1, 0, 0, 0xff, 0xff]);
    }
    while let Some(chunk) = chunks.next() {
        out.push(u8::from(chunks.peek().is_none()));
        let len = u16::try_from(chunk.len()).unwrap_or(u16::MAX);
        out.extend_from_slice(&len.to_le_bytes());
        out.extend_from_slice(&(!len).to_le_bytes());
        out.extend_from_slice(chunk);
    }
    out.extend_from_slice(&crc32(data).to_le_bytes());
    out.extend_from_slice(&u32::try_from(data.len() % (1 << 32)).unwrap_or(0).to_le_bytes());
    out
}

/// Read back a gzip member made of stored blocks (what [`gzip_stored`] writes), checking its CRC and length.
///
/// # Errors
/// When the bytes are not such a member.
pub fn gunzip_stored(bytes: &[u8]) -> Result<Vec<u8>, String> {
    if bytes.len() < 18 || bytes[..4] != [0x1f, 0x8b, 8, 0] {
        return Err("not a gzip member".to_string());
    }
    let mut at = 10;
    let mut out = Vec::new();
    loop {
        let header = *bytes.get(at).ok_or("truncated")?;
        if header & 0b110 != 0 {
            return Err("only stored blocks are read here".to_string());
        }
        let len = usize::from(u16::from_le_bytes([*bytes.get(at + 1).ok_or("truncated")?, *bytes.get(at + 2).ok_or("truncated")?]));
        let body = bytes.get(at + 5..at + 5 + len).ok_or("truncated")?;
        out.extend_from_slice(body);
        at += 5 + len;
        if header & 1 == 1 {
            break;
        }
    }
    let tail = bytes.get(at..at + 8).ok_or("truncated")?;
    let crc = u32::from_le_bytes([tail[0], tail[1], tail[2], tail[3]]);
    if crc != crc32(&out) {
        return Err("the checksum does not match".to_string());
    }
    Ok(out)
}

/// The encoded list for these revoked positions (multibase `u` + base64url of the gzip of the bitstring).
///
/// # Errors
/// When a position is outside the list.
pub fn encoded_list(revoked: &[usize]) -> Result<String, String> {
    let mut bits = vec![0_u8; STATUS_BITS / 8];
    for &i in revoked {
        if i >= STATUS_BITS {
            return Err(format!("position {i} is outside the list of {STATUS_BITS}"));
        }
        bits[i / 8] |= 0x80 >> (i % 8);
    }
    Ok(format!("u{}", b64(&gzip_stored(&bits))))
}

/// Whether `index` is set in an encoded list written by [`encoded_list`].
///
/// # Errors
/// When the list is malformed, or the index is outside it.
pub fn is_revoked(encoded: &str, index: usize) -> Result<bool, String> {
    let body = encoded.strip_prefix('u').ok_or("a multibase base64url list starts with u")?;
    let bits = gunzip_stored(&unb64(body)?)?;
    let byte = bits.get(index / 8).ok_or("the index is outside the list")?;
    Ok(byte & (0x80 >> (index % 8)) != 0)
}

/// SHA-256 of a text, lower-case hex: what an issuance record keeps to show which credential it was, without the credential.
#[must_use]
pub fn sha256_hex(text: &str) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    sha2::Sha256::digest(text.as_bytes())
        .iter()
        .flat_map(|b| [HEX[usize::from(b >> 4)], HEX[usize::from(b & 15)]])
        .map(char::from)
        .collect()
}

/// The signing key from its text form: 32 bytes, base64url.
///
/// # Errors
/// When the text is not 32 bytes of base64url.
pub fn parse_seed(text: &str) -> Result<[u8; 32], String> {
    unb64(text)?
        .try_into()
        .map_err(|_| "the signing key must be 32 bytes, base64url".to_string())
}

/// The public key as a JWK (RFC 8037), with its `kid`.
#[must_use]
pub fn jwk(seed: &[u8; 32], kid: &str) -> Value {
    let key = SigningKey::from_bytes(seed).verifying_key();
    json!({ "kty": "OKP", "crv": "Ed25519", "x": b64(key.as_bytes()), "kid": kid, "use": "sig", "alg": "EdDSA" })
}

/// Sign `payload` as a compact JWS (`vc+jwt`, `EdDSA`).
#[must_use]
pub fn sign(seed: &[u8; 32], kid: &str, payload: &Value) -> String {
    let header = json!({ "alg": "EdDSA", "kid": kid, "typ": "vc+jwt", "cty": "vc" });
    let input = format!("{}.{}", b64(header.to_string().as_bytes()), b64(payload.to_string().as_bytes()));
    let signature = SigningKey::from_bytes(seed).sign(input.as_bytes());
    format!("{input}.{}", b64(&signature.to_bytes()))
}

/// Check a compact JWS against a public key and return its payload.
///
/// # Errors
/// When it is malformed, not `EdDSA`, or the signature does not verify.
pub fn verify(public: &[u8; 32], token: &str) -> Result<Value, String> {
    let mut parts = token.split('.');
    let (Some(h), Some(p), Some(s), None) = (parts.next(), parts.next(), parts.next(), parts.next()) else {
        return Err("a compact JWS has three parts".to_string());
    };
    let header: Value = serde_json::from_slice(&unb64(h)?).map_err(|_| "bad header")?;
    if header["alg"] != "EdDSA" {
        return Err("only EdDSA is accepted".to_string());
    }
    let key = VerifyingKey::from_bytes(public).map_err(|_| "bad public key")?;
    let sig = Signature::from_slice(&unb64(s)?).map_err(|_| "bad signature")?;
    key.verify(format!("{h}.{p}").as_bytes(), &sig)
        .map_err(|_| "the signature does not verify".to_string())?;
    serde_json::from_slice(&unb64(p)?).map_err(|_| "bad payload".to_string())
}

/// What an issued badge says.
#[derive(Debug, Clone)]
pub struct Badge<'a> {
    /// The issuer's URL (also its profile id).
    pub issuer_id: &'a str,
    /// The issuer's name.
    pub issuer_name: &'a str,
    /// The issuance's id.
    pub issuance: Uuid,
    /// What was achieved.
    pub name: &'a str,
    /// A sentence on what the achievement is.
    pub description: &'a str,
    /// How it was earned.
    pub criteria: &'a str,
    /// When it takes effect, `YYYY-MM-DDTHH:MM:SSZ`.
    pub valid_from: &'a str,
    /// Where the revocation list is.
    pub status_list_url: &'a str,
    /// The position in it.
    pub status_index: usize,
    /// The skill's references in other frameworks (ESCO, SFIA), when it has any. None are guessed.
    pub alignments: &'a [Alignment<'a>],
}

/// A reference to a skill in another framework.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Alignment<'a> {
    /// The framework as people name it (`ESCO`, `SFIA`).
    pub framework: &'a str,
    /// Its identifier: a URI for ESCO, a code for SFIA.
    pub code: &'a str,
    /// Its label.
    pub label: &'a str,
}

/// The framework's name as it is shown: the slugs WPM stores for the two it knows, else the slug itself.
#[must_use]
pub fn framework_name(slug: &str) -> String {
    match slug {
        "esco" => "ESCO".to_string(),
        "sfia" => "SFIA".to_string(),
        other => other.to_string(),
    }
}

/// The Open Badges 3.0 credential. It carries no name, number, email or id of the worker.
#[must_use]
pub fn badge(b: &Badge<'_>) -> Value {
    let mut achievement = json!({
        "id": format!("urn:uuid:{}", b.issuance),
        "type": ["Achievement"],
        "achievementType": "Course",
        "name": b.name,
        "description": b.description,
        "criteria": { "narrative": b.criteria },
    });
    if !b.alignments.is_empty() {
        achievement["alignment"] = Value::Array(
            b.alignments
                .iter()
                .map(|a| {
                    let mut one = json!({
                        "type": ["Alignment"], "targetName": a.label, "targetFramework": a.framework, "targetCode": a.code,
                    });
                    // An ESCO reference is a URI; Open Badges has a place for a target that is a link.
                    if a.code.starts_with("http://") || a.code.starts_with("https://") {
                        one["targetUrl"] = json!(a.code);
                    }
                    one
                })
                .collect(),
        );
    }
    json!({
        "@context": [VC_CONTEXT, OB_CONTEXT],
        "id": format!("urn:uuid:{}", b.issuance),
        "type": ["VerifiableCredential", "OpenBadgeCredential"],
        "name": b.name,
        "issuer": { "id": b.issuer_id, "type": ["Profile"], "name": b.issuer_name },
        "validFrom": b.valid_from,
        "credentialSubject": { "type": ["AchievementSubject"], "achievement": achievement },
        "credentialStatus": {
            "id": format!("{}#{}", b.status_list_url, b.status_index),
            "type": "BitstringStatusListEntry",
            "statusPurpose": "revocation",
            "statusListIndex": b.status_index.to_string(),
            "statusListCredential": b.status_list_url,
        },
    })
}

/// The status list credential.
#[must_use]
pub fn status_list(issuer_id: &str, list_url: &str, valid_from: &str, encoded: &str) -> Value {
    json!({
        "@context": [VC_CONTEXT],
        "id": list_url,
        "type": ["VerifiableCredential", "BitstringStatusListCredential"],
        "issuer": issuer_id,
        "validFrom": valid_from,
        "credentialSubject": {
            "id": format!("{list_url}#list"),
            "type": "BitstringStatusList",
            "statusPurpose": "revocation",
            "encodedList": encoded,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const SEED: &str = "nWGxne_9WmC6hEr0kuwsxERJxWl7MmkZcDusAxyuf2A";

    fn sample<'a>() -> Badge<'a> {
        Badge {
            issuer_id: "https://hr.example.org/",
            issuer_name: "Example Employer",
            issuance: Uuid::nil(),
            name: "Safeguarding refresher",
            description: "A CPD activity.",
            criteria: "Completed and verified.",
            valid_from: "2026-10-25T09:00:00Z",
            status_list_url: "https://hr.example.org/api/credentials/status-list",
            status_index: 7,
            alignments: &[Alignment { framework: "ESCO", code: "http://data.europa.eu/esco/skill/abc", label: "safeguarding" }],
        }
    }

    /// RFC 8037 appendix A.4: the example Ed25519 key signs the example input to the published signature.
    #[test]
    fn ed25519_matches_the_rfc_8037_example() {
        let seed = parse_seed(SEED).unwrap();
        let key = SigningKey::from_bytes(&seed);
        assert_eq!(b64(key.verifying_key().as_bytes()), "11qYAYKxCrfVS_7TyWQHOg7hcvPapiMlrwIaaPcHURo");
        let input = "eyJhbGciOiJFZERTQSJ9.RXhhbXBsZSBvZiBFZDI1NTE5IHNpZ25pbmc";
        assert_eq!(
            b64(&key.sign(input.as_bytes()).to_bytes()),
            "hgyY0il_MGCjP0JzlnLWG1PPOt7-09PGcvMg3AIbQR6dWbhijcNR4ki4iylGjg5BhVsPt9g7sVvpAr_MuM0KAg"
        );
    }

    #[test]
    fn sha256_matches_the_fips_180_example() {
        assert_eq!(sha256_hex("abc"), "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
    }

    #[test]
    fn crc32_matches_the_standard_check_value() {
        assert_eq!(crc32(b"123456789"), 0xCBF4_3926);
        assert_eq!(crc32(b""), 0);
    }

    #[test]
    fn gzip_of_stored_blocks_round_trips_and_has_the_gzip_magic() {
        for len in [0_usize, 1, 100, 65_535, 65_536, 131_072 / 8] {
            let data: Vec<u8> = (0..len).map(|i| u8::try_from(i % 251).unwrap()).collect();
            let z = gzip_stored(&data);
            assert_eq!(&z[..3], &[0x1f, 0x8b, 8]);
            assert_eq!(gunzip_stored(&z).unwrap(), data, "{len}");
        }
        let mut z = gzip_stored(b"abc");
        let last = z.len() - 9;
        z[last] ^= 1;
        assert!(gunzip_stored(&z).is_err(), "a changed byte fails the checksum");
    }

    #[test]
    fn the_list_is_131072_bits_leftmost_first_and_marks_only_what_was_revoked() {
        let encoded = encoded_list(&[0, 7, 8, 131_071]).unwrap();
        assert!(encoded.starts_with('u'));
        let bits = gunzip_stored(&unb64(&encoded[1..]).unwrap()).unwrap();
        assert_eq!(bits.len(), 16_384);
        assert_eq!(bits[0], 0b1000_0001, "positions 0 and 7");
        assert_eq!(bits[1], 0b1000_0000, "position 8");
        assert_eq!(bits[16_383], 0b0000_0001, "the last position");
        for (i, want) in [(0, true), (1, false), (7, true), (8, true), (9, false), (131_071, true), (131_070, false)] {
            assert_eq!(is_revoked(&encoded, i).unwrap(), want, "{i}");
        }
        assert!(encoded_list(&[131_072]).is_err());
        assert!(is_revoked(&encoded, 131_072).is_err());
    }

    #[test]
    fn a_signed_credential_verifies_and_a_changed_one_does_not() {
        let seed = parse_seed(SEED).unwrap();
        let public = *SigningKey::from_bytes(&seed).verifying_key().as_bytes();
        let token = sign(&seed, "https://hr.example.org/#key-1", &badge(&sample()));
        let payload = verify(&public, &token).unwrap();
        assert_eq!(payload["type"][1], "OpenBadgeCredential");
        let mut parts: Vec<String> = token.split('.').map(str::to_string).collect();
        parts[1] = b64(json!({ "tampered": true }).to_string().as_bytes());
        assert!(verify(&public, &parts.join(".")).is_err());
        assert!(verify(&[7; 32], &token).is_err(), "another key");
        assert!(verify(&public, "a.b").is_err());
    }

    #[test]
    fn a_token_that_names_another_algorithm_is_refused_even_with_a_good_signature() {
        let seed = parse_seed(SEED).unwrap();
        let public = *SigningKey::from_bytes(&seed).verifying_key().as_bytes();
        let input = format!("{}.{}", b64(json!({ "alg": "none" }).to_string().as_bytes()), b64(b"{}"));
        let signature = SigningKey::from_bytes(&seed).sign(input.as_bytes());
        let token = format!("{input}.{}", b64(&signature.to_bytes()));
        assert_eq!(verify(&public, &token), Err("only EdDSA is accepted".to_string()));
    }

    #[test]
    fn the_header_says_what_it_is() {
        let seed = parse_seed(SEED).unwrap();
        let token = sign(&seed, "k#1", &json!({}));
        let header: Value = serde_json::from_slice(&unb64(token.split('.').next().unwrap()).unwrap()).unwrap();
        assert_eq!(header, json!({ "alg": "EdDSA", "kid": "k#1", "typ": "vc+jwt", "cty": "vc" }));
    }

    #[test]
    fn the_badge_follows_the_pinned_shape_and_names_no_person() {
        let v = badge(&sample());
        assert_eq!(v["@context"], json!([VC_CONTEXT, OB_CONTEXT]));
        assert_eq!(v["type"], json!(["VerifiableCredential", "OpenBadgeCredential"]));
        assert_eq!(v["credentialSubject"]["type"], json!(["AchievementSubject"]));
        assert_eq!(v["credentialSubject"]["achievement"]["alignment"][0]["targetFramework"], "ESCO");
        assert_eq!(v["credentialStatus"]["type"], "BitstringStatusListEntry");
        assert_eq!(v["credentialStatus"]["statusListIndex"], "7");
        assert_eq!(v["credentialStatus"]["statusPurpose"], "revocation");
        assert!(v["credentialSubject"].get("id").is_none(), "no subject identifier");
        assert!(v["credentialSubject"].get("identifier").is_none());
        assert_eq!(v["credentialSubject"]["achievement"]["alignment"][0]["targetUrl"], "http://data.europa.eu/esco/skill/abc");
        let mut plain = sample();
        plain.alignments = &[];
        assert!(badge(&plain)["credentialSubject"]["achievement"].get("alignment").is_none(), "none is not guessed");
        let sfia = [Alignment { framework: "SFIA", code: "PROG", label: "Programming" }];
        plain.alignments = &sfia;
        let a = &badge(&plain)["credentialSubject"]["achievement"]["alignment"][0];
        assert_eq!((a["targetFramework"].as_str(), a["targetCode"].as_str()), (Some("SFIA"), Some("PROG")));
        assert!(a.get("targetUrl").is_none(), "a code is not a link");
        assert_eq!((framework_name("esco"), framework_name("sfia"), framework_name("x")), ("ESCO".to_string(), "SFIA".to_string(), "x".to_string()));
    }

    #[test]
    fn the_status_list_credential_has_the_required_properties() {
        let v = status_list("https://hr.example.org/", "https://hr.example.org/l", "2026-10-25T09:00:00Z", "uAAA");
        assert_eq!(v["type"], json!(["VerifiableCredential", "BitstringStatusListCredential"]));
        assert_eq!(v["credentialSubject"]["type"], "BitstringStatusList");
        assert_eq!(v["credentialSubject"]["statusPurpose"], "revocation");
        assert_eq!(v["credentialSubject"]["encodedList"], "uAAA");
    }

    #[test]
    fn a_signing_key_must_be_32_bytes_of_base64url() {
        assert!(parse_seed(SEED).is_ok());
        assert!(parse_seed("AAAA").is_err());
        assert!(parse_seed("not base64!").is_err());
    }

    #[test]
    fn the_jwk_is_the_public_half_only() {
        let seed = parse_seed(SEED).unwrap();
        let key = jwk(&seed, "k");
        assert_eq!(key["x"], "11qYAYKxCrfVS_7TyWQHOg7hcvPapiMlrwIaaPcHURo");
        assert!(key.get("d").is_none(), "the private half never leaves");
    }
}

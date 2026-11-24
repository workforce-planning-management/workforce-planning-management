//! `jsonwebtoken`'s cryptography on `ring` (WPM-T536), so the `keycloak` build has no `aws-lc-rs`.
//!
//! `jsonwebtoken` 11 ships two backends, `aws_lc_rs` and `rust_crypto`; the second brings the `rsa` crate
//! (RUSTSEC-2023-0071), and the first a second TLS-capable crypto library beside the `ring` that rustls already uses. It also
//! takes a [`CryptoProvider`] of the program's own, and this is one, on `ring`. It follows the shape of `jsonwebtoken`'s own
//! `crypto::aws_lc` module (MIT licence, Copyright (c) 2015 Vincent Prouillet), whose calls `ring` mirrors.
//!
//! **What it does.** RSA (`RS256`, `RS384`, `RS512`, `PS256`, `PS384`, `PS512`), ECDSA (`ES256`, `ES384`) and `EdDSA`, to
//! sign and to verify. Keycloak's keys are `RS256` or `ES256`, and [`super::keycloak`] accepts only those two; signing is for
//! the tests. **What it refuses to verify:** the HMAC algorithms (`HS256`, `HS384`, `HS512`), which WPM never accepts: a shared
//! secret is not how a realm signs, and refusing them here closes the door on an algorithm-confusion token whatever the caller
//! allows. It still signs them, which is harmless and lets a test make exactly such a token to show it is refused. Also refused: reading RSA components from a DER public key, which `ring` cannot parse. Only `Jwk::from_encoding_key`
//! needs it, and WPM does not call that.

use jsonwebtoken::crypto::{CryptoProvider, JwtSigner, JwtVerifier, KeyUtils};
use jsonwebtoken::errors::{ErrorKind, Result, new_error};
use jsonwebtoken::jwk::{EllipticCurve, ThumbprintHash};
use jsonwebtoken::signature::{self, Signer, Verifier};
use jsonwebtoken::{Algorithm, AlgorithmFamily, DecodingKey, DecodingKeyKind, EncodingKey};
use ring::rand::SystemRandom;
use ring::signature as sig;

/// The provider, for [`install`].
static PROVIDER: CryptoProvider = CryptoProvider {
    signer_factory: new_signer,
    verifier_factory: new_verifier,
    key_utils: KeyUtils {
        rsa_pub_components_from_private_key: rsa_components_from_private_key,
        rsa_pub_components_from_public_key: rsa_components_from_public_key,
        ec_pub_components_from_private_key: ec_components_from_private_key,
        ed_pub_components_from_private_key: ed_component_from_private_key,
        compute_digest,
    },
};

/// Make this `jsonwebtoken`'s provider for the process, unless one is chosen already. With neither of its own backends built,
/// `jsonwebtoken` panics on first use without one, so everything that signs or verifies a JWT calls this first. Safe to call
/// more than once.
pub fn install() {
    let _ = PROVIDER.install_default();
}

fn refused(algorithm: Algorithm) -> jsonwebtoken::errors::Error {
    let _ = algorithm;
    new_error(ErrorKind::InvalidAlgorithm)
}

#[expect(
    clippy::trivially_copy_pass_by_ref,
    reason = "the signature `CryptoProvider::signer_factory` requires"
)]
fn new_signer(algorithm: &Algorithm, key: &EncodingKey) -> Result<Box<dyn JwtSigner>> {
    Ok(match *algorithm {
        a @ (Algorithm::RS256
        | Algorithm::RS384
        | Algorithm::RS512
        | Algorithm::PS256
        | Algorithm::PS384
        | Algorithm::PS512) => Box::new(RsaSigner::new(a, key)?),
        a @ (Algorithm::ES256 | Algorithm::ES384) => Box::new(EcdsaSigner::new(a, key)?),
        Algorithm::EdDSA => Box::new(EdDsaSigner::new(key)?),
        a @ (Algorithm::HS256 | Algorithm::HS384 | Algorithm::HS512) => {
            Box::new(HmacSigner::new(a, key)?)
        }
        // Any algorithm a later `jsonwebtoken` adds (`Algorithm` is non-exhaustive).
        a => return Err(refused(a)),
    })
}

#[expect(
    clippy::trivially_copy_pass_by_ref,
    reason = "the signature `CryptoProvider::verifier_factory` requires"
)]
fn new_verifier(algorithm: &Algorithm, key: &DecodingKey) -> Result<Box<dyn JwtVerifier>> {
    let family = match *algorithm {
        Algorithm::RS256
        | Algorithm::RS384
        | Algorithm::RS512
        | Algorithm::PS256
        | Algorithm::PS384
        | Algorithm::PS512 => AlgorithmFamily::Rsa,
        Algorithm::ES256 | Algorithm::ES384 => AlgorithmFamily::Ec,
        Algorithm::EdDSA => AlgorithmFamily::Ed,
        a => return Err(refused(a)),
    };
    if key.family() != family {
        return Err(new_error(ErrorKind::InvalidKeyFormat));
    }
    Ok(Box::new(RingVerifier {
        algorithm: *algorithm,
        key: key.clone(),
    }))
}

/// The `ring` signing parameters for an RSA algorithm.
fn rsa_signing(algorithm: Algorithm) -> &'static dyn sig::RsaEncoding {
    match algorithm {
        Algorithm::RS384 => &sig::RSA_PKCS1_SHA384,
        Algorithm::RS512 => &sig::RSA_PKCS1_SHA512,
        Algorithm::PS256 => &sig::RSA_PSS_SHA256,
        Algorithm::PS384 => &sig::RSA_PSS_SHA384,
        Algorithm::PS512 => &sig::RSA_PSS_SHA512,
        _ => &sig::RSA_PKCS1_SHA256,
    }
}

/// The `ring` verification parameters for an RSA algorithm (keys of 2048 to 8192 bits, as `jsonwebtoken`'s own backends).
fn rsa_verification(algorithm: Algorithm) -> &'static sig::RsaParameters {
    match algorithm {
        Algorithm::RS384 => &sig::RSA_PKCS1_2048_8192_SHA384,
        Algorithm::RS512 => &sig::RSA_PKCS1_2048_8192_SHA512,
        Algorithm::PS256 => &sig::RSA_PSS_2048_8192_SHA256,
        Algorithm::PS384 => &sig::RSA_PSS_2048_8192_SHA384,
        Algorithm::PS512 => &sig::RSA_PSS_2048_8192_SHA512,
        _ => &sig::RSA_PKCS1_2048_8192_SHA256,
    }
}

/// The `ring` ECDSA algorithms, to sign and to verify, in the fixed-length form JWS uses.
fn ecdsa(
    algorithm: Algorithm,
) -> (
    &'static sig::EcdsaSigningAlgorithm,
    &'static sig::EcdsaVerificationAlgorithm,
) {
    if algorithm == Algorithm::ES384 {
        (
            &sig::ECDSA_P384_SHA384_FIXED_SIGNING,
            &sig::ECDSA_P384_SHA384_FIXED,
        )
    } else {
        (
            &sig::ECDSA_P256_SHA256_FIXED_SIGNING,
            &sig::ECDSA_P256_SHA256_FIXED,
        )
    }
}

struct RsaSigner {
    algorithm: Algorithm,
    key: sig::RsaKeyPair,
}

impl RsaSigner {
    fn new(algorithm: Algorithm, key: &EncodingKey) -> Result<Self> {
        if key.family() != AlgorithmFamily::Rsa {
            return Err(new_error(ErrorKind::InvalidKeyFormat));
        }
        let key = sig::RsaKeyPair::from_der(key.as_bytes())
            .map_err(|e| new_error(ErrorKind::InvalidRsaKey(e.to_string())))?;
        Ok(Self { algorithm, key })
    }
}

impl Signer<Vec<u8>> for RsaSigner {
    fn try_sign(&self, msg: &[u8]) -> std::result::Result<Vec<u8>, signature::Error> {
        let mut out = vec![0; self.key.public().modulus_len()];
        self.key
            .sign(
                rsa_signing(self.algorithm),
                &SystemRandom::new(),
                msg,
                &mut out,
            )
            .map_err(signature::Error::from_source)?;
        Ok(out)
    }
}

impl JwtSigner for RsaSigner {
    fn algorithm(&self) -> Algorithm {
        self.algorithm
    }
}

struct EcdsaSigner {
    algorithm: Algorithm,
    key: sig::EcdsaKeyPair,
}

impl EcdsaSigner {
    fn new(algorithm: Algorithm, key: &EncodingKey) -> Result<Self> {
        if key.family() != AlgorithmFamily::Ec {
            return Err(new_error(ErrorKind::InvalidKeyFormat));
        }
        let key =
            sig::EcdsaKeyPair::from_pkcs8(ecdsa(algorithm).0, key.as_bytes(), &SystemRandom::new())
                .map_err(|_| new_error(ErrorKind::InvalidEcdsaKey))?;
        Ok(Self { algorithm, key })
    }
}

impl Signer<Vec<u8>> for EcdsaSigner {
    fn try_sign(&self, msg: &[u8]) -> std::result::Result<Vec<u8>, signature::Error> {
        let signed = self
            .key
            .sign(&SystemRandom::new(), msg)
            .map_err(signature::Error::from_source)?;
        Ok(signed.as_ref().to_vec())
    }
}

impl JwtSigner for EcdsaSigner {
    fn algorithm(&self) -> Algorithm {
        self.algorithm
    }
}

struct EdDsaSigner(sig::Ed25519KeyPair);

impl EdDsaSigner {
    fn new(key: &EncodingKey) -> Result<Self> {
        if key.family() != AlgorithmFamily::Ed {
            return Err(new_error(ErrorKind::InvalidKeyFormat));
        }
        sig::Ed25519KeyPair::from_pkcs8(key.as_bytes())
            .map(Self)
            .map_err(|_| new_error(ErrorKind::InvalidEddsaKey))
    }
}

impl Signer<Vec<u8>> for EdDsaSigner {
    fn try_sign(&self, msg: &[u8]) -> std::result::Result<Vec<u8>, signature::Error> {
        Ok(self.0.sign(msg).as_ref().to_vec())
    }
}

impl JwtSigner for EdDsaSigner {
    fn algorithm(&self) -> Algorithm {
        Algorithm::EdDSA
    }
}

/// HMAC, to sign only (see the module's note): verifying it is refused in [`new_verifier`].
struct HmacSigner {
    algorithm: Algorithm,
    key: ring::hmac::Key,
}

impl HmacSigner {
    fn new(algorithm: Algorithm, key: &EncodingKey) -> Result<Self> {
        if key.family() != AlgorithmFamily::Hmac {
            return Err(new_error(ErrorKind::InvalidKeyFormat));
        }
        let hmac = match algorithm {
            Algorithm::HS384 => ring::hmac::HMAC_SHA384,
            Algorithm::HS512 => ring::hmac::HMAC_SHA512,
            _ => ring::hmac::HMAC_SHA256,
        };
        Ok(Self {
            algorithm,
            key: ring::hmac::Key::new(hmac, key.as_bytes()),
        })
    }
}

impl Signer<Vec<u8>> for HmacSigner {
    fn try_sign(&self, msg: &[u8]) -> std::result::Result<Vec<u8>, signature::Error> {
        Ok(ring::hmac::sign(&self.key, msg).as_ref().to_vec())
    }
}

impl JwtSigner for HmacSigner {
    fn algorithm(&self) -> Algorithm {
        self.algorithm
    }
}

/// One verifier for every algorithm this provider takes: the key's family was checked against the algorithm when it was made.
struct RingVerifier {
    algorithm: Algorithm,
    key: DecodingKey,
}

impl Verifier<Vec<u8>> for RingVerifier {
    fn verify(&self, msg: &[u8], signature: &Vec<u8>) -> std::result::Result<(), signature::Error> {
        let bytes = || {
            self.key
                .try_get_as_bytes()
                .map_err(signature::Error::from_source)
        };
        match self.algorithm {
            Algorithm::ES256 | Algorithm::ES384 => {
                sig::UnparsedPublicKey::new(ecdsa(self.algorithm).1, bytes()?)
                    .verify(msg, signature)
            }
            Algorithm::EdDSA => {
                sig::UnparsedPublicKey::new(&sig::ED25519, bytes()?).verify(msg, signature)
            }
            _ => {
                let params = rsa_verification(self.algorithm);
                match self.key.kind() {
                    DecodingKeyKind::SecretOrDer(der) => {
                        sig::UnparsedPublicKey::new(params, der).verify(msg, signature)
                    }
                    DecodingKeyKind::RsaModulusExponent { n, e } => {
                        sig::RsaPublicKeyComponents { n, e }.verify(params, msg, signature)
                    }
                }
            }
        }
        .map_err(signature::Error::from_source)
    }
}

impl JwtVerifier for RingVerifier {
    fn algorithm(&self) -> Algorithm {
        self.algorithm
    }
}

fn rsa_components_from_private_key(der: &[u8]) -> Result<(Vec<u8>, Vec<u8>)> {
    let key = sig::RsaKeyPair::from_der(der)
        .map_err(|e| new_error(ErrorKind::InvalidRsaKey(e.to_string())))?;
    let components = sig::RsaPublicKeyComponents::<Vec<u8>>::from(key.public());
    Ok((components.n, components.e))
}

fn rsa_components_from_public_key(_der: &[u8]) -> Result<(Vec<u8>, Vec<u8>)> {
    Err(new_error(ErrorKind::InvalidRsaKey(
        "reading a DER public key is not supported by WPM's ring provider".to_string(),
    )))
}

fn ec_components_from_private_key(
    der: &[u8],
    algorithm: Algorithm,
) -> Result<(EllipticCurve, Vec<u8>, Vec<u8>)> {
    let (curve, half): (_, usize) = match algorithm {
        Algorithm::ES256 => (EllipticCurve::P256, 32),
        Algorithm::ES384 => (EllipticCurve::P384, 48),
        _ => return Err(new_error(ErrorKind::InvalidEcdsaKey)),
    };
    let key = sig::EcdsaKeyPair::from_pkcs8(ecdsa(algorithm).0, der, &SystemRandom::new())
        .map_err(|_| new_error(ErrorKind::InvalidEcdsaKey))?;
    let public = sig::KeyPair::public_key(&key).as_ref();
    // An uncompressed point: 0x04, then x and y.
    match public.split_first() {
        Some((4, xy)) if half.checked_mul(2) == Some(xy.len()) => {
            let (x, y) = xy.split_at(half);
            Ok((curve, x.to_vec(), y.to_vec()))
        }
        _ => Err(new_error(ErrorKind::InvalidEcdsaKey)),
    }
}

fn ed_component_from_private_key(der: &[u8], curve: &EllipticCurve) -> Result<Vec<u8>> {
    match curve {
        EllipticCurve::Ed25519 => sig::Ed25519KeyPair::from_pkcs8(der)
            .map(|key| sig::KeyPair::public_key(&key).as_ref().to_vec())
            .map_err(|_| new_error(ErrorKind::InvalidEddsaKey)),
        _ => Err(new_error(ErrorKind::InvalidAlgorithm)),
    }
}

#[expect(
    clippy::needless_pass_by_value,
    reason = "the signature `KeyUtils::compute_digest` requires"
)]
fn compute_digest(data: &[u8], hash: ThumbprintHash) -> Result<Vec<u8>> {
    let algorithm = match hash {
        ThumbprintHash::SHA256 => &ring::digest::SHA256,
        ThumbprintHash::SHA384 => &ring::digest::SHA384,
        ThumbprintHash::SHA512 => &ring::digest::SHA512,
        _ => return Err(new_error(ErrorKind::InvalidAlgorithm)),
    };
    Ok(ring::digest::digest(algorithm, data).as_ref().to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::Engine as _;
    use jsonwebtoken::{Header, Validation, decode, encode};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Claims {
        sub: String,
        exp: u64,
    }

    fn claims() -> Claims {
        Claims {
            sub: "worker:1".to_string(),
            exp: 4_102_444_800, // 2100-01-01
        }
    }

    fn es256_pkcs8() -> Vec<u8> {
        sig::EcdsaKeyPair::generate_pkcs8(
            &sig::ECDSA_P256_SHA256_FIXED_SIGNING,
            &SystemRandom::new(),
        )
        .expect("a key")
        .as_ref()
        .to_vec()
    }

    #[test]
    fn es256_signs_and_verifies_and_a_changed_token_fails() {
        install();
        let der = es256_pkcs8();
        let (_, x, y) = ec_components_from_private_key(&der, Algorithm::ES256).expect("components");
        let b64 = |b: &[u8]| base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(b);
        let decoding = DecodingKey::from_ec_components(&b64(&x), &b64(&y)).expect("a public key");
        let token = encode(
            &Header::new(Algorithm::ES256),
            &claims(),
            &EncodingKey::from_ec_der(&der),
        )
        .expect("signed");
        let read = decode::<Claims>(&token, &decoding, &Validation::new(Algorithm::ES256))
            .expect("verified");
        assert_eq!(read.claims, claims());
        // One character of the signature changed.
        let mut tampered = token.into_bytes();
        let last = tampered.len() - 2;
        tampered[last] = if tampered[last] == b'A' { b'B' } else { b'A' };
        let tampered = String::from_utf8(tampered).expect("ascii");
        assert!(
            decode::<Claims>(&tampered, &decoding, &Validation::new(Algorithm::ES256)).is_err()
        );
    }

    #[test]
    fn a_key_of_another_family_is_refused_and_hmac_is_never_verified() {
        install();
        let der = es256_pkcs8();
        assert!(new_signer(&Algorithm::RS256, &EncodingKey::from_ec_der(&der)).is_err());
        assert!(new_verifier(&Algorithm::RS256, &DecodingKey::from_ec_der(&der)).is_err());
        // An HS256 token can be made (a test needs one to show it is refused) but never verified.
        let secret = b"a shared secret";
        let token = encode(
            &Header::new(Algorithm::HS256),
            &claims(),
            &EncodingKey::from_secret(secret).expect("a key"),
        )
        .expect("signed");
        let read = decode::<Claims>(
            &token,
            &DecodingKey::from_secret(secret).expect("a key"),
            &Validation::new(Algorithm::HS256),
        );
        assert!(read.is_err());
        assert!(
            new_verifier(
                &Algorithm::HS256,
                &DecodingKey::from_secret(secret).expect("a key")
            )
            .is_err()
        );
    }

    #[test]
    fn digests_are_sha2() {
        assert_eq!(
            compute_digest(b"", ThumbprintHash::SHA256).expect("a digest"),
            ring::digest::digest(&ring::digest::SHA256, b"").as_ref()
        );
    }
}

# OIDC test keys

Two throwaway RSA-2048 private keys used **only** by `tests/oidc_entra.rs` to sign
tokens for an in-process, Entra-compatible OIDC provider. They protect nothing: they
were generated for the test, are never loaded by the service, and appear in no
configuration. `test-signing-key.pem` is the provider's key; `other-signing-key.pem` is
an attacker's, to prove a token signed by any other key is refused.

Do not reuse them anywhere. (A secret scanner may flag them; this file is why that is a
false positive.)

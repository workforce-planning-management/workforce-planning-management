# Sign in with Microsoft Entra ID (WPM-R93, WPM-D62)

Two ways to let people sign in with an Entra ID (formerly Azure AD) account. Pick one.

| | **Direct** | **Brokered through Keycloak** |
| --- | --- | --- |
| How | The API verifies Entra access tokens against the tenant's published keys | Keycloak federates to Entra as an identity provider and issues the tokens; the API verifies Keycloak's |
| API setting | The OIDC backend (`--no-default-features --features keycloak`) with the Entra values below | The same backend with Keycloak's values (see [auth.md](../auth.md)) |
| Who runs the browser redirect | Your UI or broker (the authentication service) | Keycloak |
| Roles come from | Entra **app roles** | Keycloak realm roles, mapped from Entra groups or claims |
| Best when | You have one tenant and want fewest moving parts | You need several identity providers, or role mapping Entra cannot express |

> WPM never holds a client secret or an authorization code (WPM-D62). The interactive
> sign-in, consent and token exchange belong to whichever component runs the browser
> redirect. WPM verifies the token it is handed.

## Direct: set up Entra

1. **App registration** (Entra admin centre, App registrations, New registration): a name,
   single tenant, and the redirect URIs of whatever runs the sign-in (your UI or broker).
   Record the **Application (client) ID** and **Directory (tenant) ID**.
2. **Expose an API**: set an Application ID URI, and a scope for the UI or broker to request
   (for example `access_as_user`).
3. **Token version**: in the application manifest set `"requestedAccessTokenVersion": 2`
   (the field is `accessTokenAcceptedVersion` in the older manifest). WPM expects v2 tokens,
   whose issuer is `https://login.microsoftonline.com/<tenant>/v2.0` and whose audience is the
   client ID. A v1 token (issuer `sts.windows.net`, audience an `api://…` URI) is refused.
4. **App roles** (App roles, Create): add roles whose **Value** is exactly one of the personas
   below, allowed for Users/Groups, then assign them to users or groups in **Enterprise
   applications, Users and groups**. Entra puts them in the token's `roles` claim.
5. **Optional claims**: add `email` if the API should record it. Do not rely on `groups` for
   departments: Entra sends group object ids, not names (see limits).

| App role value | Persona | Effect (see [auth.md](../auth.md)) |
| --- | --- | --- |
| `wpm-hr` | HR | Writes, and reads masked |
| `wpm-payroll` | Payroll | Reads salary and payslips unmasked |
| `wpm-svc` | Service | Machine callers: does everything. **Only for service principals** |
| `wpm-admin` | Administrator | `access = admin`: erase, sweep, import, merge |
| *(no role)* | Employee | Authenticated reader; writes limited to the person's own |

## Direct: configure the API

```sh
WPM_KEYCLOAK_JWKS_URL=https://login.microsoftonline.com/<tenant>/discovery/v2.0/keys
WPM_KEYCLOAK_ISSUER=https://login.microsoftonline.com/<tenant>/v2.0
WPM_KEYCLOAK_AUDIENCE=<application (client) id>
WPM_OIDC_SUBJECT_CLAIM=oid
WPM_ABAC_POLICY_FILE=/etc/wpm/abac-policy.json   # start from config/abac-policy.reference.json
LOCO_ENV=production
```

You can read the first two from the tenant's discovery document:
`https://login.microsoftonline.com/<tenant>/v2.0/.well-known/openid-configuration`
(`jwks_uri` and `issuer`).

**Why `oid`.** In Entra, a token's `sub` is different for every application, so a person has
a different `sub` in every app and `resource.person = $sub` self-rules, the audit actor and
the link to a person record would not line up. The object id (`oid`) is the same across the
tenant. With `WPM_OIDC_SUBJECT_CLAIM=oid` the subject is the `oid`, and **a token with no
`oid` is refused** rather than silently falling back to `sub`.

The key set is fetched at boot and refreshed every hour (`WPM_KEYCLOAK_JWKS_REFRESH_SECS`,
`0` disables); Entra rotates its keys, so keep the refresh on. Only RS256 and ES256 are
accepted, whatever the token's header claims.

## Brokered through Keycloak

Configure Entra as an OpenID Connect identity provider in the Keycloak realm, map the Entra
claims (or groups) you need to realm roles `wpm-hr`, `wpm-payroll`, `wpm-admin`, and issue
tokens to the client from the realm. Point the API at the realm as in [auth.md](../auth.md).
Nothing in the API changes.

## What is tested

`tests/oidc_entra.rs` (run with `cargo test --no-default-features --features keycloak --test
oidc_entra -- --ignored`) starts an **Entra-compatible OIDC provider in the test process** and
walks the whole path:

1. fetch the provider's discovery document;
2. sign in two users with the **authorization-code flow with PKCE** against the provider's
   authorization and token endpoints (a code is single-use; a wrong verifier is refused);
3. receive an RS256 v2 access token with Entra's claims (`ver`, `tid`, `oid`, `roles`);
4. configure the service from the discovery document, with sign-in **left on by default**;
5. call the API: no token is `401`; a user with no app role can read but not write; a user
   with `wpm-hr` can write; a wrong audience, another tenant's issuer, an expired token, a
   token signed by another key and an HS256 algorithm-confusion token are each `401`; the
   audit actor is the `oid`.

Unit tests in `src/auth/keycloak.rs` pin the claim mapping. The test was **mutation-checked**:
making the API ignore Entra's `roles` claim fails it.

## What is not tested, and limits

- **The interactive browser redirect, consent and MFA**: not exercised; they belong to the
  broker. The mock provider is built from Entra's documented token shape, not captured from a
  live tenant. **Before relying on it, sign in once against your real tenant and decode the
  token** (any JWT viewer, on a non-production token) to confirm `iss`, `aud`, `ver`, `oid`
  and `roles`.
- **Groups**: Entra's `groups` claim carries object ids (and is replaced by a link when a
  user is in more than about 200 groups). The `department` attribute would then be a GUID.
  Use app roles for personas; scope data with the policy's other attributes.
- **`organization_ref`**: a custom attribute; Entra needs a directory schema extension and an
  optional claim to send it. Not covered here.
- **Conditional access, token lifetime, revocation**: Entra's. A token stays valid until it
  expires; WPM does not call back to Entra. Keep lifetimes short.
- **National clouds** use different hosts (for example a different login host); use that
  cloud's discovery document.

## Multi-factor authentication is required at the identity provider

WPM has no passwords and no accounts of its own: it verifies tokens issued by the identity provider (WPM-D62), so
**multi-factor authentication for every account type is a setting of the identity provider, and a deployment must
require it there.** For Microsoft Entra ID, that is a Conditional Access policy requiring MFA for the WPM application
for all users, with no exemption for HR, payroll, administrator or service accounts that can reach personal data. For
Keycloak, it is a required authentication flow with a second factor, or federation to a provider that enforces one. A
service peer (`svc`) presents a signed token and is not a person; protect its signing key instead.

This repository cannot test that the identity provider enforces MFA: its tests start a local OIDC provider and prove
the token verification and the persona mapping only. An assessor (for example question C3.5 of the NHS Digital
Technology Assessment Criteria, see [the assessment](../../united-kingdom/national-health-service/digital-technology-assessment-criteria/2.0/index.md))
will ask for the policy itself.

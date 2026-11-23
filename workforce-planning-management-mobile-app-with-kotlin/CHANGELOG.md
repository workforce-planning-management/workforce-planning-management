# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added — the first slice of the Android app (WPM-T639, 2026-11-23)

- `core`, a pure Kotlin module: the time-off and leave models, problem
  details, request building with `Accepts-version: 1.0`, the connection rules,
  PKCE `S256` and the sign-in callback check, and the leave rules mirrored from
  the service (cancellable, the inclusive day span, no reason for sick leave).
- `app`: connect, sign in through a Custom Tab with the `wpm-mobile://signin`
  redirect, my time off (with "Not recorded" for an unknown allowance and the
  service's derivation), request leave, cancel a request after confirming,
  sign out; the token encrypted with an Android Keystore key; backup off and
  `FLAG_SECURE`; a debug-only developer token field; light and dark through
  Lily's tokens; English strings.
- Lily Design System for Kotlin from a checkout, as a composite build
  (WPM-D226).

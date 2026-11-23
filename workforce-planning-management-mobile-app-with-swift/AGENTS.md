# AGENTS.md — the iPhone and iPad app

Stack rules for `workforce-planning-management-mobile-app-with-swift/`. The cross-cutting rules are in the root
[AGENTS.md](../AGENTS.md); the requirements are [spec/mobile-apps.md](../spec/mobile-apps.md). (Updated 2026-11-23,
through WPM-T638.)

## Rules

1. **The service decides.** The app checks shape only and shows the service's words for a refusal (`ProblemDetails`).
   A rule the screens show (cancellable, the day span, no reason for sick leave) mirrors the domain crate and is tested
   against the same table; when the crate changes, change the mirror and its test in the same commit.
2. **Pure core first (WPM-D225).** A model, a rule or a request goes in `WPMCore` with tests, then a screen in `WPMUI`.
   `WPMCore` imports Foundation and CryptoKit only: no SwiftUI, no `URLSession`, no `Date()` (the clock is passed in),
   no storage.
3. **An unknown is never a zero.** An optional figure stays optional to the screen and is shown as "Not recorded".
4. **Strings are content.** Every word is a key in `Sources/WPMUI/Resources/Localizable.xcstrings`, read with `L("key")`.
   `StringsTests` fails on a key the code uses that the catalogue lacks, and on a catalogue entry nothing uses.
5. **Keep nothing personal (WPM-D227).** Only the address (preferences) and the token (Keychain) are stored. Do not add
   a disk cache, an offline copy, a log of a response, or a third-party SDK (WPM-D228).
6. **Lily first, the platform's controls second.** Use `Field`, `ErrorSummary`, `SummaryList`, `Tag`, `InsetText`,
   `WarningText`; check Lily's public API in its `Sources` before using a component. Tap targets at least 44 points.
7. **Swift 6 language mode**, strict concurrency, no warnings.

## Verify

`swift test` (both test targets on macOS) and `make app` (the iOS Simulator build). Say in the task entry what was not
run: a physical device, VoiceOver, a large Dynamic Type size, and sign-in against a real authentication service.

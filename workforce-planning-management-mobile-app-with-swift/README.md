# Workforce Planning Management for iPhone and iPad

A native client of the Workforce Planning Management service for a worker on a phone: connect to a deployment, sign in,
see my time off, request leave, and cancel a request that may still be cancelled. Swift 6, SwiftUI, iOS 18 or later,
and Lily Design System for Swift (`LilyUI`, `LilyThemes`).

> ⚠️ **Demo software, AI-written and unreviewed.** Not reviewed for the App Store, not run on a physical device, and not
> signed in against the sibling authentication service. Synthetic data only.

The requirements and decisions are in [spec/mobile-apps.md](../spec/mobile-apps.md) (WPM-R280–R284, WPM-D224–D228);
the task is WPM-T638 in [spec/tasks.md](../spec/tasks.md).

## Layout

| Path | What it is |
| --- | --- |
| `Sources/WPMCore` | The pure core (WPM-D225): models, decoding, request building, connection rules, PKCE, leave rules. No network, clock, storage or UI. |
| `Sources/WPMUI` | The SwiftUI screens, the session, the Keychain token store and the `URLSession` transport. Strings in `Resources/Localizable.xcstrings`. |
| `Tests/WPMCoreTests` | Exhaustive tests of the core (swift-testing). |
| `Tests/WPMUITests` | Strings catalogue checks, the session and time-off model with a stub transport, a render of the time-off screen. |
| `App/` | The Xcode app project (`WorkforcePlanningManagement.xcodeproj`, bundle `org.workforceplanningmanagement.mobile`), a thin window over `WPMUI`. |

## Lily

Lily's Swift package is not yet in a public registry, so the package takes it from a checkout of the Lily monorepo
(WPM-D226). Set `LILY_DESIGN_SYSTEM_DIR` to the monorepo's directory; the default is
`../../lilydesignsystem/lily-design-system` beside this repository's root. Xcode reads the default unless it is started
with the variable set.

## Build and test

```sh
make test     # swift test: WPMCore and WPMUI on macOS, no simulator needed
make app      # xcodebuild for the iOS Simulator
```

Open `App/WorkforcePlanningManagement.xcodeproj` in Xcode to run on a simulator.

## Sign in

Sign-in opens the deployment's authentication service in the system browser (`ASWebAuthenticationSession`, ephemeral)
with an authorization code and PKCE, and returns to `wpm-mobile://signin`. The paths default to `/api/auth/authorize`
and `/api/auth/token` (`SignInConfiguration`). The redirect is caught by the session itself, so the app registers no
URL scheme with the system. A Debug build also offers **Use a development token**, for a token minted against a local
service (for example with `scripts/check-mcp-flow.mjs --print-env`'s throwaway key); a Release build has no such screen.

## What it keeps

The deployment's address (preferences) and the token (Keychain, `kSecAttrAccessibleWhenUnlockedThisDeviceOnly`).
Nothing else: answers are held in memory, the network session is ephemeral, and the window is covered when the app
leaves the foreground, so the app switcher shows no personal data (WPM-D227). No analytics or third-party SDK (WPM-D228).

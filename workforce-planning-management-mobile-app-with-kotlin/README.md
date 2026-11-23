# Workforce Planning Management for Android

A native Android app for the person: connect to your organization's Workforce
Planning Management, sign in in the browser, see your time off, request leave
and cancel a request. Kotlin, Jetpack Compose, Material 3 for the native
controls and [Lily Design System for Kotlin](https://github.com/LilyDesignSystem/lily-design-system)
for the components Lily has. The requirements are
[spec/mobile-apps.md](../spec/mobile-apps.md) (WPM-R280–R284); the task is
WPM-T639.

> ⚠️ **Demo software, AI-written and unreviewed.** Built and tested on the JVM
> (unit tests and Robolectric) as of 2026-11-23. It has **not** been run on an
> emulator or a device, signed in against the sibling authentication service,
> or reviewed for Google Play. See "Not verified" below.

## What it does

| Screen | Calls | Rules it shows (from `core`) |
| --- | --- | --- |
| Connect | nothing until accepted | `https`, or `http` only for `localhost`, `127.0.0.1` and `10.0.2.2` |
| Sign in | the system browser (Custom Tab), then `POST` the token path | PKCE `S256`, `state` checked, redirect `wpm-mobile://signin` |
| My time off | `GET /api/me/time-off?year=` | calendar days; an allowance not recorded is "Not recorded", never 0; the service's derivation; no sick-leave reason |
| Request leave | `POST /api/me/leave-requests` | the service's six kinds; no reason field for sick leave; the inclusive day span; at most 366 calendar days |
| Cancel | `POST /api/me/leave-requests/{pid}/cancel` | offered only for a request awaiting a decision or approved and not started; confirmed first |

Every request sends `Accepts-version: 1.0`, `Accept: application/json` and the
bearer token. A refusal shows the service's own `detail`. A `401` signs the
person out and says so.

## Layout

```
core/   pure Kotlin (JVM): models, JSON, request building, the connection,
        PKCE and sign-in, the leave rules. No Android, network, clock or UI
        (WPM-D225). Tested with JUnit.
app/    the Android app: screens (Compose), the view model, HttpURLConnection
        transport, the Keystore-encrypted token store.
        src/debug/   the developer token screen (debug builds only)
        src/release/ no such screen
```

## Build

Needs JDK 17 or later, an Android SDK with platform 37.2
(`sdkmanager "platforms;android-37.2"`; `sdk.dir` in `local.properties` or
`ANDROID_HOME`), and a checkout of the Lily Design System monorepo, because
Lily for Kotlin is not yet published (WPM-D226). The build looks for Lily at, in
order: `-Plily.dir=<checkout>`, `LILY_DESIGN_SYSTEM_DIR`, then
`../../lilydesignsystem/lily-design-system` beside this repository. It includes
`lily-design-system-kotlin` from there as a composite build.

```sh
./gradlew :core:test                 # the pure core
./gradlew :app:testDebugUnitTest     # the screens' semantics and the view model (Robolectric)
./gradlew :app:lintDebug
./gradlew :app:assembleDebug         # app/build/outputs/apk/debug/app-debug.apk
./gradlew :app:installDebug          # onto a running emulator or device
```

## Run against a local service

On the emulator the host machine is `10.0.2.2`, so a service on port 5150 is
`http://10.0.2.2:5150`. A debug build's sign-in screen has a developer field
that takes a pasted token, for example one minted with the throwaway key from
`node scripts/check-mcp-flow.mjs --print-env` (see the script); a release build
has no such field.

## Privacy

The app keeps the deployment's address (private preferences) and the token
(AES-GCM, with a key that never leaves the Android Keystore, in a file under
`noBackupFilesDir`), and nothing else (WPM-D227). Backup and device transfer are
off. The window is `FLAG_SECURE`, so no screen is in the recents snapshot or a
screenshot. There is no analytics or third-party SDK (WPM-D228).

## Not verified

- On an emulator or a device: sign-in through a Custom Tab, the redirect
  intent, the Keystore token store (Robolectric has no Android Keystore), the
  `HttpURLConnection` transport, TalkBack, and font scale 2.0.
- The sibling authentication service's native-client flow: its paths
  (`/api/auth/authorize`, `/api/auth/token`) and client id (`wpm-mobile`) are
  defaults in `SignInConfig`, not checked against it.
- A round trip against a running service.
- Translations: English only (WPM-T642).

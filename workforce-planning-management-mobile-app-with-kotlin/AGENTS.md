# AGENTS.md — the Android edition

Working agreements for this subproject. The repository's rules come first:
[../AGENTS.md](../AGENTS.md). (Updated 2026-11-23, through WPM-T639.)

## Specification Driven Development

**The `spec/` directory is the source of truth.**
- Before doing anything read all files inside the `spec/` directory using your file-reading tools.
- Treat all files inside `spec/` as strict requirements. Where any file disagrees with the spec, the spec wins: fix the file, or change the spec on purpose and bring the other files into line in the same commit.
- Do not improvise structure, style, or citation practice.

This app's requirements are [../spec/mobile-apps.md](../spec/mobile-apps.md).

## Stack

Kotlin 2.4 (AGP 9's built-in Kotlin in `app`, the Kotlin JVM plugin in `core`),
Jetpack Compose from the BOM, Material 3 for the native controls, Lily's
`lily-compose` and `lily-compose-themes` from a composite build (WPM-D226),
`kotlinx-serialization-json`, `androidx.browser` for the Custom Tab.
`minSdk` 24, `compileSdk` 37.2, JVM 17, `java.time` through core library
desugaring. Versions are in `gradle/libs.versions.toml` and match Lily's where
both use a library.

## Rules

- **Pure core first (WPM-D225).** A rule, a model or a request shape goes in
  `core/` with JUnit tests first; `core` depends on nothing Android, and the
  clock, the randomness and the transport are passed in. The screens arrange
  what `core` returns.
- **The service decides.** A rule in `core` mirrors one in the service so the
  screen offers only what the service would allow; name the service function
  it mirrors (`time_off::can_worker_cancel`) and test it case for case. Never
  decide something the service does not.
- **An unknown is never a zero.** A figure the service sends as `null` is
  `Figure.NotRecorded` and the string `not_recorded`.
- **Strings are content.** Every word is in `res/values/strings.xml`
  (plurals for counts). No literal text in a composable, except a number.
- **Lily first, then Material 3.** Use Lily's component where Lily has one
  (`SummaryList`, `Tag`, `Field`, `ErrorSummary`, `InsetText`, `WarningText`);
  Lily has no button, field, dialog or picker, so those are Material 3. Pass
  `Field`'s modifier to the control. Tap targets are at least
  `LilyMinimumTouchTarget` (48 dp).
- **Nothing personal on the phone (WPM-D227).** Only the address and the
  token are stored; the token only through `TokenStore`. No disk cache, no log
  of a request or a response, no queued write. Keep `allowBackup="false"` and
  `FLAG_SECURE`.
- **No third-party SDK (WPM-D228).** A new dependency needs a reason in the
  task entry.
- **Developer tools live in `src/debug/`** and have an empty twin in
  `src/release/`; nothing debug-only is guarded by a runtime flag.

## Verify

```sh
./gradlew :core:test :app:testDebugUnitTest :app:lintDebug :app:assembleDebug :app:compileReleaseKotlin
```

Report the test counts and lint result in the task entry, and say what was not
run (an emulator, a device, TalkBack). A Robolectric test clicks a node only
after `performScrollTo()`; a click on a node outside the viewport does nothing
(found 2026-11-23: two tests passed a click to an off-screen button).

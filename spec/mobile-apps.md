# Mobile apps: iPhone and Android (WPM-R280–R286, WPM-D224–D229)

Proposed 2026-11-23 at the owner's request: two native apps beside the web UI,
`workforce-planning-management-mobile-app-with-swift/` (Swift, SwiftUI, Lily
Design System for Swift) and `workforce-planning-management-mobile-app-with-kotlin/`
(Kotlin, Jetpack Compose, Lily Design System for Kotlin). The benchmark scan
listed "a mobile app or PWA with push" as a next candidate
([roadmap.md](roadmap.md)). Tasks: WPM-T638–T644 in [tasks.md](tasks.md).

> ⚠️ **AI-written and unreviewed; demo software.** Neither app has been through
> an App Store or Google Play review, run on a physical device, or signed in
> against the sibling authentication service. What was verified is listed in
> each task entry.

## Scope of the first slice

The apps are **clients of the existing API**, for the person, on a phone. The
first slice is the self-service a worker most often needs away from a desk:

1. connect to a deployment and sign in (WPM-R281);
2. see my time off (WPM-R282);
3. request leave, and cancel a request that may still be cancelled (WPM-R283).

Everything else (contact details, payslips, goals, the manager's and HR's
screens, push notifications) is **not in this slice**. It follows the same
pattern: a pure-core model and rule, then a screen.

## WPM-R280 — Two native apps over the one API

*As a worker I can use WPM from my phone in an app built for that phone.*

- Two subprojects, one per platform. Each is a client of
  `workforce-planning-management-service-with-rust` only: **no new endpoint,
  no new table and no data the web UI cannot already show** to the same
  caller. Every rule that decides (a balance, whether a request is allowed) is
  the service's; the app shows what the service says and checks shape only.
- Each request sends `Accepts-version: 1.0` (the service's version header) and
  `Accept: application/json`. A failure is read as the service's problem
  details (RFC 9457: `status`, `code`, `detail`), and the `detail` is shown to
  the person as the service wrote it.
- **iPhone and iPad:** iOS 18 or later, Swift 6 language mode, SwiftUI. Lily's
  `LilyUI` and `LilyThemes`. No third-party runtime dependency.
- **Android:** `minSdk` 24 and `compileSdk` 37.2 (what Lily's Compose modules
  require), Kotlin 2.4, Jetpack Compose with Material 3 for the native
  controls (Lily ships no button, field or navigation of its own), Lily's
  `lily-compose` and `lily-compose-themes`. The only network and JSON
  libraries are the platform's `HttpURLConnection` and
  `kotlinx-serialization-json`.

## WPM-R281 — Connect and sign in

*As a worker I connect the app to my organization's WPM and sign in the way
the web app does.*

- The person types the deployment's address (an `https` URL; `http` only for
  `localhost`, `127.0.0.1` and the Android emulator's host, `10.0.2.2`, so a
  developer can run the service on their own machine). Anything else is refused
  before any request.
- **Sign in** opens the deployment's authentication service in the **system
  browser** (`ASWebAuthenticationSession` on iOS, a Custom Tab on Android),
  never in an embedded web view, with an authorization code and PKCE
  (RFC 7636, `S256`) as RFC 8252 asks of a native app. The redirect is the
  app's own scheme, `wpm-mobile://signin`. The `state` is checked; a mismatch
  is refused.
- The token is held in the **Keychain** (iOS, `kSecAttrAccessibleWhenUnlockedThisDeviceOnly`)
  or in storage encrypted by an **Android Keystore** key, and nowhere else: not
  in preferences, a log, a crash report or a backup. **Sign out** deletes it.
  A `401` from the service signs the person out and says so.
- **Development builds only:** a screen accepts a token pasted by a developer
  (for example one minted by `scripts/check-mcp-flow.mjs --print-env`'s
  throwaway key against a local service). A release build has no such screen.
- **Not verified:** the sibling authentication service's support for a native
  public client (an authorization endpoint with PKCE and a custom-scheme
  redirect). Its endpoint paths are configuration
  (`/api/auth/authorize` and `/api/auth/token` by default), because this repo
  cannot check them ([auth.md](auth.md), WPM-D224).

## WPM-R282 — My time off

*As a worker I can see on my phone what leave I have, what I have asked for,
and what I have taken.*

- Reads `GET /api/me/time-off?year=` and shows, for the year: each kind's
  allowance, taken, booked, requested and remaining, the year's requests with
  their status, and past absence by year. **Every figure is in calendar days
  and says so.**
- A kind with **no allowance recorded** shows its remaining as *not recorded*,
  never `0` (an unknown is never a zero). The service's `derivation` text is
  shown on the screen, as the web app does.
- A sick-leave request shows no reason (the service sends none, WPM-D73).
- The person can move to the previous or the next year. Pull to refresh.

## WPM-R283 — Request leave and cancel a request

*As a worker I can ask for leave from my phone, and take back a request that
has not started.*

- **Request:** `POST /api/me/leave-requests` with a kind from the service's
  closed list (`annual`, `public_holiday`, `sick`, `parental`, `unpaid`,
  `other`), a start and an end date (the end may not be before the start), and
  an optional reason of up to 500 characters. **Sick leave has no reason
  field**, so none can be sent. The app shows the calendar days the dates span
  (inclusive, as the service counts) before it sends; the service still decides,
  and its refusal (a balance, a cap of 366 calendar days) is shown as it wrote it.
- **Cancel:** `POST /api/me/leave-requests/{pid}/cancel`. The cancel action is
  offered only where the service would allow it: a request awaiting a decision,
  or one approved that has not yet started (the rule of
  `time_off::can_worker_cancel`, mirrored and tested in each app's core). The
  person confirms first. The service remains the authority; a refusal is shown.
- A write is sent once per tap; a second tap while it is in flight does nothing.

## WPM-R284 — Accessible, localizable, themed

*As any worker I can use the app with VoiceOver or TalkBack, at a large text
size, in the light or the dark.*

- Built from Lily's components where Lily has one (`SummaryList`, `Tag`,
  `ErrorSummary`, `Field`, `InsetText`, `WarningText`) and the platform's own
  controls where it does not. Every control has a spoken name; status is never
  colour alone (a `Tag` carries the word); tap targets are at least 44 by 44
  points on iOS and 48 by 48 dp on Android; Dynamic Type and the font scale
  apply unchanged.
- Light or dark follows the system, through `LilyTokens.light` and
  `LilyTokens.dark`, the two themes held to WCAG AA ([themes-and-contrast.md](themes-and-contrast.md)).
- **Strings are content:** every word is in the platform's string resources
  (`Localizable.xcstrings`, `res/values/strings.xml`), none in code. The
  language follows the device. The first slice ships **English only**; the
  other locales of the web UI ([locales](locales-for-global-sharing-with-svelte/index.md))
  are a follow-up (WPM-T642), and a missing translation falls back to English,
  never to a key.
- Dates are shown in the device's locale and calendar; dates sent to the
  service are ISO 8601 calendar dates, computed without a time zone shift.

## WPM-R285 — Managed devices and personal devices, phones and tablets

*As an organization I can put the app on devices I manage, and as a worker I can use it on my own phone or tablet, with no
enrolment, on either.*

- **Form factors.** Each app runs on phones **and tablets**: the iOS app is one app for iPhone and iPad (a universal build,
  not an iPhone app scaled up); the Android app runs on phones and tablets of any size, in either orientation, in split screen
  and in a resizable window. The layout follows the window's width, not the device's name: a wide window shows the year's
  summary and its requests side by side, a narrow one stacks them, and the same screens, rules and strings serve both.
- **Managed devices.** The app can be deployed and configured by a mobile device management or enterprise mobility tool
  (for example Microsoft Intune, Jamf, Workspace ONE; Apple's managed distribution; Android Enterprise, fully managed or in a
  work profile). It reads two settings from the platform's **managed app configuration** (the iOS
  `com.apple.configuration.managed` defaults; the Android application restrictions): `deployment_url`, the deployment's
  address, so the worker is not asked to type it; and `deployment_locked` (`true` or `false`, default `false`), which, when
  `true`, hides the screen where the address is changed. A setting that is **supplied but invalid** (an address that WPM-R281
  refuses, a value other than `true` or `false`) is **refused with a message naming the setting**; it is not replaced by a
  default and the app does not fall back to asking for an address (RCA-IN-2, RCA-IN-4). An absent setting means the worker
  types the address as before.
- **Personal devices (bring your own device).** Nothing is required of the device: no enrolment, no management profile, no
  device administrator, no installed certificate. The app reads nothing outside its own sandbox, asks for no device
  identifier, contacts, photos, location or other permission, and uses no management API. It cannot see, and the organization
  cannot learn through it, anything about the personal side of the device.
- **Same behaviour either way.** The app does not detect whether the device is managed and shows the same screens, with the
  same data, to the same person. Managed configuration only fills in the address.
- **What leaves with the app.** The app keeps only the address and the token (WPM-D227). Removing the app, removing a work
  profile, or a selective wipe by a management tool removes them. On iOS a Keychain item can survive deleting the app, so a
  **first launch after an install deletes any token left from an earlier install** before it shows a screen.
- **Not in the app.** No device-compliance, jailbreak, root or hardware-attestation check, and no screenshot or copy
  restriction beyond WPM-D227's: whether a device may sign in is for the identity provider's conditional access, not for this
  app (WPM-D229).

## WPM-R286 — Record text is never copied onto the device

*As an organization I know that what the service shows about a person cannot be copied out of the app onto a personal device,
and as a worker I know the same of my own record.*

- **Record text** is every word the service returned for a screen: names, dates, figures, reasons, statuses, the
  `derivation` text, and an error's `detail`. It is shown, and it **cannot be copied out of the app**: no text selection, no
  copy, cut or share action on it, no drag and drop, no printing, no Handoff or universal clipboard, no data detectors that
  turn it into a link, and no pasteboard write by the app of anything the service sent. (iOS: `textSelection(.disabled)`, no
  `ShareLink` or `UIPasteboard`, no `NSUserActivity` or Spotlight donation of record text; Android: no `SelectionContainer`,
  no `ClipboardManager` write, no share intent, `setContentCapture` and autofill off for record text.)
- **What the person types** into the leave form (the dates and the reason) is not record text until the service returns it,
  but the reason field is a **no-learning** field: the keyboard does not add its words to a personal dictionary or suggestions
  (`autocorrectionDisabled`, no personalised learning on iOS; `IME_FLAG_NO_PERSONALIZED_LEARNING` and
  `TYPE_TEXT_FLAG_NO_SUGGESTIONS` on Android). The deployment address and the token are never shown for copying either.
- **Nowhere else on the device.** Record text is not in a notification, a widget, a Spotlight or app-search index, a share
  sheet preview, an app-switcher snapshot (WPM-D227), a log, a crash report or a pasteboard.
- **Every device alike.** The app cannot tell a personal device from a managed one and does not try (WPM-D229), so this holds
  on both; it is not a setting a management tool can loosen, and managed app configuration (WPM-R285) has no key for it.
- **What this cannot do.** A screenshot or a photograph of the screen, and a screen recording, cannot be stopped on iOS, and
  nothing here claims to. On Android the window is secure (`FLAG_SECURE`), which blocks screenshots and recents thumbnails;
  on iOS the content is hidden while the screen is being recorded or mirrored (`UIScreen.isCaptured`) and in the app switcher.
  An accessibility service, and VoiceOver or TalkBack reading the text aloud, are the person's own assistive tools and are
  not blocked. The rule is that **the app writes no record text anywhere a second app, a person or a sync service could read
  it from the device**; it is not a claim that the person cannot remember or photograph what they see.

## Decisions

### WPM-D224 — The apps call the service directly with a bearer token; no mobile BFF

**Decision.** Each app is an OAuth public client (RFC 8252) that receives the
same PASETO the authentication service mints for the web BFF, and sends it as
`Authorization: Bearer` to the service. The service is unchanged: it already
verifies offline and decides with the same ABAC policy and the `/api/me`
allow-list (WPM-D76).

**Why.** The web UI keeps the token on its server so no script can read it
(a cookie session and a same-origin proxy, [architecture.md](architecture.md) and [auth.md](auth.md)). A native app has
no such server, and a cookie inside an embedded web view would be worse: the
platform's secure store is the native equivalent of the BFF's server-side
session. A mobile BFF would be a second server to run for no gain in what the
token can do.

**What it costs.** The token is on the device, so the device's lock protects
it; a token's age and revocation are the service's existing checks
([auth.md](auth.md), WPM-R72). The authentication service must offer a
native-client flow, which this repo cannot verify.

### WPM-D225 — Pure core first, on the phone too

**Decision.** Each app has a UI-free core (`WPMCore`, a Swift package target;
`core`, a Kotlin JVM module) holding the models, the JSON decoding, the request
building, the connection rules (WPM-R281) and the rules the screens show
(cancellable, the day span, no reason for sick leave, unknown is not zero). It
has no network, clock, storage or UI dependency: the clock and the transport
are passed in. It is tested exhaustively on the build machine with no simulator
or emulator. The screens only arrange what the core returns.

**Why.** The same rule as the service (AGENTS.md rule 2): a rule tested where it
is cheap to test, then wired in.

### WPM-D226 — Lily from a checkout until Lily publishes the mobile packages

**Decision.** Lily's Swift package and Kotlin modules are early and not yet in a
public registry (as of 2026-11-23 the Swift export repository was not reachable
and the Kotlin modules are unreleased). Each app takes Lily from a local
checkout of the Lily monorepo, found through `LILY_DESIGN_SYSTEM_DIR` (an
environment variable; for Gradle, also the `lily.dir` property), defaulting to
`../../lilydesignsystem/lily-design-system` beside this repository. No
machine-specific path is committed. When Lily publishes, the dependency becomes
a version from the registry, and this decision is withdrawn.

**What it costs.** CI cannot build the apps until it checks Lily out too; the
CI jobs are WPM-T641.

### WPM-D227 — Nothing personal is kept on the phone

**Decision.** The apps keep the deployment's address and the token, and
nothing else. What the service returns is held in memory for the screen showing
it and is gone when the app is closed: no offline copy, no cache on disk, no
screenshot of a personal screen in the app switcher's snapshot where the
platform allows hiding it. A request that fails is not queued for later.

**Why.** WPM's rule is that what must not be stored gets no column
([privacy](../AGENTS/privacy-and-data-rules.md)); a phone is easier to lose than
a server, and an offline copy would be a second record to export, erase and
secure. An offline mode would be a decision of its own, with its own
requirement.

### WPM-D228 — No analytics, no third-party SDK

**Decision.** The apps send nothing to anyone but the deployment the person
chose: no analytics, crash reporting, advertising identifier or third-party
SDK. A crash is the platform's own report, which the person controls.

**Why.** A worker's employer should not learn from an app which screens a
person opened, and a third-party SDK would be a processor in the record of
processing ([regulatory.md](regulatory.md)).

### WPM-D229 — Device posture is the identity provider's to judge, not the app's

**Decision.** The apps do not collect or send any statement about the device (managed or not, rooted or jailbroken, its
model, its operating system version beyond what a request header carries by default, an attestation). A deployment that
wants to allow sign-in only from compliant or managed devices does it where the sign-in happens: conditional access in its
identity provider ([auth.md](auth.md), WPM-D222), which sees the device through the system browser and the management tool.
The managed app configuration (WPM-R285) fills in the address and nothing else.

**Why.** A device check inside the app is advice the device itself gives, and it would make the app a way for an employer
to look at a personal phone, which WPM-D227 and WPM-D228 rule out. The identity provider already does this properly and for
every application, and the same rule then covers the web UI.

**What it costs.** A deployment without conditional access cannot tell a managed device from a personal one. That is the
point of WPM-R285 for personal devices, and a gap for an organization that wanted otherwise; it is a decision of its own if
it comes.

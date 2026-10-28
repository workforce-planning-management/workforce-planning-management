# MHRA: what the source supports, and what it does not

Proposed 2026-10-28 (spec round WPM-T362). Requirement **WPM-R212**, decision **WPM-D137**. No code.

> ⚠️ **Demo software.** Synthetic data only. See [regulatory.md](regulatory.md).

## What was read

Only the gov.uk organisation page for the Medicines and Healthcare products Regulatory Agency
(<https://www.gov.uk/government/organisations/medicines-and-healthcare-products-regulatory-agency>), read 2026-10-28. It says the MHRA
"regulates medicines, medical devices and blood components for transfusion in the UK" and is an executive agency sponsored by the
Department of Health and Social Care. Its other content is links: device registration, marketing authorisations and licensing, the Yellow
Card reporting service, Drug Safety Update, alerts and recalls, and consultations.

**The page sets out no obligation on an employer and says nothing about staff**: no registration of people, no qualified or responsible
person, no inspection, no training duty. The linked guidance pages were **not** read. Any workforce rule about an MHRA-regulated
employer would therefore come from memory, and WPM-D136 says not to build on that.

## Decision

- **WPM-D137 Requirements for a regulator are drafted from the regulator's own guidance, not from its landing page.** An organisation
  page names what the body does; the duties sit in guidance. A requirement that rests on a duty the author recalls is marked
  *unverified* and has no task until the guidance is read.

## Requirement

- **WPM-R212 The regulatory register names the MHRA and what it covers.** [regulatory.md](regulatory.md) lists the MHRA as a regulator
  of medicines, medical devices and blood components in the UK, with the source above and the date read, and says in one line that
  WPM holds no licence, product, device or adverse-event data (as WPM-D75 holds no diagnosis). Nothing else is claimed.

## Open, and not a requirement yet

Each needs the linked guidance read first. None is built or planned.

1. Whether an MHRA-licensed employer must show named roles are filled and by whom (for example a person responsible for a licence). If
   so, a position could carry a *required role* flag and a vacancy there could be raised early, with the person recorded by `person:` ref only.
2. Whether training or competence records for such roles have a prescribed form.
3. Whether a safety notice or recall (Field Safety Notices, Drug Safety Update) is something WPM should receive. It is product
   information, not workforce information, and is probably out of scope.
4. Whether Yellow Card reports could ever involve a worker's health. If so, WPM-D73 applies: no diagnosis is stored.

## Task

- [x] WPM-T362 Done in regulatory.md (WPM-R212). Next: read the guidance for items 1 and 2 and return with sourced requirements.

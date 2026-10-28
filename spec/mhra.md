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

## Second reading (2026-10-28): two linked pages

- **Marketing authorisations and licensing** (<https://www.gov.uk/medicines-medical-devices-blood/marketing-authorisations-variations-licensing>)
  is an index with one summary line. It says nothing about named people, competence or notifying changes. The guidance that might
  (apply for a variation; change of ownership; PSURs; MHRA Portal registration) was **not read**.
- **Regulating medical devices in the UK** (<https://www.gov.uk/guidance/regulating-medical-devices-in-the-uk>) names one role: the
  **UK responsible person**, who must hold written authority from the manufacturer, whose name and address go on the labelling, who passes
  complaints to the manufacturer without delay, and who must tell the MHRA if they end the arrangement. It says nothing on staff
  competence or training. These are duties of a *role held by an organisation*, about devices, not an employee record.

Both readings were summaries by a fetch tool, not the pages verbatim; check the wording before building on them.

**What this changes:** open item 2 (competence records) has no support in either page. Item 1 has narrow support only for the device
side and only as a role, and it is not enough to justify a position flag; no requirement is added. Item 3 stays out of scope.

## Open, and not a requirement yet

Each needs the linked guidance read first. None is built or planned.

1. Whether an MHRA-licensed employer must show named roles are filled and by whom (for example a person responsible for a licence). If
   so, a position could carry a *required role* flag and a vacancy there could be raised early, with the person recorded by `person:` ref only.
2. Whether training or competence records for such roles have a prescribed form.
3. Whether a safety notice or recall (Field Safety Notices, Drug Safety Update) is something WPM should receive. It is product
   information, not workforce information, and is probably out of scope.
4. Whether Yellow Card reports could ever involve a worker's health. If so, WPM-D73 applies: no diagnosis is stored.

5. ISO 14971 (device risk management) is added to the register as a reference only (2026-10-28). Its text could not be read, so whether it
   asks anything about the competence of the people involved is unknown and no requirement follows. The same bar applies as to the MHRA:
   read the standard, or a source that quotes it, before drafting a duty.

## Task

- [x] WPM-T362 Done in regulatory.md (WPM-R212). Next: read the guidance for items 1 and 2 and return with sourced requirements.

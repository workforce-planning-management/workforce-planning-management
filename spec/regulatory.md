# Regulatory posture

> ⚠️ **Demo software.** Not a production HR/payroll system; no real
> personal data; statutory calculations are illustrative stubs.

## Observed by design

- **Data minimisation** — identities are URNs; WPM stores employment
  facts, not demographics; display names are refreshable caches.
- **Purpose limitation & consent** — the candidate pool is
  consent-bounded (`consent_until`); expired candidates leave search
  and are flagged for purge.
- **Access control** — ABAC personas + salary/review masking; the
  activation gate must be on before any real exposure.
- **Auditability** — mutations and sensitive reads audited (the
  substrate for GDPR accountability and payroll audit).
- **Synthetic data only** in seeds and tests.

## Newer data classes (WPM-T74–T89)

- **Emergency contacts** are other people's personal data: readable only by the
  person and HR, audited without detail, in the subject-access export, deleted
  on erasure (WPM-D30).
- A worker's **job level** is high-sensitivity career data (it tracks pay): the
  person and HR only, audited without the level, in the subject-access export,
  deleted on erasure (WPM-D40).
- **Directory** and **announcements** carry no sensitive field; being *away* is
  shown, never why (WPM-D29). **Read receipts** are the reader's data (exported,
  erased) and editors see only a count (WPM-D35).
- **Skill aspirations** are used only in the owner's own gap view and never in
  workforce roll-ups (WPM-D32).
- **Joiner/leaver records and handover actions** are in the subject-access
  export; erasure scrubs notes and unassigns tasks but **keeps the handover
  audit rows** (the trail of who gave what to whom is an accountability record,
  keyed to a pid that no longer identifies anyone).
- The retention sweep list (`rules::privacy::SOFT_DELETED_TABLES`, 55 tables)
  covers every soft-deleting table, new ones included.

## Production would additionally require

- UK GDPR / DPA 2018 lawful-basis mapping per record class;
  statutory **retention schedules** (payroll and right-to-work
  records carry multi-year duties; candidate data the opposite);
  subject-access and erasure flows coordinated with the identity
  services; jurisdiction-correct payroll/tax engines; equality-law
  review of any scoring (application screening, succession
  readiness); works-council/union consultation where applicable.
  Tracked as production gates in [tasks.md](tasks.md).

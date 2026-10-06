# Privacy and data rules

The design thread of this project (WPM-D17/D20/D24/D25, extended by
WPM-D29–D36): **what must not be stored gets no column; what must not be
disclosed gets no endpoint; every limit is stated in the payload.** These rules
bind every change. The sensitivity map is in [`spec/auth.md`](../spec/auth.md).

## Rules

1. **Unrepresentability.** No health cohort, symptom, diagnosis or pulse-author
   column; no aggregate over adjustment requests. Do not add one "because it
   would be handy".
2. **An unknown is never a zero.** An undeclared skill has no shortfall and no
   priority (`null` or a distinct `undeclared` status); a rate with nothing to
   divide by is `null`; the UI shows "—".
3. **Roll-ups name no one.** Workforce views return counts and sums only
   (skill gaps, training demand, headcount by department…). Private aspirations
   are never in a roll-up; they appear only in the owner's own view (the person,
   or HR who may write their record).
4. **Say "nobody" rather than guess.** Cover, on-call and handover views report
   an empty answer plainly.
5. **Say what an estimate rests on.** Hours are `courses`, `mixed` or `estimate`.
6. **Never expose *why*.** The directory shows a person is away, not the kind of
   leave. Notification bodies carry names and dates only.
7. **Third-party data is restricted.** Emergency contacts: the person and HR
   only; audit rows record *that* a contact changed, never its details.
8. **Counts, not watching.** Read receipts: the reader sees their own; editors
   see a number.
9. **Plain text and https only** for anything a user authors that other people
   see (announcements, links).

## Checklist for a new table or field that holds personal data

- [ ] **Soft-deleting?** Add it to `rules::privacy::SOFT_DELETED_TABLES`
      (sorted; update the length pin in its test) so the retention sweep covers it.
- [ ] **Person-keyed?** Add it to the **subject-access export** and the **erasure
      statements** in `src/controllers/privacy.rs`. Append erasure statements at
      the **end** — the audit snapshot reads their counts by position.
- [ ] **Audit trail kept?** If the rows are an accountability record (a
      handover), erasure scrubs notes and keeps the rows.
- [ ] **Authorization:** record-level (`authorize_record`) or organization scope
      (`scope_organization_refs`); out of scope is a **404**.
- [ ] **Audit entries** for mutations, without sensitive detail.
- [ ] **Spec:** domain model, the auth sensitivity map, audit actions,
      regulatory notes ([spec-driven-delivery](spec-driven-delivery.md)).
- [ ] **Test** that the thing is hidden from whom it must be hidden from, and
      that an export/erasure round-trip includes it.

## Never

- Real personal data in a fixture, example, screenshot or test. Everything is
  synthetic.
- Logging salary, review content, adjustment words, or contact details.
- A token in browser JavaScript or `localStorage`.

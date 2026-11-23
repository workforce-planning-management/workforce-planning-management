package io.github.workforceplanningmanagement.mobile

import io.github.workforceplanningmanagement.mobile.core.decodeTimeOff
import java.time.LocalDate

val TODAY: LocalDate = LocalDate.of(2026, 11, 23)

const val TIME_OFF_JSON = """{
  "unit": "calendar_days", "as_of": "2026-11-23", "year": 2026,
  "balances": [
    {"kind": "annual", "year": 2026, "entitled": 25, "used": 10, "taken": 8, "booked": 2, "requested": 3, "remaining": 15},
    {"kind": "unpaid", "year": 2026, "entitled": null, "used": 0, "taken": 0, "booked": 0, "requested": 2, "remaining": null}
  ],
  "requests": [
    {"pid": "past-approved", "kind": "annual", "start_on": "2026-08-03", "end_on": "2026-08-10", "days": 8, "status": "approved", "reason": "Family visit"},
    {"pid": "sick", "kind": "sick", "start_on": "2026-10-12", "end_on": "2026-10-13", "days": 2, "status": "approved", "reason": "should never show"},
    {"pid": "future-requested", "kind": "annual", "start_on": "2026-12-21", "end_on": "2026-12-23", "days": 3, "status": "requested"},
    {"pid": "future-approved", "kind": "annual", "start_on": "2026-12-28", "end_on": "2026-12-29", "days": 2, "status": "approved"},
    {"pid": "rejected", "kind": "other", "start_on": "2026-12-30", "end_on": "2026-12-30", "days": 1, "status": "rejected"}
  ],
  "history": [{"year": 2025, "absences": [{"kind": "annual", "absences": 3, "days": 21}]}],
  "derivation": "remaining = days allowed less the days already subtracted for approved requests."
}"""

val TIME_OFF = decodeTimeOff(TIME_OFF_JSON)

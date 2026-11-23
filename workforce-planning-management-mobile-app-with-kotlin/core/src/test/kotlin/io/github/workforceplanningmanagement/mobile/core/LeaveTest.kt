package io.github.workforceplanningmanagement.mobile.core

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test
import java.time.LocalDate

class LeaveTest {
    private fun d(m: Int, day: Int, y: Int = 2026) = LocalDate.of(y, m, day)
    private val today = d(6, 15)

    @Test fun cancelMirrorsTheService() {
        // time_off::can_worker_cancel, case for case.
        assertTrue("awaiting a decision, even if the dates passed", Leave.canCancel("requested", d(1, 5), today))
        assertTrue(Leave.canCancel("requested", d(7, 1), today))
        assertTrue("approved and not started", Leave.canCancel("approved", d(6, 16), today))
        assertFalse("approved and starting today", Leave.canCancel("approved", d(6, 15), today))
        assertFalse("approved and over", Leave.canCancel("approved", d(6, 1), today))
        assertFalse(Leave.canCancel("rejected", d(7, 1), today))
        assertFalse(Leave.canCancel("cancelled", d(7, 1), today))
        assertFalse(Leave.canCancel("", d(7, 1), today))
    }

    @Test fun theDaySpanIsInclusive() {
        assertEquals(1L, Leave.daySpan(d(6, 15), d(6, 15)))
        assertEquals(8L, Leave.daySpan(d(8, 3), d(8, 10)))
        assertEquals(366L, Leave.daySpan(d(1, 1, 2028), d(12, 31, 2028)))
        assertEquals(2L, Leave.daySpan(d(12, 31, 2026), d(1, 1, 2027)))
        assertEquals(29L, Leave.daySpan(d(2, 1, 2028), d(2, 29, 2028)))
        assertNull(Leave.daySpan(d(6, 16), d(6, 15)))
    }

    @Test fun sickLeaveHasNoReason() {
        assertFalse(Leave.reasonAllowed("sick"))
        Leave.KINDS.filter { it != "sick" }.forEach { assertTrue(it, Leave.reasonAllowed(it)) }
    }

    @Test fun theKindsAreTheServicesClosedList() {
        assertEquals(listOf("annual", "public_holiday", "sick", "parental", "unpaid", "other"), Leave.KINDS)
    }

    @Test fun anEmptyDraftNamesEveryMissingField() {
        assertEquals(
            listOf(DraftProblem.KindMissing, DraftProblem.StartMissing, DraftProblem.EndMissing),
            Leave.problems(LeaveDraft()),
        )
    }

    @Test fun draftProblems() {
        val ok = LeaveDraft("annual", d(8, 3), d(8, 10))
        assertEquals(emptyList<DraftProblem>(), Leave.problems(ok))
        assertEquals(listOf(DraftProblem.KindUnknown), Leave.problems(ok.copy(kind = "holiday")))
        assertEquals(listOf(DraftProblem.EndBeforeStart), Leave.problems(ok.copy(endOn = d(8, 2))))
        assertEquals(listOf(DraftProblem.TooLong), Leave.problems(ok.copy(startOn = d(1, 1), endOn = d(1, 2, 2027))))
        assertEquals(emptyList<DraftProblem>(), Leave.problems(ok.copy(startOn = d(1, 1, 2028), endOn = d(12, 31, 2028))))
        assertEquals(listOf(DraftProblem.ReasonTooLong), Leave.problems(ok.copy(reason = "x".repeat(501))))
        assertEquals(emptyList<DraftProblem>(), Leave.problems(ok.copy(reason = " " + "x".repeat(500) + " ")))
        assertEquals("a sick-leave reason is ignored, never refused",
            emptyList<DraftProblem>(), Leave.problems(ok.copy(kind = "sick", reason = "x".repeat(600))))
        assertEquals(listOf(DraftProblem.EndMissing), Leave.problems(ok.copy(endOn = null)))
    }

    @Test fun thePayload() {
        assertNull(Leave.payload(LeaveDraft()))
        assertEquals(LeaveRequestPayload("annual", d(8, 3), d(8, 10), "Family visit"),
            Leave.payload(LeaveDraft("annual", d(8, 3), d(8, 10), "  Family visit ")))
        assertEquals(LeaveRequestPayload("annual", d(8, 3), d(8, 10), null),
            Leave.payload(LeaveDraft("annual", d(8, 3), d(8, 10), "   ")))
        assertEquals("sick leave never sends a reason", LeaveRequestPayload("sick", d(8, 3), d(8, 4), null),
            Leave.payload(LeaveDraft("sick", d(8, 3), d(8, 4), "flu")))
    }

    @Test fun thePayloadIsTheServicesJson() {
        assertEquals("""{"kind":"annual","start_on":"2026-08-03","end_on":"2026-08-10","reason":"Trip"}""",
            Leave.encode(LeaveRequestPayload("annual", d(8, 3), d(8, 10), "Trip")))
        assertEquals("""{"kind":"sick","start_on":"2026-08-03","end_on":"2026-08-04"}""",
            Leave.encode(LeaveRequestPayload("sick", d(8, 3), d(8, 4))))
    }
}

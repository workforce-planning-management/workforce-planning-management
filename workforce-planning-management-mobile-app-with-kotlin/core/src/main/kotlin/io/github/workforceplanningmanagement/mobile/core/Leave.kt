// Requesting and cancelling leave (WPM-R283). These mirror the service's rules so the screen offers only what the service
// would allow; the service still decides, and its refusal is shown as it wrote it.
@file:UseSerializers(LocalDateSerializer::class)

package io.github.workforceplanningmanagement.mobile.core

import kotlinx.serialization.Serializable
import kotlinx.serialization.UseSerializers
import java.time.LocalDate
import java.time.temporal.ChronoUnit

/** Why a draft cannot be sent. */
public enum class DraftProblem { KindMissing, KindUnknown, StartMissing, EndMissing, EndBeforeStart, TooLong, ReasonTooLong }

/** What the person has filled in so far. */
public data class LeaveDraft(
    val kind: String? = null,
    val startOn: LocalDate? = null,
    val endOn: LocalDate? = null,
    val reason: String = "",
)

/** The body of `POST /api/me/leave-requests`. */
@Serializable
public data class LeaveRequestPayload(
    val kind: String,
    @kotlinx.serialization.SerialName("start_on") val startOn: LocalDate,
    @kotlinx.serialization.SerialName("end_on") val endOn: LocalDate,
    val reason: String? = null,
)

/** The leave rules. */
public object Leave {
    /** The service's closed list of kinds (`tokens::LEAVE_KINDS`), in its order. */
    public val KINDS: List<String> = listOf("annual", "public_holiday", "sick", "parental", "unpaid", "other")

    /** The longest request, in calendar days (`leave::day_span`). */
    public const val MAX_DAYS: Int = 366

    /** The longest reason, in characters. */
    public const val REASON_MAX: Int = 500

    /** Whether a reason may be given for [kind]: never for sick leave (WPM-D73), so the screen has no field for it. */
    public fun reasonAllowed(kind: String): Boolean = kind != "sick"

    /** The calendar days from [start] to [end] inclusive, as the service counts; null when the end is before the start. */
    public fun daySpan(start: LocalDate, end: LocalDate): Long? =
        if (end < start) null else ChronoUnit.DAYS.between(start, end) + 1

    /**
     * Whether the person may cancel a request ([time_off::can_worker_cancel]): one awaiting a decision, even if its dates
     * have passed, or one approved that starts after [today]. Nothing else.
     */
    public fun canCancel(status: String, startOn: LocalDate, today: LocalDate): Boolean = when (status) {
        "requested" -> true
        "approved" -> startOn > today
        else -> false
    }

    /** The problems with [draft], in the order of the fields; empty when it can be sent. */
    public fun problems(draft: LeaveDraft): List<DraftProblem> = buildList {
        when (val kind = draft.kind) {
            null -> add(DraftProblem.KindMissing)
            !in KINDS -> add(DraftProblem.KindUnknown)
            else -> if (reasonAllowed(kind) && draft.reason.trim().length > REASON_MAX) add(DraftProblem.ReasonTooLong)
        }
        val start = draft.startOn
        val end = draft.endOn
        if (start == null) add(DraftProblem.StartMissing)
        if (end == null) add(DraftProblem.EndMissing)
        if (start != null && end != null) {
            val span = daySpan(start, end)
            if (span == null) add(DraftProblem.EndBeforeStart) else if (span > MAX_DAYS) add(DraftProblem.TooLong)
        }
    }.sortedBy { it.ordinal }

    /** The payload for [draft], or null while it has problems. A sick-leave payload never carries a reason. */
    public fun payload(draft: LeaveDraft): LeaveRequestPayload? {
        if (problems(draft).isNotEmpty()) return null
        val kind = draft.kind!!
        val reason = draft.reason.trim().takeIf { reasonAllowed(kind) && it.isNotEmpty() }
        return LeaveRequestPayload(kind, draft.startOn!!, draft.endOn!!, reason)
    }

    internal fun encode(payload: LeaveRequestPayload): String = Json.encodeToString(LeaveRequestPayload.serializer(), payload)
}

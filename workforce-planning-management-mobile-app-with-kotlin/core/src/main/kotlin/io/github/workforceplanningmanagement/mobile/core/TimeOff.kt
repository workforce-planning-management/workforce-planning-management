// GET /api/me/time-off (WPM-R282). Every figure is in calendar days; an allowance or remaining the service could not give is
// null here and "not recorded" on the screen, never 0.
@file:UseSerializers(LocalDateSerializer::class)

package io.github.workforceplanningmanagement.mobile.core

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlinx.serialization.UseSerializers
import java.time.LocalDate

/** The caller's time off for one year. */
@Serializable
public data class TimeOff(
    val unit: String,
    @SerialName("as_of") val asOf: LocalDate,
    val year: Int,
    val balances: List<Balance> = emptyList(),
    val requests: List<LeaveRequest> = emptyList(),
    val history: List<HistoryYear> = emptyList(),
    val derivation: String = "",
)

/** One kind's figures for the year. [entitled] and [remaining] are null when no allowance is recorded. */
@Serializable
public data class Balance(
    val kind: String,
    val year: Int,
    val entitled: Int? = null,
    val used: Int = 0,
    val taken: Int = 0,
    val booked: Int = 0,
    val requested: Int = 0,
    val remaining: Int? = null,
    @SerialName("remaining_if_requested_approved") val remainingIfRequestedApproved: Int? = null,
)

/** One request. A sick-leave request carries no reason. */
@Serializable
public data class LeaveRequest(
    val pid: String,
    val kind: String,
    @SerialName("start_on") val startOn: LocalDate,
    @SerialName("end_on") val endOn: LocalDate,
    val days: Int,
    val status: String,
    val reason: String? = null,
)

/** Past absence in one year. */
@Serializable
public data class HistoryYear(val year: Int, val absences: List<Absence> = emptyList())

/** Approved and over: how many absences of a kind, and their calendar days. */
@Serializable
public data class Absence(val kind: String, val absences: Int, val days: Int)

/** A figure that is known, or was never recorded. */
public sealed interface Figure {
    /** Calendar days. */
    public data class Days(val days: Int) : Figure

    /** No allowance is recorded: unknown, not zero. */
    public data object NotRecorded : Figure
}

/** The remaining figure of [balance]. */
public fun Balance.remainingFigure(): Figure = remaining?.let { Figure.Days(it) } ?: Figure.NotRecorded

/** The allowance of [balance]. */
public fun Balance.entitledFigure(): Figure = entitled?.let { Figure.Days(it) } ?: Figure.NotRecorded

/** Reads a time-off response body. */
public fun decodeTimeOff(body: String): TimeOff = Json.decodeFromString(TimeOff.serializer(), body)

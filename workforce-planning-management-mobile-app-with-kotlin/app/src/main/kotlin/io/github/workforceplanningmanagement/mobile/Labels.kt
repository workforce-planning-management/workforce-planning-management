// The service's tokens and the core's outcomes in the app's words (WPM-R284). An unknown token from a newer service is shown
// as the service sent it rather than hidden.
package io.github.workforceplanningmanagement.mobile

import androidx.compose.runtime.Composable
import androidx.compose.ui.res.pluralStringResource
import androidx.compose.ui.res.stringResource
import io.github.workforceplanningmanagement.mobile.core.AddressProblem
import io.github.workforceplanningmanagement.mobile.core.DraftProblem
import io.github.workforceplanningmanagement.mobile.core.Figure
import java.time.LocalDate
import java.time.format.DateTimeFormatter
import java.time.format.FormatStyle

@Composable
fun kindLabel(kind: String): String = when (kind) {
    "annual" -> stringResource(R.string.kind_annual)
    "public_holiday" -> stringResource(R.string.kind_public_holiday)
    "sick" -> stringResource(R.string.kind_sick)
    "parental" -> stringResource(R.string.kind_parental)
    "unpaid" -> stringResource(R.string.kind_unpaid)
    "other" -> stringResource(R.string.kind_other)
    else -> kind
}

@Composable
fun statusLabel(status: String): String = when (status) {
    "requested" -> stringResource(R.string.status_requested)
    "approved" -> stringResource(R.string.status_approved)
    "rejected" -> stringResource(R.string.status_rejected)
    "cancelled" -> stringResource(R.string.status_cancelled)
    else -> status
}

@Composable
fun days(n: Int): String = pluralStringResource(R.plurals.calendar_days, n, n)

@Composable
fun figure(f: Figure): String = when (f) {
    is Figure.Days -> days(f.days)
    Figure.NotRecorded -> stringResource(R.string.not_recorded)
}

/** A calendar date in the device's locale. */
fun date(d: LocalDate): String = d.format(DateTimeFormatter.ofLocalizedDate(FormatStyle.MEDIUM))

@Composable
fun addressProblem(p: AddressProblem): String = stringResource(
    when (p) {
        AddressProblem.Empty -> R.string.connect_problem_empty
        AddressProblem.NotAnAddress -> R.string.connect_problem_not_an_address
        AddressProblem.InsecureScheme -> R.string.connect_problem_insecure
        AddressProblem.HasCredentials -> R.string.connect_problem_credentials
        AddressProblem.HasQueryOrFragment -> R.string.connect_problem_query
    },
)

@Composable
fun draftProblem(p: DraftProblem): String = stringResource(
    when (p) {
        DraftProblem.KindMissing -> R.string.draft_kind_missing
        DraftProblem.KindUnknown -> R.string.draft_kind_unknown
        DraftProblem.StartMissing -> R.string.draft_start_missing
        DraftProblem.EndMissing -> R.string.draft_end_missing
        DraftProblem.EndBeforeStart -> R.string.draft_end_before_start
        DraftProblem.TooLong -> R.string.draft_too_long
        DraftProblem.ReasonTooLong -> R.string.draft_reason_too_long
    },
)

@Composable
fun failure(f: Failure): String = when (f) {
    Failure.Unreachable -> stringResource(R.string.error_unreachable)
    Failure.Unreadable -> stringResource(R.string.error_unreadable)
    is Failure.Refused -> f.detail ?: stringResource(R.string.error_refused, f.status)
}

@Composable
fun notice(n: Notice): String = stringResource(
    when (n) {
        Notice.SignInFailed -> R.string.signin_failed
        Notice.StateMismatch -> R.string.signin_state_mismatch
        Notice.SignInDenied -> R.string.signin_denied
        Notice.NoBrowser -> R.string.signin_no_browser
        Notice.SignedOutExpired -> R.string.signed_out_expired
        Notice.LeaveSent -> R.string.leave_sent
        Notice.RequestCancelled -> R.string.request_cancelled
    },
)

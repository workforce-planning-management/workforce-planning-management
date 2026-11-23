// The screens. Each takes its state and callbacks, so it is tested without the view model (Robolectric). Lily draws the
// components Lily has; Material 3 draws the native controls (WPM-R284).
@file:OptIn(ExperimentalMaterial3Api::class)

package io.github.workforceplanningmanagement.mobile

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.selection.selectable
import androidx.compose.foundation.selection.selectableGroup
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.DatePicker
import androidx.compose.material3.DatePickerDialog
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.RadioButton
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.pulltorefresh.PullToRefreshBox
import androidx.compose.material3.rememberDatePickerState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.pluralStringResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.clearAndSetSemantics
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.heading
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.unit.dp
import io.github.lilydesignsystem.compose.ErrorSummary
import io.github.lilydesignsystem.compose.Field
import io.github.lilydesignsystem.compose.InsetText
import io.github.lilydesignsystem.compose.LilyMinimumTouchTarget
import io.github.lilydesignsystem.compose.Problem
import io.github.lilydesignsystem.compose.SummaryList
import io.github.lilydesignsystem.compose.SummaryListItem
import io.github.lilydesignsystem.compose.Tag
import io.github.lilydesignsystem.compose.WarningText
import io.github.workforceplanningmanagement.mobile.core.AddressProblem
import io.github.workforceplanningmanagement.mobile.core.Leave
import io.github.workforceplanningmanagement.mobile.core.ManagedProblem
import io.github.workforceplanningmanagement.mobile.core.LeaveDraft
import io.github.workforceplanningmanagement.mobile.core.LeaveRequest
import io.github.workforceplanningmanagement.mobile.core.TimeOff
import io.github.workforceplanningmanagement.mobile.core.entitledFigure
import io.github.workforceplanningmanagement.mobile.core.remainingFigure
import java.time.LocalDate

private val Gap = 16.dp
private val Touch = Modifier.heightIn(min = LilyMinimumTouchTarget)

@Composable
private fun Heading(text: String, modifier: Modifier = Modifier, large: Boolean = false) {
    Text(
        text,
        style = if (large) MaterialTheme.typography.headlineSmall else MaterialTheme.typography.titleMedium,
        modifier = modifier.semantics { heading() },
    )
}

@Composable
private fun NoticeText(text: String) {
    InsetText { Text(text, style = MaterialTheme.typography.bodyMedium) }
}

@Composable
fun ConnectScreen(initial: String, problem: AddressProblem?, onContinue: (String) -> Unit, modifier: Modifier = Modifier) {
    var address by rememberSaveable { mutableStateOf(initial) }
    Column(modifier.verticalScroll(rememberScrollState()).padding(Gap), verticalArrangement = Arrangement.spacedBy(Gap)) {
        Heading(stringResource(R.string.connect_title), large = true)
        Field(
            label = stringResource(R.string.connect_address),
            description = stringResource(R.string.connect_address_help),
            error = problem?.let { addressProblem(it) },
            required = true,
            requiredLabel = stringResource(R.string.required),
        ) { fieldModifier ->
            OutlinedTextField(
                value = address,
                onValueChange = { address = it },
                singleLine = true,
                modifier = fieldModifier.fillMaxWidth().testTag("address"),
            )
        }
        Button(onClick = { onContinue(address) }, modifier = Touch) { Text(stringResource(R.string.connect_continue)) }
    }
}

@Composable
fun SignInScreen(
    baseUrl: String,
    notice: Notice?,
    onSignIn: () -> Unit,
    onChangeAddress: (() -> Unit)?,
    developerToken: (@Composable (onToken: (String) -> Unit) -> Unit)?,
    onToken: (String) -> Unit,
    modifier: Modifier = Modifier,
) {
    Column(modifier.verticalScroll(rememberScrollState()).padding(Gap), verticalArrangement = Arrangement.spacedBy(Gap)) {
        Heading(stringResource(R.string.signin_title), large = true)
        notice?.let { NoticeText(notice(it)) }
        Text(stringResource(R.string.signin_connected_to, baseUrl))
        Text(stringResource(R.string.signin_body))
        Button(onClick = onSignIn, modifier = Touch) { Text(stringResource(R.string.signin_button)) }
        if (onChangeAddress != null) {
            OutlinedButton(onClick = onChangeAddress, modifier = Touch) { Text(stringResource(R.string.signin_change_address)) }
        }
        developerToken?.invoke(onToken)
    }
}

@Composable
fun TimeOffScreen(
    state: UiState,
    today: LocalDate,
    onPreviousYear: () -> Unit,
    onNextYear: () -> Unit,
    onRefresh: () -> Unit,
    onRequestLeave: () -> Unit,
    onCancel: (String) -> Unit,
    onSignOut: () -> Unit,
    modifier: Modifier = Modifier,
) {
    var confirming by remember { mutableStateOf<LeaveRequest?>(null) }
    PullToRefreshBox(isRefreshing = state.loading, onRefresh = onRefresh, modifier = modifier) {
        LazyColumn(
            modifier = Modifier.fillMaxSize().testTag("time-off"),
            contentPadding = PaddingValues(Gap),
            verticalArrangement = Arrangement.spacedBy(Gap),
        ) {
            item {
                Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    Heading(stringResource(R.string.time_off_title), Modifier.weight(1f), large = true)
                    TextButton(onClick = onSignOut, modifier = Touch) { Text(stringResource(R.string.sign_out)) }
                }
            }
            item {
                Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    val prev = stringResource(R.string.time_off_previous_year, state.year - 1)
                    val next = stringResource(R.string.time_off_next_year, state.year + 1)
                    // The arrows are visual; each button's spoken name is the whole phrase.
                    OutlinedButton(onClick = onPreviousYear, modifier = Touch.testTag("previous-year").semantics { contentDescription = prev }) {
                        Text("‹ ${state.year - 1}", modifier = Modifier.clearAndSetSemantics { })
                    }
                    Heading(stringResource(R.string.time_off_year, state.year), Modifier.weight(1f))
                    OutlinedButton(onClick = onNextYear, modifier = Touch.testTag("next-year").semantics { contentDescription = next }) {
                        Text("${state.year + 1} ›", modifier = Modifier.clearAndSetSemantics { })
                    }
                }
            }
            item {
                Button(onClick = onRequestLeave, modifier = Touch.testTag("request-leave")) {
                    Text(stringResource(R.string.request_leave))
                }
            }
            state.notice?.let { n -> item { NoticeText(notice(n)) } }
            state.loadFailure?.let { f ->
                item { WarningText(prefix = stringResource(R.string.problem_title), text = failure(f)) }
            }
            if (state.timeOff == null && state.loading) {
                item { Text(stringResource(R.string.loading)) }
            }
            state.timeOff?.let { t -> timeOffItems(t, today) { confirming = it } }
            item {
                OutlinedButton(onClick = onRefresh, modifier = Touch) { Text(stringResource(R.string.time_off_refresh)) }
            }
        }
    }
    confirming?.let { r ->
        AlertDialog(
            onDismissRequest = { confirming = null },
            title = { Text(stringResource(R.string.request_cancel_confirm_title)) },
            text = {
                Text(stringResource(R.string.request_cancel_confirm_body, kindLabel(r.kind),
                    stringResource(R.string.request_dates, date(r.startOn), date(r.endOn))))
            },
            confirmButton = {
                TextButton(onClick = { confirming = null; onCancel(r.pid) }, enabled = !state.sending) {
                    Text(stringResource(R.string.request_cancel_yes))
                }
            },
            dismissButton = { TextButton(onClick = { confirming = null }) { Text(stringResource(R.string.request_cancel_no)) } },
        )
    }
}

private fun androidx.compose.foundation.lazy.LazyListScope.timeOffItems(
    t: TimeOff,
    today: LocalDate,
    onCancel: (LeaveRequest) -> Unit,
) {
    item { Text(stringResource(R.string.time_off_unit), style = MaterialTheme.typography.bodyMedium) }
    item { Text(stringResource(R.string.time_off_as_of, date(t.asOf)), style = MaterialTheme.typography.bodySmall) }

    item { Heading(stringResource(R.string.time_off_balances)) }
    if (t.balances.isEmpty()) item { Text(stringResource(R.string.time_off_no_balances)) }
    t.balances.forEach { b ->
        item(key = "balance-${b.kind}") {
            Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                Heading(kindLabel(b.kind))
                SummaryList(label = kindLabel(b.kind), modifier = Modifier.testTag("balance-${b.kind}")) {
                    SummaryListItem(stringResource(R.string.balance_entitled)) { Text(figure(b.entitledFigure())) }
                    SummaryListItem(stringResource(R.string.balance_taken)) { Text(days(b.taken)) }
                    SummaryListItem(stringResource(R.string.balance_booked)) { Text(days(b.booked)) }
                    SummaryListItem(stringResource(R.string.balance_requested)) { Text(days(b.requested)) }
                    SummaryListItem(stringResource(R.string.balance_remaining)) { Text(figure(b.remainingFigure())) }
                }
                HorizontalDivider()
            }
        }
    }

    item { Heading(stringResource(R.string.time_off_requests)) }
    if (t.requests.isEmpty()) item { Text(stringResource(R.string.time_off_no_requests)) }
    t.requests.forEach { r ->
        item(key = "request-${r.pid}") {
            Column(verticalArrangement = Arrangement.spacedBy(4.dp), modifier = Modifier.testTag("request-${r.pid}")) {
                Text(kindLabel(r.kind), style = MaterialTheme.typography.titleSmall)
                Text(stringResource(R.string.request_dates, date(r.startOn), date(r.endOn)))
                Text(days(r.days))
                Tag(text = statusLabel(r.status), label = stringResource(R.string.request_status_label))
                // The service sends no reason for sick leave (WPM-D73); nothing is shown for it.
                r.reason?.takeIf { Leave.reasonAllowed(r.kind) && it.isNotBlank() }?.let {
                    Text(stringResource(R.string.request_reason, it))
                }
                if (Leave.canCancel(r.status, r.startOn, today)) {
                    OutlinedButton(onClick = { onCancel(r) }, modifier = Touch.testTag("cancel-${r.pid}")) {
                        Text(stringResource(R.string.request_cancel))
                    }
                }
            }
        }
    }

    item { Heading(stringResource(R.string.time_off_history)) }
    if (t.history.isEmpty()) item { Text(stringResource(R.string.time_off_no_history)) }
    t.history.forEach { y ->
        item(key = "history-${y.year}") {
            SummaryList(label = y.year.toString()) {
                y.absences.forEach { a ->
                    SummaryListItem("${y.year} · ${kindLabel(a.kind)}") {
                        Text(pluralStringResource(R.plurals.absences, a.absences, a.absences, days(a.days)))
                    }
                }
            }
        }
    }

    if (t.derivation.isNotBlank()) {
        item { Heading(stringResource(R.string.time_off_how)) }
        item { InsetText { Text(t.derivation, style = MaterialTheme.typography.bodySmall) } }
    }
}

@Composable
fun RequestLeaveScreen(
    draft: LeaveDraft,
    sending: Boolean,
    failure: Failure?,
    onChange: (LeaveDraft) -> Unit,
    onSend: () -> Unit,
    onBack: () -> Unit,
    modifier: Modifier = Modifier,
) {
    var tried by rememberSaveable { mutableStateOf(false) }
    var picking by rememberSaveable { mutableStateOf<String?>(null) }
    val problems = if (tried) Leave.problems(draft) else emptyList()
    Column(modifier.verticalScroll(rememberScrollState()).padding(Gap), verticalArrangement = Arrangement.spacedBy(Gap)) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            TextButton(onClick = onBack, modifier = Touch) { Text(stringResource(R.string.leave_back)) }
        }
        Heading(stringResource(R.string.request_leave_title), large = true)
        if (problems.isNotEmpty()) {
            ErrorSummary(title = stringResource(R.string.problem_title), problems = problems.map { Problem(draftProblem(it)) {} })
        }
        failure?.let { WarningText(prefix = stringResource(R.string.problem_title), text = failure(it)) }

        Heading(stringResource(R.string.leave_kind))
        Column(Modifier.selectableGroup()) {
            Leave.KINDS.forEach { kind ->
                Row(
                    verticalAlignment = Alignment.CenterVertically,
                    modifier = Touch.fillMaxWidth().testTag("kind-$kind").selectable(
                        selected = draft.kind == kind,
                        onClick = { onChange(draft.copy(kind = kind)) },
                        role = Role.RadioButton,
                    ),
                ) {
                    RadioButton(selected = draft.kind == kind, onClick = null)
                    Text(kindLabel(kind), modifier = Modifier.padding(start = 8.dp))
                }
            }
        }

        DateField(R.string.leave_start, draft.startOn, "start") { picking = "start" }
        DateField(R.string.leave_end, draft.endOn, "end") { picking = "end" }

        val start = draft.startOn
        val end = draft.endOn
        if (start != null && end != null) {
            Leave.daySpan(start, end)?.let { span ->
                Text(stringResource(R.string.leave_span, days(span.toInt())), modifier = Modifier.testTag("span"))
            }
        }

        val kind = draft.kind
        if (kind != null && !Leave.reasonAllowed(kind)) {
            InsetText { Text(stringResource(R.string.leave_sick_no_reason)) }
        } else {
            Field(
                label = stringResource(R.string.leave_reason),
                description = stringResource(R.string.leave_reason_help),
            ) { fieldModifier ->
                OutlinedTextField(
                    value = draft.reason,
                    onValueChange = { onChange(draft.copy(reason = it)) },
                    modifier = fieldModifier.fillMaxWidth().testTag("reason"),
                    minLines = 2,
                )
            }
        }

        Button(
            onClick = {
                tried = true
                if (Leave.problems(draft).isEmpty()) onSend()
            },
            enabled = !sending,
            modifier = Touch.testTag("send"),
        ) { Text(stringResource(R.string.leave_send)) }
    }

    picking?.let { which ->
        val initial = if (which == "start") draft.startOn else draft.endOn ?: draft.startOn
        val pickerState = rememberDatePickerState(initialSelectedDateMillis = initial?.let { it.toEpochDay() * DAY_MILLIS })
        DatePickerDialog(
            onDismissRequest = { picking = null },
            confirmButton = {
                TextButton(onClick = {
                    // The picker's millis are UTC midnight: a calendar date, with no time zone shift.
                    pickerState.selectedDateMillis?.let { ms ->
                        val day = LocalDate.ofEpochDay(Math.floorDiv(ms, DAY_MILLIS))
                        onChange(if (which == "start") draft.copy(startOn = day) else draft.copy(endOn = day))
                    }
                    picking = null
                }) { Text(stringResource(R.string.date_ok)) }
            },
            dismissButton = { TextButton(onClick = { picking = null }) { Text(stringResource(R.string.date_cancel)) } },
        ) { DatePicker(state = pickerState) }
    }
}

private const val DAY_MILLIS = 86_400_000L

@Composable
private fun DateField(label: Int, value: LocalDate?, tag: String, onPick: () -> Unit) {
    Field(label = stringResource(label), required = true, requiredLabel = stringResource(R.string.required)) { fieldModifier ->
        OutlinedButton(onClick = onPick, modifier = fieldModifier.then(Touch).fillMaxWidth().testTag("date-$tag")) {
            Text(value?.let { date(it) } ?: stringResource(R.string.leave_choose_date))
        }
    }
}

/** A setting a management tool supplied is not valid (WPM-R285): named, with no default and no address field in its place. */
@Composable
fun MisconfiguredScreen(problem: ManagedProblem, modifier: Modifier = Modifier) {
    Column(modifier.verticalScroll(rememberScrollState()).padding(Gap).testTag("misconfigured"), verticalArrangement = Arrangement.spacedBy(Gap)) {
        Heading(stringResource(R.string.managed_title), large = true)
        NoticeText(managedProblem(problem))
        Text(stringResource(R.string.managed_help))
    }
}

@Composable
private fun managedProblem(problem: ManagedProblem): String = when (problem) {
    is ManagedProblem.DeploymentUrl ->
        stringResource(R.string.managed_error_url) + " " + addressProblem(problem.why)
    ManagedProblem.DeploymentUrlNotText -> stringResource(R.string.managed_error_url_not_text)
    ManagedProblem.DeploymentLocked -> stringResource(R.string.managed_error_locked)
    ManagedProblem.LockedWithoutUrl -> stringResource(R.string.managed_error_locked_without_url)
}

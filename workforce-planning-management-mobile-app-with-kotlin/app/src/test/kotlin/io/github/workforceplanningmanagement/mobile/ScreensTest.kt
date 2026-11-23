// The screens' semantics, as TalkBack reads them (WPM-R282–R284), on the JVM with Robolectric.
package io.github.workforceplanningmanagement.mobile

import androidx.compose.ui.test.assert
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.hasContentDescription
import androidx.compose.ui.test.hasText
import androidx.compose.ui.test.isHeading
import androidx.compose.ui.test.junit4.v2.createComposeRule
import androidx.compose.ui.test.onAllNodesWithText
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performScrollTo
import androidx.compose.ui.test.performScrollToNode
import androidx.compose.ui.test.assertCountEquals
import androidx.compose.ui.test.onAllNodesWithTag
import io.github.workforceplanningmanagement.mobile.core.AddressProblem
import io.github.workforceplanningmanagement.mobile.core.LeaveDraft
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config
import java.time.LocalDate

@RunWith(RobolectricTestRunner::class)
@Config(sdk = [36])
class ScreensTest {
    @get:Rule val rule = createComposeRule()

    private val state = UiState(baseUrl = "https://wpm.example.org", signedIn = true, year = 2026, timeOff = TIME_OFF)

    private fun timeOff(onCancel: (String) -> Unit = {}, s: UiState = state) = rule.setContent {
        WpmTheme {
            TimeOffScreen(s, TODAY, {}, {}, {}, {}, onCancel, {})
        }
    }

    private fun scrollTo(text: String) =
        rule.onNodeWithTag("time-off").performScrollToNode(hasText(text, substring = true))

    @Test fun anUnrecordedAllowanceIsNotRecordedNeverZero() {
        timeOff()
        rule.onNodeWithTag("time-off").performScrollToNode(androidx.compose.ui.test.hasTestTag("balance-unpaid"))
        rule.onNodeWithTag("balance-unpaid", useUnmergedTree = true).assertIsDisplayed()
        rule.onAllNodesWithText("Not recorded", useUnmergedTree = true).assertCountEquals(2)
        rule.onAllNodesWithText("0 calendar days", useUnmergedTree = true).fetchSemanticsNodes().let {
            // Unpaid's taken and booked are 0 calendar days; its allowance and remaining are not.
            assertEquals(2, it.size)
        }
    }

    @Test fun everyFigureSaysCalendarDays() {
        timeOff()
        rule.onNodeWithText("Every figure is in calendar days.").assertIsDisplayed()
        rule.onAllNodesWithText("15 calendar days", useUnmergedTree = true).assertCountEquals(1)
    }

    @Test fun theDerivationIsShown() {
        timeOff()
        scrollTo("remaining = days allowed")
        rule.onNodeWithText("remaining = days allowed", substring = true, useUnmergedTree = true).assertIsDisplayed()
    }

    @Test fun headingsAreHeadings() {
        timeOff()
        rule.onNode(isHeading() and hasText("My time off")).assertIsDisplayed()
    }

    @Test fun theStatusTagSaysItsContext() {
        timeOff()
        rule.onNodeWithTag("time-off").performScrollToNode(androidx.compose.ui.test.hasTestTag("request-future-requested"))
        rule.onNode(hasContentDescription("Awaiting a decision", substring = true) and
            hasContentDescription("Status", substring = true), useUnmergedTree = true).assertExists()
    }

    @Test fun cancelIsOfferedOnlyWhereTheServiceWouldAllowIt() {
        timeOff()
        rule.onNodeWithTag("time-off").performScrollToNode(androidx.compose.ui.test.hasTestTag("request-rejected"))
        rule.onNodeWithTag("cancel-future-requested").assertExists()
        rule.onNodeWithTag("cancel-future-approved").assertExists()
        rule.onAllNodesWithTag("cancel-past-approved").assertCountEquals(0)
        rule.onAllNodesWithTag("cancel-sick").assertCountEquals(0)
        rule.onAllNodesWithTag("cancel-rejected").assertCountEquals(0)
    }

    @Test fun aSickReasonIsNeverShown() {
        timeOff()
        rule.onNodeWithTag("time-off").performScrollToNode(androidx.compose.ui.test.hasTestTag("request-rejected"))
        rule.onAllNodesWithText("should never show", substring = true, useUnmergedTree = true).assertCountEquals(0)
    }

    @Test fun cancellingAsksFirst() {
        var cancelled: String? = null
        timeOff(onCancel = { cancelled = it })
        rule.onNodeWithTag("time-off").performScrollToNode(androidx.compose.ui.test.hasTestTag("cancel-future-requested"))
        rule.onNodeWithTag("cancel-future-requested").performClick()
        assertEquals(null, cancelled)
        rule.onNodeWithText("Cancel this request?").assertIsDisplayed()
        rule.onNodeWithText("Cancel the request").performClick()
        assertEquals("future-requested", cancelled)
    }

    @Test fun theYearButtonsSayTheYear() {
        timeOff()
        rule.onNodeWithContentDescription("Previous year, 2025").assertExists()
        rule.onNodeWithContentDescription("Next year, 2027").assertExists()
    }

    @Test fun aRefusalShowsTheServicesWords() {
        timeOff(s = state.copy(timeOff = null, loadFailure = Failure.Refused(422, "a leave total is too large to show")))
        rule.onNodeWithText("a leave total is too large to show", substring = true, useUnmergedTree = true).assertExists()
    }

    private fun requestLeave(draft: LeaveDraft, onSend: () -> Unit = {}) = rule.setContent {
        WpmTheme { RequestLeaveScreen(draft, sending = false, failure = null, onChange = {}, onSend = onSend, onBack = {}) }
    }

    @Test fun sickLeaveHasNoReasonField() {
        requestLeave(LeaveDraft(kind = "sick"))
        rule.onAllNodesWithTag("reason").assertCountEquals(0)
        rule.onNodeWithText("never why", substring = true, useUnmergedTree = true).assertExists()
    }

    @Test fun otherLeaveHasAReasonField() {
        requestLeave(LeaveDraft(kind = "annual"))
        rule.onNodeWithTag("reason").assertExists()
    }

    @Test fun theDaySpanIsShownInCalendarDays() {
        requestLeave(LeaveDraft("annual", LocalDate.of(2026, 12, 21), LocalDate.of(2026, 12, 23)))
        rule.onNodeWithTag("span").assertExists()
        rule.onNodeWithText("This is 3 calendar days.").assertExists()
    }

    @Test fun anIncompleteDraftIsNotSentAndSaysWhy() {
        var sent = false
        requestLeave(LeaveDraft(), onSend = { sent = true })
        rule.onNodeWithTag("send").performScrollTo().performClick()
        assertFalse(sent)
        rule.onNodeWithText("There is a problem", useUnmergedTree = true).assertExists()
        rule.onNodeWithText("Choose the kind of leave").assertExists()
    }

    @Test fun aCompleteDraftIsSent() {
        var sent = false
        requestLeave(LeaveDraft("annual", LocalDate.of(2026, 12, 21), LocalDate.of(2026, 12, 23)), onSend = { sent = true })
        rule.onNodeWithTag("send").performScrollTo().performClick()
        assertEquals(true, sent)
    }

    @Test fun theKindsAreRadioButtons() {
        requestLeave(LeaveDraft())
        rule.onNodeWithTag("kind-annual").assertExists()
        rule.onNodeWithText("Public holiday", useUnmergedTree = true).assertExists()
    }

    @Test fun anAddressProblemIsTheFieldsError() {
        rule.setContent { WpmTheme { ConnectScreen("", AddressProblem.InsecureScheme, {}) } }
        rule.onNodeWithTag("address").assert(
            androidx.compose.ui.test.SemanticsMatcher.expectValue(
                androidx.compose.ui.semantics.SemanticsProperties.Error, "The address must start with https://",
            ),
        )
    }
}

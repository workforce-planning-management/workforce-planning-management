// WPM-R285: the layout follows the window's width. A wide window shows the summary and the requests side by side; a narrow
// one stacks them; the same strings and rules serve both.
package io.github.workforceplanningmanagement.mobile

import androidx.compose.ui.test.assertCountEquals
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.hasTestTag
import androidx.compose.ui.test.junit4.v2.createComposeRule
import androidx.compose.ui.test.onAllNodesWithTag
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performScrollToNode
import androidx.compose.ui.test.performClick
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config

@RunWith(RobolectricTestRunner::class)
@Config(sdk = [36])
class WideLayoutTest {
    @get:Rule val rule = createComposeRule()

    private val state = UiState(baseUrl = "https://wpm.example.org", signedIn = true, year = 2026, timeOff = TIME_OFF)

    /** The window is the test's display: `w900dp` is a tablet, the default is a phone. */
    private fun show(onCancel: (String) -> Unit = {}) = rule.setContent {
        WpmTheme { TimeOffScreen(state, TODAY, {}, {}, {}, {}, onCancel, {}) }
    }

    @Test fun theBoundaryIsSixHundredDp() {
        assertFalse(isWide(599))
        assertTrue(isWide(600))
        assertTrue(isWide(1280))
        assertEquals(600, WIDE_WINDOW_DP)
    }

    @Config(sdk = [36], qualifiers = "w900dp-h900dp")
    @Test fun aWideWindowShowsTheSummaryAndTheRequestsSideBySide() {
        show()
        rule.onNodeWithTag("time-off-summary").assertIsDisplayed()
        rule.onNodeWithTag("time-off-requests").assertIsDisplayed()
        rule.onAllNodesWithTag("time-off").assertCountEquals(0)
        rule.onNodeWithTag("time-off-summary").performScrollToNode(hasTestTag("balance-unpaid"))
        rule.onNodeWithTag("balance-unpaid", useUnmergedTree = true).assertIsDisplayed()
    }

    @Config(sdk = [36], qualifiers = "w400dp-h900dp")
    @Test fun aNarrowWindowStacksThem() {
        show()
        rule.onNodeWithTag("time-off").assertIsDisplayed()
        rule.onAllNodesWithTag("time-off-summary").assertCountEquals(0)
        rule.onAllNodesWithTag("time-off-requests").assertCountEquals(0)
    }

    @Config(sdk = [36], qualifiers = "w900dp-h900dp")
    @Test fun theRequestsAreInTheirOwnColumnAndStillCancel() {
        var cancelled: String? = null
        show { cancelled = it }
        val cancel = state.timeOff!!.requests.first { r ->
            io.github.workforceplanningmanagement.mobile.core.Leave.canCancel(r.status, r.startOn, TODAY)
        }
        rule.onNodeWithTag("time-off-requests").performScrollToNode(hasTestTag("cancel-${cancel.pid}"))
        rule.onNodeWithTag("cancel-${cancel.pid}").performClick()
        rule.onNodeWithText("Cancel the request").performClick()
        assertEquals(cancel.pid, cancelled)
    }
}

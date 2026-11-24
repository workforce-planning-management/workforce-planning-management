// WPM-R286: record text is never copied onto the device. Held three ways: the sources contain no call that copies, shares,
// drags, prints or notifies; no text on the record screen offers a copy action; the window keeps itself out of autofill,
// content capture and screenshots.
package io.github.workforceplanningmanagement.mobile

import android.view.View
import android.view.WindowManager
import androidx.compose.ui.semantics.SemanticsActions
import androidx.compose.ui.test.SemanticsMatcher
import androidx.compose.ui.test.assertCountEquals
import androidx.compose.ui.test.junit4.v2.createComposeRule
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.Robolectric
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config
import java.io.File

@RunWith(RobolectricTestRunner::class)
@Config(sdk = [36])
class RecordTextTest {
    @get:Rule val rule = createComposeRule()

    private fun sources(): List<File> {
        val roots = listOf("src/main", "../core/src/main").map(::File).filter(File::exists)
        return roots.flatMap { it.walkTopDown().filter { f -> f.isFile && f.extension == "kt" }.toList() }
    }

    @Test fun noSourceCopiesSharesDragsPrintsOrNotifies() {
        val banned = listOf(
            "ClipboardManager", "ClipData", "LocalClipboard", "SelectionContainer", "DisableSelection.copy",
            "Intent.ACTION_SEND", "createChooser", "ShareCompat", "NotificationManager", "NotificationCompat",
            "AppWidgetProvider", "PrintManager", "startDragAndDrop", "dragAndDropSource", "ContentCaptureSession",
            "ShortcutManager", "SearchIndexable", "AppSearch", "Log.d(", "Log.i(", "Log.v(", "Log.w(", "Log.e(",
        )
        val found = sources().filter { it.name != "RecordTextTest.kt" }.flatMap { file ->
            val text = file.readText()
            banned.filter(text::contains).map { "${file.name}: $it" }
        }
        assertTrue("record text must not leave the app: $found", found.isEmpty())
        assertTrue("the scan found the app's sources", sources().size > 5)
    }

    @Test fun noTextOnTheRecordScreenOffersACopyAction() {
        rule.setContent {
            WpmTheme { TimeOffScreen(UiState(baseUrl = "https://wpm.example.org", signedIn = true, year = 2026, timeOff = TIME_OFF), TODAY, {}, {}, {}, {}, {}, {}) }
        }
        rule.onAllNodes(SemanticsMatcher.keyIsDefined(SemanticsActions.CopyText), useUnmergedTree = true).assertCountEquals(0)
    }

    @Test fun theWindowIsSecureAndOutOfAutofillAndContentCapture() {
        val activity = Robolectric.buildActivity(MainActivity::class.java).setup().get()
        val flags = activity.window.attributes.flags
        assertEquals(WindowManager.LayoutParams.FLAG_SECURE, flags and WindowManager.LayoutParams.FLAG_SECURE)
        val view = activity.window.decorView
        assertEquals(View.IMPORTANT_FOR_AUTOFILL_NO_EXCLUDE_DESCENDANTS, view.importantForAutofill)
        assertEquals(View.IMPORTANT_FOR_CONTENT_CAPTURE_NO_EXCLUDE_DESCENDANTS, view.importantForContentCapture)
    }
}

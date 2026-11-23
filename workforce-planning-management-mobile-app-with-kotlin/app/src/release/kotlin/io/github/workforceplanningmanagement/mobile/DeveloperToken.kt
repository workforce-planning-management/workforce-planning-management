// A release build has no developer token screen (WPM-R281).
package io.github.workforceplanningmanagement.mobile

import androidx.compose.runtime.Composable

object DeveloperToken {
    val entry: (@Composable (onToken: (String) -> Unit) -> Unit)? = null
}

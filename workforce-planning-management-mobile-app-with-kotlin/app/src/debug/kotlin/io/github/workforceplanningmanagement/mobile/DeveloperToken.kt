// Debug builds only (WPM-R281): paste a token minted for a local service. The release source set has no such screen.
package io.github.workforceplanningmanagement.mobile

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.material3.OutlinedButton
import androidx.compose.foundation.text.input.rememberTextFieldState
import androidx.compose.material3.OutlinedSecureTextField
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.semantics.heading
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.unit.dp
import io.github.lilydesignsystem.compose.Field
import io.github.lilydesignsystem.compose.LilyMinimumTouchTarget

object DeveloperToken {
    val entry: (@Composable (onToken: (String) -> Unit) -> Unit)? = { onToken -> Entry(onToken) }

    @Composable
    private fun Entry(onToken: (String) -> Unit) {
        val token = rememberTextFieldState()
        Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
            Text(stringResource(R.string.dev_token_title), modifier = Modifier.semantics { heading() })
            Field(label = stringResource(R.string.dev_token_label), description = stringResource(R.string.dev_token_help)) { m ->
                OutlinedSecureTextField(state = token, modifier = m.fillMaxWidth().testTag("dev-token"))
            }
            OutlinedButton(onClick = { onToken(token.text.toString()) }, modifier = Modifier.heightIn(min = LilyMinimumTouchTarget)) {
                Text(stringResource(R.string.dev_token_use))
            }
        }
    }
}

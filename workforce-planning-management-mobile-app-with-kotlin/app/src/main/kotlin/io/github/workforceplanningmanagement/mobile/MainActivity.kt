// The one activity: light or dark with the system through Lily's tokens, edge to edge, FLAG_SECURE so no personal screen is
// in the recents snapshot or a screenshot (WPM-D227), and the sign-in redirect wpm-mobile://signin (WPM-R281).
package io.github.workforceplanningmanagement.mobile

import android.content.ActivityNotFoundException
import android.content.Intent
import android.os.Bundle
import android.view.WindowManager
import androidx.activity.ComponentActivity
import androidx.activity.compose.BackHandler
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.browser.customtabs.CustomTabsIntent
import androidx.core.net.toUri
import androidx.compose.foundation.background
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.safeDrawingPadding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.lifecycle.viewmodel.compose.viewModel
import androidx.lifecycle.viewmodel.initializer
import androidx.lifecycle.viewmodel.viewModelFactory
import io.github.lilydesignsystem.compose.themes.LilyTheme
import io.github.lilydesignsystem.compose.themes.LilyTokens
import kotlinx.coroutines.Dispatchers
import java.security.SecureRandom
import java.time.LocalDate

class MainActivity : ComponentActivity() {
    private var viewModel: AppViewModel? = null

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        window.setFlags(WindowManager.LayoutParams.FLAG_SECURE, WindowManager.LayoutParams.FLAG_SECURE)
        enableEdgeToEdge()
        val app = applicationContext
        val factory = viewModelFactory {
            initializer {
                val random = SecureRandom()
                AppViewModel(
                    tokens = KeystoreTokenStore(app),
                    addresses = PreferencesAddressStore(app),
                    transport = HttpUrlTransport(),
                    io = Dispatchers.IO,
                    today = { LocalDate.now() },
                    random = { n -> ByteArray(n).also(random::nextBytes) },
                )
            }
        }
        setContent {
            val vm: AppViewModel = viewModel(factory = factory)
            viewModel = vm
            WpmTheme { App(vm, onOpenBrowser = ::openBrowser) }
        }
        handle(intent)
    }

    override fun onNewIntent(intent: Intent) {
        super.onNewIntent(intent)
        handle(intent)
    }

    private fun handle(intent: Intent?) {
        val data = intent?.data ?: return
        if (data.scheme == "wpm-mobile" && data.host == "signin") {
            // The view model exists once setContent has run; a cold start through the redirect has no sign-in to complete.
            window.decorView.post { viewModel?.completeSignIn(data.toString()) }
        }
    }

    private fun openBrowser(url: String): Boolean = try {
        CustomTabsIntent.Builder().build().launchUrl(this, url.toUri())
        true
    } catch (_: ActivityNotFoundException) {
        false
    }
}

@Composable
fun WpmTheme(content: @Composable () -> Unit) {
    val tokens = if (isSystemInDarkTheme()) LilyTokens.dark else LilyTokens.light
    val base = if (isSystemInDarkTheme()) darkColorScheme() else lightColorScheme()
    val colors = base.copy(
        primary = tokens.primary, onPrimary = tokens.primaryContent,
        background = tokens.base100, onBackground = tokens.baseContent,
        surface = tokens.base100, onSurface = tokens.baseContent,
        error = tokens.error, onError = tokens.errorContent,
    )
    LilyTheme(tokens) {
        MaterialTheme(colorScheme = colors) {
            androidx.compose.foundation.layout.Box(Modifier.fillMaxSize().background(tokens.base100).safeDrawingPadding()) {
                content()
            }
        }
    }
}

@Composable
fun App(vm: AppViewModel, onOpenBrowser: (String) -> Boolean) {
    val state by vm.state.collectAsState()
    val base = state.baseUrl
    when {
        base == null -> ConnectScreen(initial = "", problem = state.addressProblem, onContinue = vm::connect)
        !state.signedIn -> SignInScreen(
            baseUrl = base,
            notice = state.notice,
            onSignIn = { vm.startSignIn()?.let { if (!onOpenBrowser(it)) vm.noBrowser() } },
            onChangeAddress = vm::changeAddress,
            developerToken = DeveloperToken.entry,
            onToken = vm::useToken,
        )
        state.screen == Screen.RequestLeave -> {
            BackHandler { vm.closeRequestLeave() }
            RequestLeaveScreen(
                draft = state.draft,
                sending = state.sending,
                failure = state.sendFailure,
                onChange = vm::updateDraft,
                onSend = vm::sendLeave,
                onBack = vm::closeRequestLeave,
            )
        }
        else -> TimeOffScreen(
            state = state,
            today = vm.today(),
            onPreviousYear = vm::previousYear,
            onNextYear = vm::nextYear,
            onRefresh = vm::load,
            onRequestLeave = vm::openRequestLeave,
            onCancel = vm::cancel,
            onSignOut = { vm.signOut() },
        )
    }
}

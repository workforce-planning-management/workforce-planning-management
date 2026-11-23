// The app's state and what each action does. The rules are the core's (WPM-D225); this only sequences calls and keeps the
// result in memory for the screen showing it (WPM-D227).
package io.github.workforceplanningmanagement.mobile

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import io.github.workforceplanningmanagement.mobile.core.AddressProblem
import io.github.workforceplanningmanagement.mobile.core.AddressResult
import io.github.workforceplanningmanagement.mobile.core.ApiResult
import io.github.workforceplanningmanagement.mobile.core.CallbackProblem
import io.github.workforceplanningmanagement.mobile.core.CallbackResult
import io.github.workforceplanningmanagement.mobile.core.Connection
import io.github.workforceplanningmanagement.mobile.core.HttpResponse
import io.github.workforceplanningmanagement.mobile.core.Leave
import io.github.workforceplanningmanagement.mobile.core.ManagedConfig
import io.github.workforceplanningmanagement.mobile.core.ManagedProblem
import io.github.workforceplanningmanagement.mobile.core.ManagedResult
import io.github.workforceplanningmanagement.mobile.core.LeaveDraft
import io.github.workforceplanningmanagement.mobile.core.PendingSignIn
import io.github.workforceplanningmanagement.mobile.core.Pkce
import io.github.workforceplanningmanagement.mobile.core.SignIn
import io.github.workforceplanningmanagement.mobile.core.SignInConfig
import io.github.workforceplanningmanagement.mobile.core.TimeOff
import io.github.workforceplanningmanagement.mobile.core.Transport
import io.github.workforceplanningmanagement.mobile.core.WpmApi
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import java.io.IOException
import java.time.LocalDate

/** A message the screen shows once, in its own words. */
enum class Notice { SignInFailed, StateMismatch, SignInDenied, NoBrowser, SignedOutExpired, LeaveSent, RequestCancelled }

/** Why something could not be shown or done. [detail] is the service's own words, when it gave them. */
sealed interface Failure {
    data object Unreachable : Failure
    data object Unreadable : Failure
    data class Refused(val status: Int, val detail: String?) : Failure
}

enum class Screen { TimeOff, RequestLeave }

data class UiState(
    val baseUrl: String? = null,
    val addressProblem: AddressProblem? = null,
    val signedIn: Boolean = false,
    val screen: Screen = Screen.TimeOff,
    val year: Int,
    val loading: Boolean = false,
    val timeOff: TimeOff? = null,
    val loadFailure: Failure? = null,
    val draft: LeaveDraft = LeaveDraft(),
    val sending: Boolean = false,
    val sendFailure: Failure? = null,
    val notice: Notice? = null,
    /** A setting a management tool supplied that is not valid (WPM-R285): the app stops on it and names it. */
    val managedProblem: ManagedProblem? = null,
    /** A management tool set the address and asked that it not be changed (WPM-R285). */
    val addressLocked: Boolean = false,
)

class AppViewModel(
    private val tokens: TokenStore,
    private val addresses: AddressStore,
    private val transport: Transport,
    private val io: CoroutineDispatcher,
    private val today: () -> LocalDate,
    private val random: (Int) -> ByteArray,
    private val config: SignInConfig = SignInConfig(),
    /** The management tool's values (application restrictions), or null when there are none (WPM-R285). */
    private val managed: () -> Map<String, Any?>? = { null },
) : ViewModel() {
    private val _state = MutableStateFlow(initialState())
    val state: StateFlow<UiState> = _state.asStateFlow()
    private var pending: PendingSignIn? = null

    private fun initialState(): UiState {
        val year = today().year
        return when (val m = ManagedConfig.read(managed())) {
            // No fallback to a saved address and no connect screen: the setting is named and the app waits for it (RCA-IN-4).
            is ManagedResult.Refused -> UiState(year = year, managedProblem = m.problem)
            is ManagedResult.Settings -> {
                val saved = addresses.load()
                val preset = m.baseUrl
                // A token for another deployment is not kept.
                if (preset != null && preset != saved) {
                    tokens.clear()
                    addresses.save(preset)
                }
                val base = preset ?: saved
                UiState(baseUrl = base, signedIn = tokens.load() != null, year = year, addressLocked = m.locked)
            }
        }
    }

    init {
        if (_state.value.signedIn && _state.value.baseUrl != null) load()
    }

    fun today(): LocalDate = today.invoke()

    fun connect(input: String) {
        if (_state.value.addressLocked) return
        when (val r = Connection.parse(input)) {
            is AddressResult.Accepted -> {
                addresses.save(r.baseUrl)
                _state.update { it.copy(baseUrl = r.baseUrl, addressProblem = null) }
            }
            is AddressResult.Refused -> _state.update { it.copy(addressProblem = r.problem) }
        }
    }

    fun changeAddress() {
        signOut()
        if (_state.value.addressLocked) return
        addresses.clear()
        _state.update { it.copy(baseUrl = null) }
    }

    /** Starts a sign-in and returns the address to open in the browser. The verifier stays in memory. */
    fun startSignIn(): String? {
        val base = _state.value.baseUrl ?: return null
        val p = SignIn.start(base, config, random(Pkce.RANDOM_BYTES), random(Pkce.RANDOM_BYTES))
        pending = p
        _state.update { it.copy(notice = null) }
        return p.authorizeUrl
    }

    fun noBrowser() {
        pending = null
        _state.update { it.copy(notice = Notice.NoBrowser) }
    }

    /** The browser came back to wpm-mobile://signin. */
    fun completeSignIn(uri: String) {
        val p = pending
        val base = _state.value.baseUrl
        pending = null
        if (p == null || base == null) {
            _state.update { it.copy(notice = Notice.SignInFailed) }
            return
        }
        when (val r = SignIn.callback(uri, p, config)) {
            is CallbackResult.Refused -> _state.update {
                it.copy(notice = when (r.problem) {
                    CallbackProblem.StateMismatch -> Notice.StateMismatch
                    CallbackProblem.Denied -> Notice.SignInDenied
                    else -> Notice.SignInFailed
                })
            }
            is CallbackResult.Code -> viewModelScope.launch {
                val response = withContext(io) {
                    try {
                        transport.send(SignIn.tokenRequest(base, config, r.code, p))
                    } catch (_: IOException) {
                        HttpResponse(0, "")
                    }
                }
                val token = SignIn.accessToken(response)
                if (token == null) {
                    _state.update { it.copy(notice = Notice.SignInFailed) }
                } else {
                    useToken(token)
                }
            }
        }
    }

    /** A token from the developer screen (debug builds only) or from the sign-in. */
    fun useToken(token: String) {
        val t = token.trim()
        if (t.isEmpty()) return
        tokens.save(t)
        _state.update { it.copy(signedIn = true, notice = null) }
        load()
    }

    fun signOut(notice: Notice? = null) {
        tokens.clear()
        pending = null
        _state.update {
            UiState(baseUrl = it.baseUrl, year = today().year, notice = notice, addressLocked = it.addressLocked)
        }
    }

    fun previousYear() = setYear(_state.value.year - 1)
    fun nextYear() = setYear(_state.value.year + 1)
    private fun setYear(year: Int) {
        _state.update { it.copy(year = year, timeOff = null) }
        load()
    }

    fun dismissNotice() = _state.update { it.copy(notice = null) }

    private fun api(): WpmApi? {
        val base = _state.value.baseUrl ?: return null
        val token = tokens.load() ?: return null
        return WpmApi(base, token, transport)
    }

    fun load() {
        val api = api() ?: return
        val year = _state.value.year
        _state.update { it.copy(loading = true, loadFailure = null) }
        viewModelScope.launch {
            val r = withContext(io) { api.timeOff(year) }
            if (r is ApiResult.SignedOut) return@launch signOut(Notice.SignedOutExpired)
            _state.update {
                if (it.year != year) it
                else when (r) {
                    is ApiResult.Ok -> it.copy(loading = false, timeOff = r.value)
                    else -> it.copy(loading = false, loadFailure = failure(r))
                }
            }
        }
    }

    fun openRequestLeave() = _state.update { it.copy(screen = Screen.RequestLeave, sendFailure = null, notice = null) }
    fun closeRequestLeave() = _state.update { it.copy(screen = Screen.TimeOff, sendFailure = null) }
    fun updateDraft(draft: LeaveDraft) = _state.update { it.copy(draft = draft) }

    /** Sends the draft once; a second tap while it is in flight does nothing. */
    fun sendLeave() {
        val s = _state.value
        if (s.sending) return
        val payload = Leave.payload(s.draft) ?: return
        val api = api() ?: return
        _state.update { it.copy(sending = true, sendFailure = null) }
        viewModelScope.launch {
            val r = withContext(io) { api.requestLeave(payload) }
            when (r) {
                is ApiResult.Ok -> {
                    _state.update {
                        it.copy(sending = false, screen = Screen.TimeOff, draft = LeaveDraft(), notice = Notice.LeaveSent)
                    }
                    load()
                }
                ApiResult.SignedOut -> signOut(Notice.SignedOutExpired)
                else -> _state.update { it.copy(sending = false, sendFailure = failure(r)) }
            }
        }
    }

    /** Cancels one request, after the person confirmed. */
    fun cancel(pid: String) {
        if (_state.value.sending) return
        val api = api() ?: return
        _state.update { it.copy(sending = true, loadFailure = null) }
        viewModelScope.launch {
            val r = withContext(io) { api.cancel(pid) }
            when (r) {
                is ApiResult.Ok -> {
                    _state.update { it.copy(sending = false, notice = Notice.RequestCancelled) }
                    load()
                }
                ApiResult.SignedOut -> signOut(Notice.SignedOutExpired)
                else -> _state.update { it.copy(sending = false, loadFailure = failure(r)) }
            }
        }
    }

    private fun failure(r: ApiResult<*>): Failure = when (r) {
        is ApiResult.Refused -> Failure.Refused(r.status, r.problem?.detail)
        is ApiResult.Unreachable -> if (r.unreadable) Failure.Unreadable else Failure.Unreachable
        else -> Failure.Unreachable
    }
}

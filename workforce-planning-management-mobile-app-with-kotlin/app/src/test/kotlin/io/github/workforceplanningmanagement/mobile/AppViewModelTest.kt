// The view model's sequences with a fake transport: sign-in, 401, one send per tap (WPM-R281, WPM-R283).
package io.github.workforceplanningmanagement.mobile

import io.github.workforceplanningmanagement.mobile.core.AddressProblem
import io.github.workforceplanningmanagement.mobile.core.ManagedProblem
import io.github.workforceplanningmanagement.mobile.core.HttpRequest
import io.github.workforceplanningmanagement.mobile.core.HttpResponse
import io.github.workforceplanningmanagement.mobile.core.LeaveDraft
import io.github.workforceplanningmanagement.mobile.core.SignIn
import io.github.workforceplanningmanagement.mobile.core.Transport
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.setMain
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import java.time.LocalDate

@OptIn(ExperimentalCoroutinesApi::class)
class AppViewModelTest {
    private class MemoryTokens(var token: String? = null) : TokenStore {
        override fun load() = token
        override fun save(token: String) { this.token = token }
        override fun clear() { token = null }
    }

    private class MemoryAddresses(var address: String? = null) : AddressStore {
        override fun load() = address
        override fun save(baseUrl: String) { address = baseUrl }
        override fun clear() { address = null }
    }

    private val sent = mutableListOf<HttpRequest>()
    private var respond: (HttpRequest) -> HttpResponse = { HttpResponse(200, TIME_OFF_JSON) }
    private val transport = Transport { sent += it; respond(it) }
    private val dispatcher = UnconfinedTestDispatcher()

    @Before fun main() = Dispatchers.setMain(dispatcher)
    @After fun reset() = Dispatchers.resetMain()

    private fun vm(
        tokens: TokenStore = MemoryTokens(),
        addresses: AddressStore = MemoryAddresses(),
        managed: Map<String, Any?>? = null,
    ) = AppViewModel(tokens, addresses, transport, dispatcher, { TODAY }, { n -> ByteArray(n) { 7 } }, managed = { managed })

    @Test fun aPresetAddressSkipsTheConnectScreen() {
        val addresses = MemoryAddresses()
        val v = vm(addresses = addresses, managed = mapOf("deployment_url" to "https://wpm.example.org"))
        assertEquals("https://wpm.example.org", v.state.value.baseUrl)
        assertFalse(v.state.value.addressLocked)
        assertEquals("https://wpm.example.org", addresses.address)
    }

    @Test fun aLockedAddressCannotBeChanged() {
        val v = vm(managed = mapOf("deployment_url" to "https://wpm.example.org", "deployment_locked" to true))
        assertTrue(v.state.value.addressLocked)
        v.connect("https://other.example.org")
        assertEquals("https://wpm.example.org", v.state.value.baseUrl)
        v.changeAddress()
        assertEquals("the address stays; only the token goes", "https://wpm.example.org", v.state.value.baseUrl)
    }

    @Test fun aChangedPresetDropsTheTokenAndAnUnchangedOneKeepsIt() {
        val kept = MemoryTokens("t")
        vm(kept, MemoryAddresses("https://wpm.example.org"), mapOf("deployment_url" to "https://wpm.example.org"))
        assertEquals("t", kept.token)
        val moved = MemoryTokens("t")
        val v = vm(moved, MemoryAddresses("https://wpm.example.org"), mapOf("deployment_url" to "https://new.example.org"))
        assertNull("a token for another deployment is not kept", moved.token)
        assertFalse(v.state.value.signedIn)
    }

    @Test fun anInvalidSettingStopsTheAppAndNamesIt() {
        val tokens = MemoryTokens("t")
        val v = vm(tokens, MemoryAddresses("https://wpm.example.org"), mapOf("deployment_url" to "http://wpm.example.org"))
        assertEquals(ManagedProblem.DeploymentUrl(AddressProblem.InsecureScheme), v.state.value.managedProblem)
        assertNull("no fallback to the saved address", v.state.value.baseUrl)
        assertEquals("nothing is deleted because of a setting", "t", tokens.token)
        assertEquals(
            ManagedProblem.DeploymentLocked,
            vm(managed = mapOf("deployment_url" to "https://wpm.example.org", "deployment_locked" to "maybe")).state.value.managedProblem,
        )
    }

    @Test fun noManagedSettingsBehavesAsBefore() {
        val v = vm()
        assertNull(v.state.value.managedProblem)
        assertFalse(v.state.value.addressLocked)
        assertNull(v.state.value.baseUrl)
    }

    @Test fun aRefusedAddressSendsNothing() {
        val v = vm()
        v.connect("http://wpm.example.org")
        assertEquals(AddressProblem.InsecureScheme, v.state.value.addressProblem)
        assertNull(v.state.value.baseUrl)
        assertTrue(sent.isEmpty())
    }

    @Test fun anAcceptedAddressIsKept() {
        val addresses = MemoryAddresses()
        val v = vm(addresses = addresses)
        v.connect("https://wpm.example.org/")
        assertEquals("https://wpm.example.org", addresses.address)
        assertEquals("https://wpm.example.org", v.state.value.baseUrl)
    }

    @Test fun aSignedInStartLoadsTheYear() {
        val v = vm(MemoryTokens("t"), MemoryAddresses("https://wpm.example.org"))
        assertEquals("https://wpm.example.org/api/me/time-off?year=2026", sent.single().url)
        assertEquals(5, v.state.value.timeOff?.requests?.size)
    }

    @Test fun signInExchangesTheCodeWithTheVerifier() {
        val tokens = MemoryTokens()
        val v = vm(tokens, MemoryAddresses("https://wpm.example.org"))
        respond = { r ->
            if (r.url.endsWith("/api/auth/token")) HttpResponse(200, """{"access_token":"v4.public.abc"}""")
            else HttpResponse(200, TIME_OFF_JSON)
        }
        val url = v.startSignIn()!!
        val state = SignIn.parseQuery(url.substringAfter('?'))["state"]!!
        v.completeSignIn("wpm-mobile://signin?code=c1&state=$state")
        assertEquals("v4.public.abc", tokens.token)
        assertTrue(v.state.value.signedIn)
        val exchange = sent.first()
        assertEquals("c1", SignIn.parseQuery(exchange.body)["code"])
        assertEquals("Bearer v4.public.abc", sent.last().headers["Authorization"])
    }

    @Test fun aForeignStateIsRefused() {
        val tokens = MemoryTokens()
        val v = vm(tokens, MemoryAddresses("https://wpm.example.org"))
        v.startSignIn()
        v.completeSignIn("wpm-mobile://signin?code=c1&state=forged")
        assertEquals(Notice.StateMismatch, v.state.value.notice)
        assertNull(tokens.token)
        assertTrue(sent.isEmpty())
    }

    @Test fun aReturnWithNoSignInStartedIsRefused() {
        val v = vm(addresses = MemoryAddresses("https://wpm.example.org"))
        v.completeSignIn("wpm-mobile://signin?code=c1&state=x")
        assertEquals(Notice.SignInFailed, v.state.value.notice)
    }

    @Test fun a401SignsOutAndSaysSo() {
        val tokens = MemoryTokens("expired")
        respond = { HttpResponse(401, "") }
        val v = vm(tokens, MemoryAddresses("https://wpm.example.org"))
        assertNull(tokens.token)
        assertFalse(v.state.value.signedIn)
        assertEquals(Notice.SignedOutExpired, v.state.value.notice)
    }

    @Test fun signOutDeletesTheToken() {
        val tokens = MemoryTokens("t")
        val v = vm(tokens, MemoryAddresses("https://wpm.example.org"))
        v.signOut()
        assertNull(tokens.token)
        assertNull(v.state.value.timeOff)
    }

    @Test fun aLeaveRequestIsSentOnceAndTheYearReloaded() {
        val v = vm(MemoryTokens("t"), MemoryAddresses("https://wpm.example.org"))
        sent.clear()
        respond = { HttpResponse(201, "{}") }
        v.openRequestLeave()
        v.updateDraft(LeaveDraft("annual", LocalDate.of(2026, 12, 21), LocalDate.of(2026, 12, 23)))
        v.sendLeave()
        assertEquals("POST", sent.first().method)
        assertEquals(Notice.LeaveSent, v.state.value.notice)
        assertEquals(Screen.TimeOff, v.state.value.screen)
        assertEquals(2, sent.size) // the request, then the reload
    }

    @Test fun anIncompleteDraftSendsNothing() {
        val v = vm(MemoryTokens("t"), MemoryAddresses("https://wpm.example.org"))
        sent.clear()
        v.updateDraft(LeaveDraft(kind = "annual"))
        v.sendLeave()
        assertTrue(sent.isEmpty())
    }

    @Test fun aRefusalIsShownAsTheServiceWroteIt() {
        val v = vm(MemoryTokens("t"), MemoryAddresses("https://wpm.example.org"))
        respond = { HttpResponse(422, """{"status":422,"code":"unprocessable","detail":"not enough annual leave"}""") }
        v.openRequestLeave()
        v.updateDraft(LeaveDraft("annual", LocalDate.of(2026, 12, 21), LocalDate.of(2026, 12, 23)))
        v.sendLeave()
        assertEquals(Failure.Refused(422, "not enough annual leave"), v.state.value.sendFailure)
        assertEquals(Screen.RequestLeave, v.state.value.screen)
        assertFalse(v.state.value.sending)
    }

    @Test fun theYearMovesAndReloads() {
        val v = vm(MemoryTokens("t"), MemoryAddresses("https://wpm.example.org"))
        v.previousYear()
        assertEquals(2025, v.state.value.year)
        assertEquals("https://wpm.example.org/api/me/time-off?year=2025", sent.last().url)
    }
}

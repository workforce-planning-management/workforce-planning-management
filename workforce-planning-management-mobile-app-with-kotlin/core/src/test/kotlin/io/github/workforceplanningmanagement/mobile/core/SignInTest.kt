package io.github.workforceplanningmanagement.mobile.core

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

class SignInTest {
    private val config = SignInConfig()
    private val pending = SignIn.start("https://wpm.example.org", config, ByteArray(32) { 1 }, ByteArray(32) { 2 })

    @Test fun theAuthorizeUrlCarriesCodeStateAndAnS256Challenge() {
        val url = pending.authorizeUrl
        assertTrue(url.startsWith("https://wpm.example.org/api/auth/authorize?"))
        val q = SignIn.parseQuery(url.substringAfter('?'))
        assertEquals("code", q["response_type"])
        assertEquals("wpm-mobile", q["client_id"])
        assertEquals("wpm-mobile://signin", q["redirect_uri"])
        assertEquals(pending.state, q["state"])
        assertEquals("S256", q["code_challenge_method"])
        assertEquals(Pkce.challenge(pending.verifier), q["code_challenge"])
        assertTrue("the verifier never leaves the app", pending.verifier !in url)
    }

    @Test fun theStateAndVerifierDiffer() {
        assertTrue(pending.state != pending.verifier)
    }

    @Test fun aReturnWithTheStateAndACodeIsAccepted() {
        val r = SignIn.callback("wpm-mobile://signin?code=abc%2F1&state=${pending.state}", pending, config)
        assertEquals(CallbackResult.Code("abc/1"), r)
    }

    @Test fun aWrongOrMissingStateIsRefused() {
        assertEquals(CallbackResult.Refused(CallbackProblem.StateMismatch),
            SignIn.callback("wpm-mobile://signin?code=abc&state=other", pending, config))
        assertEquals(CallbackResult.Refused(CallbackProblem.StateMismatch),
            SignIn.callback("wpm-mobile://signin?code=abc", pending, config))
        assertEquals(CallbackResult.Refused(CallbackProblem.StateMismatch),
            SignIn.callback("wpm-mobile://signin?code=abc&state=${pending.state.dropLast(1)}", pending, config))
    }

    @Test fun anotherRedirectIsRefused() {
        assertEquals(CallbackResult.Refused(CallbackProblem.NotOurRedirect),
            SignIn.callback("https://evil.example/signin?code=abc&state=${pending.state}", pending, config))
        assertEquals(CallbackResult.Refused(CallbackProblem.NotOurRedirect),
            SignIn.callback("wpm-mobile://other?code=abc&state=${pending.state}", pending, config))
        assertEquals(CallbackResult.Refused(CallbackProblem.NotOurRedirect),
            SignIn.callback("not a uri", pending, config))
    }

    @Test fun aDenialOrAMissingCodeIsRefused() {
        assertEquals(CallbackResult.Refused(CallbackProblem.Denied, "access_denied"),
            SignIn.callback("wpm-mobile://signin?error=access_denied&state=${pending.state}", pending, config))
        assertEquals(CallbackResult.Refused(CallbackProblem.MissingCode),
            SignIn.callback("wpm-mobile://signin?code=&state=${pending.state}", pending, config))
    }

    @Test fun theTokenRequestIsAFormPostWithTheVerifier() {
        val r = SignIn.tokenRequest("https://wpm.example.org", config, "abc", pending)
        assertEquals("POST", r.method)
        assertEquals("https://wpm.example.org/api/auth/token", r.url)
        assertEquals("application/x-www-form-urlencoded", r.headers["Content-Type"])
        val form = SignIn.parseQuery(r.body)
        assertEquals("authorization_code", form["grant_type"])
        assertEquals("abc", form["code"])
        assertEquals(pending.verifier, form["code_verifier"])
        assertEquals("wpm-mobile://signin", form["redirect_uri"])
    }

    @Test fun theAccessTokenIsReadFromA200Only() {
        assertEquals("v4.public.x", SignIn.accessToken(HttpResponse(200, """{"access_token":"v4.public.x","token_type":"Bearer"}""")))
        assertNull(SignIn.accessToken(HttpResponse(400, """{"access_token":"v4.public.x"}""")))
        assertNull(SignIn.accessToken(HttpResponse(200, """{"access_token":"  "}""")))
        assertNull(SignIn.accessToken(HttpResponse(200, "not json")))
    }

    @Test fun pathsAreConfiguration() {
        val c = SignInConfig(authorizePath = "/oauth/authorize", tokenPath = "/oauth/token", clientId = "x")
        val p = SignIn.start("https://a.example", c, ByteArray(32), ByteArray(32) { 9 })
        assertTrue(p.authorizeUrl.startsWith("https://a.example/oauth/authorize?"))
        assertEquals("https://a.example/oauth/token", SignIn.tokenRequest("https://a.example", c, "c", p).url)
    }
}

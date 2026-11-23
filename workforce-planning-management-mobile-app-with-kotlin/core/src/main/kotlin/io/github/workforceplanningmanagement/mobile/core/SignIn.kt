// The authorization-code sign-in with PKCE through the system browser (WPM-R281, WPM-D224). The authentication service's
// endpoint paths are configuration, because this repository cannot check them.
package io.github.workforceplanningmanagement.mobile.core

import java.net.URI
import java.net.URLDecoder
import java.net.URLEncoder

/** Where the authentication service is and who the app says it is. Defaults are WPM-R281's; none is verified. */
public data class SignInConfig(
    val authorizePath: String = "/api/auth/authorize",
    val tokenPath: String = "/api/auth/token",
    val clientId: String = "wpm-mobile",
    val redirectUri: String = REDIRECT_URI,
) {
    public companion object {
        /** The app's own redirect: its scheme and the `signin` host. */
        public const val REDIRECT_URI: String = "wpm-mobile://signin"
    }
}

/** A sign-in started: what to open, and what to keep in memory until the browser returns. */
public data class PendingSignIn(val authorizeUrl: String, val state: String, val verifier: String)

/** Why a return from the browser was refused. */
public enum class CallbackProblem { NotOurRedirect, StateMismatch, MissingCode, Denied }

/** The outcome of the browser's return. */
public sealed interface CallbackResult {
    /** An authorization code to exchange. */
    public data class Code(val code: String) : CallbackResult

    /** Refused; [error] is the service's `error` value when it sent one. */
    public data class Refused(val problem: CallbackProblem, val error: String? = null) : CallbackResult
}

/** Building and checking the sign-in. */
public object SignIn {
    /** Starts a sign-in against [baseUrl] with fresh [stateRandom] and [verifierRandom] bytes. */
    public fun start(
        baseUrl: String,
        config: SignInConfig,
        stateRandom: ByteArray,
        verifierRandom: ByteArray,
    ): PendingSignIn {
        val state = Pkce.verifier(stateRandom)
        val verifier = Pkce.verifier(verifierRandom)
        val query = form(
            "response_type" to "code",
            "client_id" to config.clientId,
            "redirect_uri" to config.redirectUri,
            "state" to state,
            "code_challenge" to Pkce.challenge(verifier),
            "code_challenge_method" to "S256",
        )
        return PendingSignIn("$baseUrl${config.authorizePath}?$query", state, verifier)
    }

    /** Reads the browser's return [uri] against the [pending] sign-in. The state is compared in full. */
    public fun callback(uri: String, pending: PendingSignIn, config: SignInConfig): CallbackResult {
        val parsed = runCatching { URI(uri) }.getOrNull()
            ?: return CallbackResult.Refused(CallbackProblem.NotOurRedirect)
        val expected = URI(config.redirectUri)
        if (parsed.scheme != expected.scheme || parsed.host != expected.host) {
            return CallbackResult.Refused(CallbackProblem.NotOurRedirect)
        }
        val params = parseQuery(parsed.rawQuery)
        if (params["state"] != pending.state) return CallbackResult.Refused(CallbackProblem.StateMismatch)
        params["error"]?.let { return CallbackResult.Refused(CallbackProblem.Denied, it) }
        val code = params["code"]?.takeIf { it.isNotEmpty() }
            ?: return CallbackResult.Refused(CallbackProblem.MissingCode)
        return CallbackResult.Code(code)
    }

    /** The token request for [code]: a form POST to the token path. */
    public fun tokenRequest(baseUrl: String, config: SignInConfig, code: String, pending: PendingSignIn): HttpRequest =
        HttpRequest(
            method = "POST",
            url = "$baseUrl${config.tokenPath}",
            headers = mapOf("Accept" to "application/json", "Content-Type" to "application/x-www-form-urlencoded"),
            body = form(
                "grant_type" to "authorization_code",
                "code" to code,
                "redirect_uri" to config.redirectUri,
                "client_id" to config.clientId,
                "code_verifier" to pending.verifier,
            ),
        )

    /** The access token from a token response, or null when there is none. */
    public fun accessToken(response: HttpResponse): String? {
        if (response.status != 200) return null
        return runCatching { Json.decodeFromString<TokenResponse>(response.body).accessToken }
            .getOrNull()?.takeIf { it.isNotBlank() }
    }

    internal fun form(vararg pairs: Pair<String, String>): String =
        pairs.joinToString("&") { (k, v) -> "${encode(k)}=${encode(v)}" }

    private fun encode(s: String) = URLEncoder.encode(s, Charsets.UTF_8).replace("+", "%20")

    /** Decodes a query or form body into its parameters; the last of a repeated name wins. */
    public fun parseQuery(raw: String?): Map<String, String> =
        raw.orEmpty().split('&').filter { it.isNotEmpty() }.associate { part ->
            val i = part.indexOf('=')
            val k = if (i < 0) part else part.substring(0, i)
            val v = if (i < 0) "" else part.substring(i + 1)
            URLDecoder.decode(k, Charsets.UTF_8) to URLDecoder.decode(v, Charsets.UTF_8)
        }
}

@kotlinx.serialization.Serializable
internal data class TokenResponse(
    @kotlinx.serialization.SerialName("access_token") val accessToken: String,
)

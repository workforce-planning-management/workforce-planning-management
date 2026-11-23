// The /api/me calls of the first slice (WPM-R280–R283). Every request says Accepts-version 1.0 and Accept JSON, and carries
// the bearer token; a failure is read as the service's problem details (RFC 9457).
package io.github.workforceplanningmanagement.mobile.core

import kotlinx.serialization.Serializable
import java.io.IOException
import java.net.URLEncoder

/** The service's problem details. [detail] is shown to the person as the service wrote it. */
@Serializable
public data class ProblemDetails(
    val status: Int? = null,
    val code: String? = null,
    val title: String? = null,
    val detail: String? = null,
)

/** The outcome of a call. */
public sealed interface ApiResult<out T> {
    /** It worked. */
    public data class Ok<T>(val value: T) : ApiResult<T>

    /** 401: the token is missing, expired or revoked. The app signs the person out. */
    public data object SignedOut : ApiResult<Nothing>

    /** The service refused or failed, and said why (or not). */
    public data class Refused(val status: Int, val problem: ProblemDetails?) : ApiResult<Nothing>

    /** Nothing came back (no network, a timeout), or what came back could not be read. */
    public data class Unreachable(val unreadable: Boolean = false) : ApiResult<Nothing>
}

/** The client for one deployment and one token. */
public class WpmApi(private val baseUrl: String, private val token: String, private val transport: Transport) {
    /** The request headers every call sends. */
    public fun headers(json: Boolean = false): Map<String, String> = buildMap {
        put("Accept", "application/json")
        put(VERSION_HEADER, API_VERSION)
        put("Authorization", "Bearer $token")
        if (json) put("Content-Type", "application/json")
    }

    /** `GET /api/me/time-off?year=`. */
    public fun timeOffRequest(year: Int): HttpRequest =
        HttpRequest("GET", "$baseUrl/api/me/time-off?year=$year", headers())

    /** `POST /api/me/leave-requests`. */
    public fun requestLeaveRequest(payload: LeaveRequestPayload): HttpRequest =
        HttpRequest("POST", "$baseUrl/api/me/leave-requests", headers(json = true), Leave.encode(payload))

    /** `POST /api/me/leave-requests/{pid}/cancel`. */
    public fun cancelRequest(pid: String): HttpRequest =
        HttpRequest("POST", "$baseUrl/api/me/leave-requests/${URLEncoder.encode(pid, Charsets.UTF_8)}/cancel", headers())

    /** My time off for [year]. */
    public fun timeOff(year: Int): ApiResult<TimeOff> = call(timeOffRequest(year)) { decodeTimeOff(it) }

    /** Ask for leave. The service decides; its refusal comes back as [ApiResult.Refused]. */
    public fun requestLeave(payload: LeaveRequestPayload): ApiResult<Unit> = call(requestLeaveRequest(payload)) { }

    /** Cancel one of my requests. */
    public fun cancel(pid: String): ApiResult<Unit> = call(cancelRequest(pid)) { }

    private fun <T> call(request: HttpRequest, decode: (String) -> T): ApiResult<T> {
        val response = try {
            transport.send(request)
        } catch (_: IOException) {
            return ApiResult.Unreachable()
        }
        return interpret(response, decode)
    }

    public companion object {
        /** The service's version header and the version this app speaks. */
        public const val VERSION_HEADER: String = "Accepts-version"
        public const val API_VERSION: String = "1.0"

        /** Turns a response into a result. */
        public fun <T> interpret(response: HttpResponse, decode: (String) -> T): ApiResult<T> = when (response.status) {
            in 200..299 -> runCatching { ApiResult.Ok(decode(response.body)) }
                .getOrElse { ApiResult.Unreachable(unreadable = true) }
            401 -> ApiResult.SignedOut
            else -> ApiResult.Refused(response.status, problem(response.body))
        }

        /** The problem details in [body], or null when it holds none. */
        public fun problem(body: String): ProblemDetails? =
            runCatching { Json.decodeFromString(ProblemDetails.serializer(), body) }.getOrNull()
                ?.takeIf { it.detail != null || it.code != null || it.title != null }
    }
}

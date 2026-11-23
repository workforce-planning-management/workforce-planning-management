// The transport the core builds requests for. The app supplies one over HttpURLConnection; tests supply a fake.
package io.github.workforceplanningmanagement.mobile.core

/** A request, as data. */
public data class HttpRequest(
    val method: String,
    val url: String,
    val headers: Map<String, String> = emptyMap(),
    val body: String? = null,
)

/** A response, as data. */
public data class HttpResponse(val status: Int, val body: String)

/** Sends one request. Blocking; the app calls it off the main thread. Throws an IOException when nothing came back. */
public fun interface Transport {
    public fun send(request: HttpRequest): HttpResponse
}

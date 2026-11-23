// The platform's HttpURLConnection as the core's Transport (WPM-R280): no third-party network library.
package io.github.workforceplanningmanagement.mobile

import io.github.workforceplanningmanagement.mobile.core.HttpRequest
import io.github.workforceplanningmanagement.mobile.core.HttpResponse
import io.github.workforceplanningmanagement.mobile.core.Transport
import java.net.HttpURLConnection
import java.net.URL

class HttpUrlTransport(private val timeoutMillis: Int = 20_000) : Transport {
    override fun send(request: HttpRequest): HttpResponse {
        val connection = URL(request.url).openConnection() as HttpURLConnection
        try {
            connection.requestMethod = request.method
            connection.connectTimeout = timeoutMillis
            connection.readTimeout = timeoutMillis
            connection.useCaches = false
            connection.instanceFollowRedirects = false
            request.headers.forEach { (name, value) -> connection.setRequestProperty(name, value) }
            request.body?.let { body ->
                connection.doOutput = true
                connection.outputStream.use { it.write(body.toByteArray(Charsets.UTF_8)) }
            }
            val status = connection.responseCode
            val stream = if (status >= 400) connection.errorStream else connection.inputStream
            val text = stream?.use { it.readBytes().toString(Charsets.UTF_8) }.orEmpty()
            return HttpResponse(status, text)
        } finally {
            connection.disconnect()
        }
    }
}

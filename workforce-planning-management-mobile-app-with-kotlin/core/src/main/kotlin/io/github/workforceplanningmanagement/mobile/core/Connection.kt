// The deployment's address (WPM-R281): https, or http only for a developer's own machine. Checked before any request.
package io.github.workforceplanningmanagement.mobile.core

import java.net.URI
import java.net.URISyntaxException

/** Why an address was refused. The app turns each into its own words. */
public enum class AddressProblem { Empty, NotAnAddress, InsecureScheme, HasCredentials, HasQueryOrFragment }

/** The outcome of reading an address the person typed. */
public sealed interface AddressResult {
    /** A usable base URL, with no trailing slash, such as `https://wpm.example.org`. */
    public data class Accepted(val baseUrl: String) : AddressResult

    /** Refused before any request was sent. */
    public data class Refused(val problem: AddressProblem) : AddressResult
}

/** The connection rules. */
public object Connection {
    /** The hosts `http` is allowed for: this machine and the Android emulator's view of its host. */
    public val LOCAL_HOSTS: Set<String> = setOf("localhost", "127.0.0.1", "10.0.2.2")

    /** Reads [input] as a deployment address. Surrounding spaces do not matter; the scheme and host are case-insensitive. */
    public fun parse(input: String): AddressResult {
        val text = input.trim()
        if (text.isEmpty()) return AddressResult.Refused(AddressProblem.Empty)
        val uri = try {
            URI(text)
        } catch (_: URISyntaxException) {
            return AddressResult.Refused(AddressProblem.NotAnAddress)
        }
        val scheme = uri.scheme?.lowercase() ?: return AddressResult.Refused(AddressProblem.NotAnAddress)
        val host = uri.host?.lowercase()
        if (host.isNullOrEmpty() || uri.isOpaque) return AddressResult.Refused(AddressProblem.NotAnAddress)
        when (scheme) {
            "https" -> Unit
            "http" -> if (host !in LOCAL_HOSTS) return AddressResult.Refused(AddressProblem.InsecureScheme)
            else -> return AddressResult.Refused(AddressProblem.InsecureScheme)
        }
        if (uri.rawUserInfo != null) return AddressResult.Refused(AddressProblem.HasCredentials)
        if (uri.rawQuery != null || uri.rawFragment != null) return AddressResult.Refused(AddressProblem.HasQueryOrFragment)
        val port = if (uri.port == -1) "" else ":${uri.port}"
        val path = (uri.rawPath ?: "").trimEnd('/')
        return AddressResult.Accepted("$scheme://$host$port$path")
    }
}

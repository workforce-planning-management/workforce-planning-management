package io.github.workforceplanningmanagement.mobile.core

import org.junit.Assert.assertEquals
import org.junit.Test

class ConnectionTest {
    private fun ok(input: String) = (Connection.parse(input) as AddressResult.Accepted).baseUrl
    private fun refused(input: String) = (Connection.parse(input) as AddressResult.Refused).problem

    @Test fun httpsIsAcceptedAndNormalized() {
        assertEquals("https://wpm.example.org", ok("  https://wpm.example.org/  "))
        assertEquals("https://wpm.example.org", ok("HTTPS://WPM.Example.ORG"))
        assertEquals("https://wpm.example.org:8443/wpm", ok("https://wpm.example.org:8443/wpm//"))
    }

    @Test fun httpOnlyForTheDevelopersOwnMachine() {
        assertEquals("http://localhost:5150", ok("http://localhost:5150"))
        assertEquals("http://127.0.0.1:5150", ok("http://127.0.0.1:5150/"))
        assertEquals("http://10.0.2.2:5150", ok("http://10.0.2.2:5150"))
        assertEquals(AddressProblem.InsecureScheme, refused("http://wpm.example.org"))
        assertEquals(AddressProblem.InsecureScheme, refused("http://192.168.1.10:5150"))
        assertEquals(AddressProblem.InsecureScheme, refused("http://localhost.example.org"))
    }

    @Test fun otherSchemesAreRefused() {
        assertEquals(AddressProblem.InsecureScheme, refused("ftp://wpm.example.org"))
        assertEquals(AddressProblem.InsecureScheme, refused("wpm-mobile://signin"))
    }

    @Test fun whatIsNotAnAddressIsRefused() {
        assertEquals(AddressProblem.Empty, refused(""))
        assertEquals(AddressProblem.Empty, refused("   "))
        assertEquals(AddressProblem.NotAnAddress, refused("wpm.example.org"))
        assertEquals(AddressProblem.NotAnAddress, refused("https://"))
        assertEquals(AddressProblem.NotAnAddress, refused("https://exa mple.org"))
        assertEquals(AddressProblem.NotAnAddress, refused("mailto:hr@example.org"))
    }

    @Test fun credentialsQueriesAndFragmentsAreRefused() {
        assertEquals(AddressProblem.HasCredentials, refused("https://user:pass@wpm.example.org"))
        assertEquals(AddressProblem.HasQueryOrFragment, refused("https://wpm.example.org/?a=1"))
        assertEquals(AddressProblem.HasQueryOrFragment, refused("https://wpm.example.org/#top"))
    }
}

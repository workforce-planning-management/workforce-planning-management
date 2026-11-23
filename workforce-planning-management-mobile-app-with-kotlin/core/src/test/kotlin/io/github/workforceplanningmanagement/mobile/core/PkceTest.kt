package io.github.workforceplanningmanagement.mobile.core

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertThrows
import org.junit.Assert.assertTrue
import org.junit.Test

class PkceTest {
    @Test fun rfc7636AppendixB() {
        // The worked example of RFC 7636, Appendix B: this verifier gives this S256 challenge.
        val verifier = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk"
        assertTrue(Pkce.isVerifier(verifier))
        assertEquals("E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM", Pkce.challenge(verifier))
    }

    @Test fun aVerifierIsBase64UrlWithoutPadding() {
        assertEquals("AAECAwQFBgcICQoLDA0ODxAREhMUFRYXGBkaGxwdHh8", Pkce.verifier(ByteArray(32) { it.toByte() }))
        val v = Pkce.verifier(ByteArray(32) { 0xFB.toByte() })
        assertTrue("no +, / or = in base64url without padding", v.none { it in "+/=" })
        assertTrue(Pkce.isVerifier(v))
    }

    @Test fun aVerifierIs43CharactersFrom32Bytes() {
        val v = Pkce.verifier(ByteArray(32) { it.toByte() })
        assertEquals(43, v.length)
        assertTrue(Pkce.isVerifier(v))
    }

    @Test fun tooFewBytesAreRefused() {
        assertThrows(IllegalArgumentException::class.java) { Pkce.verifier(ByteArray(31)) }
    }

    @Test fun theVerifierAlphabetAndLength() {
        assertFalse(Pkce.isVerifier("a".repeat(42)))
        assertTrue(Pkce.isVerifier("a".repeat(43)))
        assertTrue(Pkce.isVerifier("A-._~".repeat(25)))
        assertFalse(Pkce.isVerifier("a".repeat(129)))
        assertFalse(Pkce.isVerifier("a".repeat(42) + "+"))
        assertFalse(Pkce.isVerifier("a".repeat(42) + "="))
        assertThrows(IllegalArgumentException::class.java) { Pkce.challenge("short") }
    }
}

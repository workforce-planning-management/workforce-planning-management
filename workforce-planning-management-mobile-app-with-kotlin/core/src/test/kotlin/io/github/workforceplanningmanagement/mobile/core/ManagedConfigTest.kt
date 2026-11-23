package io.github.workforceplanningmanagement.mobile.core

import org.junit.Assert.assertEquals
import org.junit.Test

class ManagedConfigTest {
    private val url = "https://wpm.example.org"
    private fun read(vararg pairs: Pair<String, Any?>) = ManagedConfig.read(mapOf(*pairs))

    @Test fun nothingSetIsNotAnError() {
        assertEquals(ManagedResult.Settings(null, false), ManagedConfig.read(null))
        assertEquals(ManagedResult.Settings(null, false), read())
        assertEquals(ManagedResult.Settings(null, false), read("other" to 1))
        assertEquals(ManagedResult.Settings(null, false), read("deployment_url" to null))
    }

    @Test fun anAddressAloneIsUsedAndNotLocked() {
        assertEquals(ManagedResult.Settings(url, false), read("deployment_url" to "$url/"))
    }

    @Test fun lockedNeedsAnAddress() {
        assertEquals(ManagedResult.Settings(url, true), read("deployment_url" to url, "deployment_locked" to true))
        assertEquals(ManagedResult.Refused(ManagedProblem.LockedWithoutUrl), read("deployment_locked" to true))
        assertEquals(ManagedResult.Settings(null, false), read("deployment_locked" to false))
    }

    @Test fun booleansAreBooleanOrTrueFalseText() {
        assertEquals(ManagedResult.Settings(url, true), read("deployment_url" to url, "deployment_locked" to "true"))
        assertEquals(ManagedResult.Settings(url, false), read("deployment_url" to url, "deployment_locked" to "false"))
        for (bad in listOf<Any>("yes", "TRUE", "1", 1, 0, 2.5, listOf("true"))) {
            assertEquals(
                "$bad",
                ManagedResult.Refused(ManagedProblem.DeploymentLocked),
                read("deployment_url" to url, "deployment_locked" to bad),
            )
        }
    }

    @Test fun aBadAddressIsRefusedAndNamed() {
        assertEquals(
            ManagedResult.Refused(ManagedProblem.DeploymentUrl(AddressProblem.InsecureScheme)),
            read("deployment_url" to "http://wpm.example.org"),
        )
        assertEquals(
            "supplied but empty is refused, not absent",
            ManagedResult.Refused(ManagedProblem.DeploymentUrl(AddressProblem.Empty)),
            read("deployment_url" to ""),
        )
        assertEquals(ManagedResult.Refused(ManagedProblem.DeploymentUrlNotText), read("deployment_url" to 5))
    }

    @Test fun aBadLockedFlagIsRefusedEvenWithAGoodAddress() {
        assertEquals(
            ManagedResult.Refused(ManagedProblem.DeploymentLocked),
            read("deployment_url" to url, "deployment_locked" to "maybe"),
        )
    }
}

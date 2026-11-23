// What a management tool pre-sets (WPM-R285): the deployment's address and whether it may be changed. Read from Android's
// application restrictions. Absent is not an error (the person types the address). Supplied but invalid is refused and
// named, never replaced by a default or by asking (RCA-IN-2, RCA-IN-4).
package io.github.workforceplanningmanagement.mobile.core

/** The setting that was wrong, and why. */
public sealed interface ManagedProblem {
    public data class DeploymentUrl(val why: AddressProblem) : ManagedProblem
    public data object DeploymentUrlNotText : ManagedProblem
    public data object DeploymentLocked : ManagedProblem

    /** `deployment_locked` is true and there is no `deployment_url`, which would leave no way to connect. */
    public data object LockedWithoutUrl : ManagedProblem
}

/** The outcome of reading the tool's values. */
public sealed interface ManagedResult {
    /** [baseUrl] is null when the tool set no address. */
    public data class Settings(val baseUrl: String?, val locked: Boolean) : ManagedResult

    public data class Refused(val problem: ManagedProblem) : ManagedResult
}

public object ManagedConfig {
    public const val URL_KEY: String = "deployment_url"
    public const val LOCKED_KEY: String = "deployment_locked"

    /** Reads [values]. A boolean is a `Boolean` or the text `true` or `false`; nothing else is a boolean. */
    public fun read(values: Map<String, Any?>?): ManagedResult {
        var baseUrl: String? = null
        values?.get(URL_KEY)?.let { raw ->
            val text = raw as? String ?: return ManagedResult.Refused(ManagedProblem.DeploymentUrlNotText)
            when (val r = Connection.parse(text)) {
                is AddressResult.Accepted -> baseUrl = r.baseUrl
                is AddressResult.Refused -> return ManagedResult.Refused(ManagedProblem.DeploymentUrl(r.problem))
            }
        }
        var locked = false
        values?.get(LOCKED_KEY)?.let { raw ->
            locked = when {
                raw is Boolean -> raw
                raw == "true" -> true
                raw == "false" -> false
                else -> return ManagedResult.Refused(ManagedProblem.DeploymentLocked)
            }
        }
        if (locked && baseUrl == null) return ManagedResult.Refused(ManagedProblem.LockedWithoutUrl)
        return ManagedResult.Settings(baseUrl, locked)
    }
}

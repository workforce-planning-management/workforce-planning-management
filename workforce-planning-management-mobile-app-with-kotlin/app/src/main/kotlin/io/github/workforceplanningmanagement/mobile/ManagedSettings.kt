// What a management tool pre-set for this app: Android's application restrictions (WPM-R285). Read-only; the app asks
// for no device-management permission and works the same when there are none (a personal device).
package io.github.workforceplanningmanagement.mobile

import android.content.Context
import android.content.RestrictionsManager

object ManagedSettings {
    /** The tool's values, or null when it set none. Only `String` and `Boolean` values are read. */
    fun read(context: Context): Map<String, Any?>? {
        val manager = context.getSystemService(Context.RESTRICTIONS_SERVICE) as? RestrictionsManager ?: return null
        val bundle = manager.applicationRestrictions
        if (bundle == null || bundle.isEmpty) return null
        return bundle.keySet().associateWith { key -> bundle.get(key) }
    }
}

// Workforce Planning Management for Android (spec/mobile-apps.md, WPM-R280–R284, WPM-T639).
pluginManagement {
    repositories {
        google()
        mavenCentral()
        gradlePluginPortal()
    }
}
dependencyResolutionManagement {
    repositoriesMode.set(RepositoriesMode.FAIL_ON_PROJECT_REPOS)
    repositories {
        google()
        mavenCentral()
    }
}
rootProject.name = "workforce-planning-management-mobile-app-with-kotlin"
include(":core", ":app")

// Lily Design System for Kotlin from a checkout of the Lily monorepo, until Lily publishes it (WPM-D226). The path is the
// `lily.dir` Gradle property, else LILY_DESIGN_SYSTEM_DIR, else a sibling checkout: ../../lilydesignsystem/lily-design-system
// from the root of this repository. Gradle substitutes io.github.lilydesignsystem:<module> with the included build's project.
val lilyDir: File = run {
    val given = providers.gradleProperty("lily.dir").orNull ?: providers.environmentVariable("LILY_DESIGN_SYSTEM_DIR").orNull
    if (given != null) file(given) else rootDir.resolve("../../../lilydesignsystem/lily-design-system").canonicalFile
}
val lilyKotlin = lilyDir.resolve("lily-design-system-kotlin")
require(lilyKotlin.resolve("settings.gradle.kts").isFile) {
    "Lily Design System for Kotlin not found at $lilyKotlin. Check out https://github.com/LilyDesignSystem/lily-design-system " +
        "and pass -Plily.dir=<that checkout> or set LILY_DESIGN_SYSTEM_DIR (spec/mobile-apps.md, WPM-D226)."
}
includeBuild(lilyKotlin)

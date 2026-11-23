// swift-tools-version: 6.0
// Workforce Planning Management for iPhone and iPad (spec/mobile-apps.md, WPM-R280–R284, WPM-T638).
//
// WPMCore is the pure core (WPM-D225): models, decoding, request building and rules, with no network, clock, storage or
// UI. WPMUI is the SwiftUI screens over Lily's LilyUI and LilyThemes. The app target lives in App/.
//
// Lily comes from a checkout of the Lily monorepo until Lily publishes its Swift package (WPM-D226): set
// LILY_DESIGN_SYSTEM_DIR to the monorepo's directory; the default is ../../lilydesignsystem/lily-design-system beside
// this repository's root.
import Foundation
import PackageDescription

let lilyMonorepo: String = {
    if let dir = Context.environment["LILY_DESIGN_SYSTEM_DIR"], !dir.isEmpty { return dir }
    return Context.packageDirectory + "/../../../lilydesignsystem/lily-design-system"
}()

let package = Package(
    name: "WorkforcePlanningManagementMobile",
    defaultLocalization: "en",
    platforms: [.iOS(.v18), .macOS(.v15)],
    products: [
        .library(name: "WPMCore", targets: ["WPMCore"]),
        .library(name: "WPMUI", targets: ["WPMUI"]),
    ],
    dependencies: [
        .package(name: "LilyDesignSystem", path: lilyMonorepo + "/lily-design-system-swift"),
    ],
    targets: [
        .target(name: "WPMCore"),
        .target(
            name: "WPMUI",
            dependencies: [
                "WPMCore",
                .product(name: "LilyUI", package: "LilyDesignSystem"),
                .product(name: "LilyThemes", package: "LilyDesignSystem"),
            ],
            resources: [.process("Resources")]
        ),
        .testTarget(name: "WPMCoreTests", dependencies: ["WPMCore"]),
        .testTarget(name: "WPMUITests", dependencies: ["WPMUI", "WPMCore"]),
    ],
    swiftLanguageModes: [.v6]
)

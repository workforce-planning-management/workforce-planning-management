// The app's one window: connect, sign in, then my time off (spec/mobile-apps.md). Light or dark follows the system
// through Lily's two gated themes (WPM-R284), and personal screens are hidden from the app switcher (WPM-D227).
import LilyThemes
import SwiftUI
import WPMCore
#if canImport(UIKit)
import UIKit
#endif

public struct RootView: View {
    @State private var session: Session
    @Environment(\.colorScheme) private var colorScheme

    public init(session: Session) {
        _session = State(initialValue: session)
    }

    public var body: some View {
        NavigationStack {
            switch session.phase {
            case .misconfigured: MisconfiguredView(problem: session.managedProblem)
            case .needsConnection: ConnectView(session: session)
            case .signedOut: SignInView(session: session)
            case .signedIn: TimeOffView(session: session)
            }
        }
        .lilyTheme(colorScheme == .dark ? .dark : .light)
        // Record text cannot be copied out of the app (WPM-R286): no text on any screen is selectable.
        .textSelection(.disabled)
        .privacyShield()
    }
}

/// Covers the window whenever the scene is not active, so the app switcher's snapshot shows no personal data, and while the
/// screen is being recorded or mirrored (WPM-R286). A screenshot cannot be stopped on iOS and nothing here claims to.
struct PrivacyShield: ViewModifier {
    @Environment(\.scenePhase) private var scenePhase
    @State private var captured = Self.screenIsCaptured()

    /// The one rule: hidden unless the scene is active and the screen is not being captured.
    static func hides(sceneActive: Bool, captured: Bool) -> Bool { !sceneActive || captured }

    static func screenIsCaptured() -> Bool {
        #if canImport(UIKit) && !os(watchOS)
        return UIScreen.main.isCaptured
        #else
        return false
        #endif
    }

    func body(content: Content) -> some View {
        content
            .overlay {
                if Self.hides(sceneActive: scenePhase == .active, captured: captured) {
                    ZStack {
                        Rectangle().fill(.background)
                        Text(L("app.name")).font(.title2)
                    }
                    .ignoresSafeArea()
                    .accessibilityHidden(true)
                }
            }
            #if canImport(UIKit) && !os(watchOS)
            .onReceive(NotificationCenter.default.publisher(for: UIScreen.capturedDidChangeNotification)) { _ in
                captured = Self.screenIsCaptured()
            }
            #endif
    }
}

extension View {
    func privacyShield() -> some View { modifier(PrivacyShield()) }
}

// The app's one window: connect, sign in, then my time off (spec/mobile-apps.md). Light or dark follows the system
// through Lily's two gated themes (WPM-R284), and personal screens are hidden from the app switcher (WPM-D227).
import LilyThemes
import SwiftUI
import WPMCore

public struct RootView: View {
    @State private var session: Session
    @Environment(\.colorScheme) private var colorScheme

    public init(session: Session) {
        _session = State(initialValue: session)
    }

    public var body: some View {
        NavigationStack {
            switch session.phase {
            case .needsConnection: ConnectView(session: session)
            case .signedOut: SignInView(session: session)
            case .signedIn: TimeOffView(session: session)
            }
        }
        .lilyTheme(colorScheme == .dark ? .dark : .light)
        .privacyShield()
    }
}

/// Covers the window whenever the scene is not active, so the app switcher's snapshot shows no personal data.
struct PrivacyShield: ViewModifier {
    @Environment(\.scenePhase) private var scenePhase

    func body(content: Content) -> some View {
        content.overlay {
            if scenePhase != .active {
                ZStack {
                    Rectangle().fill(.background)
                    Text(L("app.name")).font(.title2)
                }
                .ignoresSafeArea()
                .accessibilityHidden(true)
            }
        }
    }
}

extension View {
    func privacyShield() -> some View { modifier(PrivacyShield()) }
}

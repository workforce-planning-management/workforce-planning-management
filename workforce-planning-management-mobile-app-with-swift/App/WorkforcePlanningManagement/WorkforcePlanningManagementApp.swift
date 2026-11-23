// The iPhone and iPad app (spec/mobile-apps.md, WPM-T638): a window over WPMUI's RootView. Everything else is in the
// package one directory up, where it is tested with `swift test`.
import SwiftUI
import WPMUI

@main
struct WorkforcePlanningManagementApp: App {
    @State private var session = Session()

    var body: some Scene {
        WindowGroup {
            RootView(session: session)
        }
    }
}

// A management tool supplied a setting that is not valid (spec/mobile-apps.md, WPM-R285). The app stops and names the
// setting; it neither uses a default nor asks the person for an address in its place (RCA-IN-2, RCA-IN-4).
import LilyUI
import SwiftUI
import WPMCore

struct MisconfiguredView: View {
    let problem: ManagedConfiguration.Problem?

    var body: some View {
        Form {
            Section {
                ErrorSummary(title: L("error.title"), problems: [.init(Words.managed(problem)) {}])
            }
            Section {
                Text(L("managed.help"))
            }
        }
        .navigationTitle(L("managed.title"))
    }
}

// Connect to a deployment (spec/mobile-apps.md, WPM-R281): the address is checked before any request.
import LilyUI
import SwiftUI
import WPMCore

struct ConnectView: View {
    let session: Session
    @State private var address = ""
    @State private var problem: String?
    @AccessibilityFocusState private var fieldFocused: Bool

    var body: some View {
        Form {
            Section {
                Text(L("connect.intro"))
            }
            if let problem {
                ErrorSummary(title: L("error.title"), problems: [.init(problem) { fieldFocused = true }])
            }
            Section {
                Field(label: L("connect.address"), description: L("connect.address.hint"), error: problem,
                      required: true, requiredLabel: L("field.required")) {
                    TextField("", text: $address)
                        .textFieldStyle(.roundedBorder)
                        .textContentType(.URL)
                        .autocorrectionDisabled()
                        #if os(iOS)
                        .keyboardType(.URL)
                        .textInputAutocapitalization(.never)
                        #endif
                        .accessibilityFocused($fieldFocused)
                        .onSubmit(connect)
                }
            }
            Section {
                Button(L("connect.action"), action: connect)
                    .frame(minHeight: 44)
            }
        }
        .navigationTitle(L("connect.title"))
    }

    func connect() {
        problem = session.connect(address).map(Words.connection)
    }
}

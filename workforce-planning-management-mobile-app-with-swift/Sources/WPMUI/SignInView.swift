// Sign in through the system browser (spec/mobile-apps.md, WPM-R281, WPM-D224): ASWebAuthenticationSession, never an
// embedded web view; an authorization code with PKCE; the state checked; the token to the Keychain.
//
// Not verified: the sibling authentication service's native-client flow. A development build can take a pasted token.
import AuthenticationServices
import LilyUI
import Security
import SwiftUI
import WPMCore

struct SignInView: View {
    let session: Session
    @Environment(\.webAuthenticationSession) private var webAuthenticationSession
    @State private var problem: String?
    @State private var working = false
    #if DEBUG
    @State private var pasting = false
    #endif

    var body: some View {
        Form {
            if let notice = session.notice {
                Section { InsetText { Text(notice) } }
            }
            if let problem {
                ErrorSummary(title: L("error.title"), problems: [.init(problem) {}])
            }
            Section {
                Text(L("signIn.intro"))
                if let address = session.connection?.baseURL.absoluteString {
                    SummaryList(label: L("signIn.deployment")) {
                        SummaryListItem(term: L("signIn.address")) { Text(address) }
                    }
                }
            }
            Section {
                Button(L("signIn.action")) { Task { await signIn() } }
                    .disabled(working)
                    .frame(minHeight: 44)
                Button(L("signIn.change"), role: .destructive) { session.disconnect() }
                    .frame(minHeight: 44)
                #if DEBUG
                Button(L("developer.pasteToken")) { pasting = true }
                    .frame(minHeight: 44)
                #endif
            }
        }
        .navigationTitle(L("signIn.title"))
        #if DEBUG
        .sheet(isPresented: $pasting) { DeveloperTokenView(session: session) }
        #endif
    }

    func signIn() async {
        guard !working, let connection = session.connection else { return }
        working = true
        defer { working = false }
        problem = nil
        session.notice = nil
        let configuration = session.signInConfiguration
        guard let pkce = try? PKCE(randomBytes: randomBytes(32)) else { problem = L("signIn.failed"); return }
        let state = Base64URL.encode(randomBytes(16))
        let url = SignIn.authorizeURL(connection: connection, configuration: configuration, pkce: pkce, state: state)
        let callback: URL
        do {
            callback = try await webAuthenticationSession.authenticate(
                using: url, callbackURLScheme: SignInConfiguration.callbackScheme, preferredBrowserSession: .ephemeral)
        } catch {
            // The person closed the browser, or it could not open: nothing was sent.
            return
        }
        switch SignIn.code(from: callback, configuration: configuration, expectedState: state) {
        case .failure(.refused(_, let description)):
            problem = description ?? L("signIn.failed")
        case .failure:
            problem = L("signIn.failed")
        case .success(let code):
            await exchange(code: code, connection: connection, configuration: configuration, pkce: pkce)
        }
    }

    func exchange(code: String, connection: Connection, configuration: SignInConfiguration, pkce: PKCE) async {
        let request = SignIn.tokenRequest(connection: connection, configuration: configuration, code: code, pkce: pkce)
        do {
            let response = try await session.transport.send(request)
            try WPMClient.check(response)
            let token = try WPMClient.decode(SignIn.TokenResponse.self, from: response)
            session.signIn(token: token.accessToken)
        } catch let error as APIError {
            problem = Words.error(error)
        } catch {
            problem = Words.error(.unreachable)
        }
    }

    func randomBytes(_ count: Int) -> [UInt8] {
        var bytes = [UInt8](repeating: 0, count: count)
        let status = SecRandomCopyBytes(kSecRandomDefault, count, &bytes)
        precondition(status == errSecSuccess, "the system random generator failed")
        return bytes
    }
}

#if DEBUG
/// Development builds only (WPM-R281): a developer pastes a token minted for a local service. A release build has no
/// such screen.
struct DeveloperTokenView: View {
    let session: Session
    @Environment(\.dismiss) private var dismiss
    @State private var token = ""

    var body: some View {
        NavigationStack {
            Form {
                WarningText(prefix: L("warning.prefix"), text: L("developer.warning"))
                Field(label: L("developer.token")) {
                    SecureField("", text: $token)
                        .textFieldStyle(.roundedBorder)
                        .autocorrectionDisabled()
                }
                Button(L("developer.use")) {
                    session.signIn(token: token)
                    dismiss()
                }
                .disabled(token.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty)
                .frame(minHeight: 44)
            }
            .navigationTitle(L("developer.title"))
            .toolbar {
                ToolbarItem(placement: .cancellationAction) { Button(L("action.cancel")) { dismiss() } }
            }
        }
    }
}
#endif

import Foundation
import Testing
@testable import WPMCore
@testable import WPMUI

@MainActor
@Suite struct ManagedSessionTests {
    func make(managed: [String: Any]? = nil, saved: String? = nil, token: String? = nil, launched: Bool = true)
        -> (Session, InMemoryTokenStore, UserDefaults) {
        let defaults = UserDefaults(suiteName: "wpm-managed-\(UUID().uuidString)")!
        if let managed { defaults.set(managed, forKey: ManagedConfiguration.defaultsKey) }
        if let saved { defaults.set(saved, forKey: Session.addressKey) }
        if launched { defaults.set(true, forKey: Session.launchedKey) }
        let tokens = InMemoryTokenStore(token)
        return (Session(tokens: tokens, defaults: defaults, transport: StubTransport { _ in APIResponse(status: 200, body: Data()) }),
                tokens, defaults)
    }

    @Test func aPresetAddressSkipsTheConnectScreen() {
        let (s, _, defaults) = make(managed: ["deployment_url": "https://wpm.example.org"])
        #expect(s.phase == .signedOut)
        #expect(s.addressLocked == false)
        #expect(defaults.string(forKey: Session.addressKey) == "https://wpm.example.org")
    }

    @Test func aLockedAddressCannotBeChanged() {
        let (s, _, _) = make(managed: ["deployment_url": "https://wpm.example.org", "deployment_locked": true])
        #expect(s.addressLocked)
        #expect(s.connect("https://other.example.org") == nil)
        #expect(s.connection?.baseURL.absoluteString == "https://wpm.example.org")
        s.disconnect()
        #expect(s.connection != nil, "the address stays; only the token goes")
    }

    @Test func aChangedPresetAddressDropsTheTokenAndAnUnchangedOneKeepsIt() {
        let (kept, keptTokens, _) = make(managed: ["deployment_url": "https://wpm.example.org"],
                                         saved: "https://wpm.example.org", token: "t")
        #expect(kept.phase == .signedIn)
        #expect(keptTokens.read() == "t")
        let (moved, movedTokens, _) = make(managed: ["deployment_url": "https://new.example.org"],
                                           saved: "https://wpm.example.org", token: "t")
        #expect(moved.phase == .signedOut)
        #expect(movedTokens.read() == nil, "a token for another deployment is not kept")
    }

    @Test func anInvalidSettingStopsTheAppAndNamesIt() {
        let (s, tokens, _) = make(managed: ["deployment_url": "http://wpm.example.org"], saved: "https://wpm.example.org", token: "t")
        #expect(s.phase == .misconfigured, "no fallback to the saved address and no connect screen")
        #expect(s.connection == nil)
        #expect(Words.managed(s.managedProblem).contains("deployment_url"))
        #expect(tokens.read() == "t", "nothing is deleted because of a setting")
        let (b, _, _) = make(managed: ["deployment_url": "https://wpm.example.org", "deployment_locked": "maybe"])
        #expect(Words.managed(b.managedProblem).contains("deployment_locked"))
        let (c, _, _) = make(managed: ["deployment_locked": true])
        #expect(Words.managed(c.managedProblem).contains("deployment_url"))
    }

    @Test func noManagedConfigurationBehavesAsBefore() {
        let (s, _, _) = make()
        #expect(s.phase == .needsConnection)
        #expect(s.managedProblem == nil)
        #expect(s.addressLocked == false)
    }

    @Test func theFirstLaunchAfterAnInstallDropsATokenLeftBehind() {
        let (s, tokens, defaults) = make(saved: "https://wpm.example.org", token: "left-over", launched: false)
        #expect(tokens.read() == nil)
        #expect(s.phase == .signedOut)
        #expect(defaults.bool(forKey: Session.launchedKey))
        let (later, laterTokens, _) = make(saved: "https://wpm.example.org", token: "mine")
        #expect(later.phase == .signedIn)
        #expect(laterTokens.read() == "mine")
    }
}

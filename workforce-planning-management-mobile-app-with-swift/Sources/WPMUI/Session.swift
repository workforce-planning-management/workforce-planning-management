// Who the app is connected to and whether the person is signed in (spec/mobile-apps.md, WPM-R281).
import Foundation
import Observation
import WPMCore

@MainActor
@Observable
public final class Session {
    public enum Phase: Equatable { case misconfigured, needsConnection, signedOut, signedIn }

    public private(set) var connection: Connection?
    public private(set) var token: String?
    /// Shown once on the sign-in screen, such as "you were signed out".
    public var notice: String?
    public var signInConfiguration = SignInConfiguration()
    /// A setting a management tool supplied that is not valid (WPM-R285). The app stops on it and names it.
    public private(set) var managedProblem: ManagedConfiguration.Problem?
    /// True when a management tool set the address and asked that it not be changed (WPM-R285).
    public private(set) var addressLocked = false

    @ObservationIgnored let tokens: any TokenStore
    @ObservationIgnored let defaults: UserDefaults
    @ObservationIgnored public let transport: any Transport

    static let addressKey = "wpm.connection.address"
    static let launchedKey = "wpm.launched"

    public init(tokens: any TokenStore = KeychainTokenStore(), defaults: UserDefaults = .standard,
                transport: any Transport = URLSessionTransport()) {
        self.tokens = tokens
        self.defaults = defaults
        self.transport = transport
        // A Keychain item can outlive the app, so the first launch after an install drops any token left by an earlier one
        // (WPM-R285).
        if !defaults.bool(forKey: Self.launchedKey) {
            tokens.delete()
            defaults.set(true, forKey: Self.launchedKey)
        }
        var preset: Connection?
        switch ManagedConfiguration.read(defaults.dictionary(forKey: ManagedConfiguration.defaultsKey)) {
        case .failure(let problem):
            managedProblem = problem
            return
        case .success(let managed):
            preset = managed.connection
            addressLocked = managed.locked
        }
        let saved = defaults.string(forKey: Self.addressKey).flatMap { text -> Connection? in
            if case .success(let c) = Connection.parse(text) { return c }
            return nil
        }
        if let preset {
            if saved != preset { tokens.delete() } else { token = tokens.read() }
            connection = preset
            defaults.set(preset.baseURL.absoluteString, forKey: Self.addressKey)
        } else if let saved {
            connection = saved
            token = tokens.read()
        } else {
            tokens.delete()
        }
    }

    public var phase: Phase {
        if managedProblem != nil { return .misconfigured }
        if connection == nil { return .needsConnection }
        return token == nil ? .signedOut : .signedIn
    }

    /// Checks and keeps the address. Returns the problem when it is refused.
    @discardableResult
    public func connect(_ text: String) -> Connection.Problem? {
        if addressLocked { return nil }
        switch Connection.parse(text) {
        case .failure(let problem): return problem
        case .success(let c):
            if c != connection { tokens.delete(); token = nil }
            connection = c
            defaults.set(c.baseURL.absoluteString, forKey: Self.addressKey)
            return nil
        }
    }

    /// Forgets the deployment and the token.
    public func disconnect() {
        signOut()
        if addressLocked { return }
        connection = nil
        defaults.removeObject(forKey: Self.addressKey)
    }

    public func signIn(token newToken: String) {
        let trimmed = newToken.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmed.isEmpty else { return }
        token = trimmed
        if !tokens.write(trimmed) { notice = L("signIn.notSaved") }
    }

    /// Deletes the token (WPM-R281: sign out deletes it).
    public func signOut(notice: String? = nil) {
        tokens.delete()
        token = nil
        self.notice = notice
    }

    public var client: WPMClient? {
        guard let connection, let token else { return nil }
        return WPMClient(connection: connection, token: token, transport: transport)
    }

    /// A 401 signs the person out and says so; anything else is the caller's to show.
    public func handle(_ error: APIError) {
        if error == .unauthorized { signOut(notice: L("error.signedOut")) }
    }
}

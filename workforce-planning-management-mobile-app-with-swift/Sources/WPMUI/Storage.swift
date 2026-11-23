// What the app keeps (spec/mobile-apps.md, WPM-R281, WPM-D227): the deployment's address in preferences and the token
// in the Keychain, readable only while the device is unlocked and never moved to another device or a backup. Nothing
// else is stored; the network session is ephemeral, so no answer is cached on disk.
import Foundation
import Security
import WPMCore

public protocol TokenStore: Sendable {
    func read() -> String?
    func write(_ token: String) -> Bool
    func delete()
}

public struct KeychainTokenStore: TokenStore {
    let service: String
    let account = "token"

    public init(service: String = "org.workforceplanningmanagement.mobile") {
        self.service = service
    }

    var query: [String: Any] {
        [kSecClass as String: kSecClassGenericPassword,
         kSecAttrService as String: service,
         kSecAttrAccount as String: account]
    }

    public func read() -> String? {
        var q = query
        q[kSecReturnData as String] = true
        q[kSecMatchLimit as String] = kSecMatchLimitOne
        var item: CFTypeRef?
        guard SecItemCopyMatching(q as CFDictionary, &item) == errSecSuccess, let data = item as? Data else { return nil }
        return String(data: data, encoding: .utf8)
    }

    public func write(_ token: String) -> Bool {
        delete()
        var q = query
        q[kSecValueData as String] = Data(token.utf8)
        q[kSecAttrAccessible as String] = kSecAttrAccessibleWhenUnlockedThisDeviceOnly
        return SecItemAdd(q as CFDictionary, nil) == errSecSuccess
    }

    public func delete() {
        SecItemDelete(query as CFDictionary)
    }
}

/// For previews and tests: holds the token in memory only.
public final class InMemoryTokenStore: TokenStore, @unchecked Sendable {
    private let lock = NSLock()
    private var token: String?

    public init(_ token: String? = nil) { self.token = token }

    public func read() -> String? { lock.withLock { token } }
    public func write(_ token: String) -> Bool { lock.withLock { self.token = token }; return true }
    public func delete() { lock.withLock { token = nil } }
}

/// The real transport: an ephemeral `URLSession` (no cache, no cookies on disk).
public struct URLSessionTransport: Transport {
    let session: URLSession

    public init() {
        let configuration = URLSessionConfiguration.ephemeral
        configuration.requestCachePolicy = .reloadIgnoringLocalCacheData
        configuration.urlCache = nil
        configuration.httpCookieStorage = nil
        configuration.timeoutIntervalForRequest = 30
        session = URLSession(configuration: configuration)
    }

    public func send(_ request: APIRequest) async throws -> APIResponse {
        var r = URLRequest(url: request.url)
        r.httpMethod = request.method
        for (name, value) in request.headers { r.setValue(value, forHTTPHeaderField: name) }
        r.httpBody = request.body
        let (data, response) = try await session.data(for: r)
        guard let http = response as? HTTPURLResponse else { throw URLError(.badServerResponse) }
        return APIResponse(status: http.statusCode, body: data)
    }
}

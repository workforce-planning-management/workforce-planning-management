// Which deployment the app talks to (spec/mobile-apps.md, WPM-R281). An `https` address; `http` only for a developer's
// own machine (`localhost`, `127.0.0.1`, and the Android emulator's host `10.0.2.2`). Checked before any request.
import Foundation

public struct Connection: Equatable, Sendable {
    /// The deployment's base address, with no trailing slash, query or fragment.
    public let baseURL: URL

    public static let localHosts: Set<String> = ["localhost", "127.0.0.1", "10.0.2.2"]

    public enum Problem: Error, Equatable, Sendable {
        case empty
        case notAnAddress
        case insecureScheme
        case credentialsInAddress
        case queryOrFragment
    }

    /// Checks what the person typed. Surrounding spaces are ignored; nothing else is guessed (no scheme is added).
    public static func parse(_ text: String) -> Result<Connection, Problem> {
        let trimmed = text.trimmingCharacters(in: .whitespacesAndNewlines)
        if trimmed.isEmpty { return .failure(.empty) }
        guard let parts = URLComponents(string: trimmed), let scheme = parts.scheme?.lowercased(),
              let host = parts.host?.lowercased(), !host.isEmpty
        else { return .failure(.notAnAddress) }
        switch scheme {
        case "https": break
        case "http" where localHosts.contains(host): break
        case "http": return .failure(.insecureScheme)
        default: return .failure(.notAnAddress)
        }
        if parts.user != nil || parts.password != nil { return .failure(.credentialsInAddress) }
        if parts.query != nil || parts.fragment != nil { return .failure(.queryOrFragment) }
        var clean = URLComponents()
        clean.scheme = scheme
        clean.host = host
        clean.port = parts.port
        var path = parts.path
        while path.hasSuffix("/") { path.removeLast() }
        clean.path = path
        guard let url = clean.url else { return .failure(.notAnAddress) }
        return .success(Connection(baseURL: url))
    }

    /// `path` (percent-encoded, starting with `/`) under the base address, with `query` in order.
    public func url(_ path: String, query: [URLQueryItem] = []) -> URL {
        var parts = URLComponents(url: baseURL, resolvingAgainstBaseURL: false) ?? URLComponents()
        parts.percentEncodedPath = parts.percentEncodedPath + path
        parts.queryItems = query.isEmpty ? nil : query
        // The base is a checked URL and `path` is the app's own, so this does not fail.
        return parts.url ?? baseURL
    }
}

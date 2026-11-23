// Sign-in as a native public client (spec/mobile-apps.md, WPM-R281, WPM-D224): an authorization code with PKCE
// (RFC 7636, S256) through the system browser, as RFC 8252 asks. The randomness is the caller's; everything here is pure.
//
// Not verified: the sibling authentication service's support for this flow. Its paths are configuration.
import CryptoKit
import Foundation

public enum Base64URL {
    /// Base64url with no padding (RFC 4648 §5), as PKCE uses.
    public static func encode(_ bytes: some Sequence<UInt8>) -> String {
        Data(bytes).base64EncodedString()
            .replacingOccurrences(of: "+", with: "-")
            .replacingOccurrences(of: "/", with: "_")
            .replacingOccurrences(of: "=", with: "")
    }
}

public struct PKCE: Equatable, Sendable {
    public let verifier: String

    public enum Problem: Error, Equatable, Sendable {
        case badVerifier
    }

    /// A verifier must be 43 to 128 characters of `[A-Z] [a-z] [0-9] - . _ ~` (RFC 7636 §4.1).
    public init(verifier: String) throws(Problem) {
        let allowed = Set("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-._~")
        guard (43...128).contains(verifier.count), verifier.allSatisfy(allowed.contains) else { throw .badVerifier }
        self.verifier = verifier
    }

    /// A verifier from 32 random bytes (43 characters). The caller supplies the bytes from the system's generator.
    public init(randomBytes: [UInt8]) throws(Problem) {
        try self.init(verifier: Base64URL.encode(randomBytes))
    }

    /// `BASE64URL(SHA256(ASCII(verifier)))`.
    public var challenge: String {
        Base64URL.encode(SHA256.hash(data: Data(verifier.utf8)))
    }

    public static let method = "S256"
}

/// Where and how to sign in. The paths default to the authentication service's documented shape and can be changed per
/// deployment, because this repository cannot check them.
public struct SignInConfiguration: Equatable, Sendable {
    public var authorizePath: String
    public var tokenPath: String
    public var clientID: String
    public var redirectURI: String
    public var scope: String

    public static let callbackScheme = "wpm-mobile"

    public init(authorizePath: String = "/api/auth/authorize", tokenPath: String = "/api/auth/token",
                clientID: String = "wpm-mobile", redirectURI: String = "wpm-mobile://signin",
                scope: String = "openid") {
        self.authorizePath = authorizePath
        self.tokenPath = tokenPath
        self.clientID = clientID
        self.redirectURI = redirectURI
        self.scope = scope
    }
}

public enum SignIn {
    /// The address to open in the system browser.
    public static func authorizeURL(connection: Connection, configuration: SignInConfiguration, pkce: PKCE,
                                    state: String) -> URL {
        connection.url(configuration.authorizePath, query: [
            URLQueryItem(name: "response_type", value: "code"),
            URLQueryItem(name: "client_id", value: configuration.clientID),
            URLQueryItem(name: "redirect_uri", value: configuration.redirectURI),
            URLQueryItem(name: "scope", value: configuration.scope),
            URLQueryItem(name: "state", value: state),
            URLQueryItem(name: "code_challenge", value: pkce.challenge),
            URLQueryItem(name: "code_challenge_method", value: PKCE.method),
        ])
    }

    public enum CallbackProblem: Error, Equatable, Sendable {
        case wrongRedirect
        case stateMismatch
        case refused(error: String, description: String?)
        case noCode
    }

    /// Reads the browser's redirect: the state must be the one sent, and the address the app's own redirect.
    public static func code(from callback: URL, configuration: SignInConfiguration,
                            expectedState: String) -> Result<String, CallbackProblem> {
        guard let parts = URLComponents(url: callback, resolvingAgainstBaseURL: false),
              let expected = URLComponents(string: configuration.redirectURI),
              parts.scheme?.lowercased() == expected.scheme?.lowercased(),
              (parts.host ?? "") == (expected.host ?? ""), parts.path == expected.path
        else { return .failure(.wrongRedirect) }
        let items = parts.queryItems ?? []
        func value(_ name: String) -> String? { items.first { $0.name == name }?.value }
        guard value("state") == expectedState else { return .failure(.stateMismatch) }
        if let error = value("error") { return .failure(.refused(error: error, description: value("error_description"))) }
        guard let code = value("code"), !code.isEmpty else { return .failure(.noCode) }
        return .success(code)
    }

    /// The token request: a form POST that proves the verifier.
    public static func tokenRequest(connection: Connection, configuration: SignInConfiguration, code: String,
                                    pkce: PKCE) -> APIRequest {
        let fields: [(String, String)] = [
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", configuration.redirectURI),
            ("client_id", configuration.clientID),
            ("code_verifier", pkce.verifier),
        ]
        let body = fields.map { "\(formEncode($0.0))=\(formEncode($0.1))" }.joined(separator: "&")
        return APIRequest(
            method: "POST", url: connection.url(configuration.tokenPath),
            headers: ["Accept": "application/json", "Content-Type": "application/x-www-form-urlencoded"],
            body: Data(body.utf8))
    }

    /// `application/x-www-form-urlencoded` for one value: unreserved characters kept, everything else percent-encoded.
    public static func formEncode(_ text: String) -> String {
        var allowed = CharacterSet.alphanumerics.intersection(CharacterSet(charactersIn: Unicode.Scalar(0)...Unicode.Scalar(127)))
        allowed.insert(charactersIn: "-._~")
        return text.addingPercentEncoding(withAllowedCharacters: allowed) ?? text
    }

    public struct TokenResponse: Codable, Equatable, Sendable {
        public let accessToken: String
        public let tokenType: String?

        enum CodingKeys: String, CodingKey {
            case accessToken = "access_token"
            case tokenType = "token_type"
        }
    }
}

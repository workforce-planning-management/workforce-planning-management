import Foundation
import Testing
@testable import WPMCore

@Suite struct ConnectionTests {
    @Test(arguments: [
        ("https://wpm.example.org", "https://wpm.example.org"),
        ("  https://WPM.Example.org/  ", "https://wpm.example.org"),
        ("https://example.org/wpm/", "https://example.org/wpm"),
        ("https://example.org:8443", "https://example.org:8443"),
        ("HTTPS://example.org", "https://example.org"),
        ("http://localhost:5150", "http://localhost:5150"),
        ("http://127.0.0.1:5150", "http://127.0.0.1:5150"),
        ("http://10.0.2.2:5150", "http://10.0.2.2:5150"),
    ])
    func accepts(_ text: String, _ expected: String) throws {
        let connection = try Connection.parse(text).get()
        #expect(connection.baseURL.absoluteString == expected)
    }

    @Test(arguments: [
        ("", Connection.Problem.empty),
        ("   ", .empty),
        ("wpm.example.org", .notAnAddress),
        ("ftp://example.org", .notAnAddress),
        ("https://", .notAnAddress),
        ("http://wpm.example.org", .insecureScheme),
        ("http://192.168.1.10", .insecureScheme),
        ("http://localhost.example.org", .insecureScheme),
        ("https://user:pass@example.org", .credentialsInAddress),
        ("https://example.org/?a=1", .queryOrFragment),
        ("https://example.org/#top", .queryOrFragment),
        ("javascript:alert(1)", .notAnAddress),
    ])
    func refuses(_ text: String, _ problem: Connection.Problem) {
        #expect(Connection.parse(text) == .failure(problem))
    }

    @Test func pathsAndQueriesAreAddedUnderTheBase() throws {
        let c = try Connection.parse("https://example.org/wpm").get()
        #expect(c.url("/api/me/time-off", query: [URLQueryItem(name: "year", value: "2026")]).absoluteString
            == "https://example.org/wpm/api/me/time-off?year=2026")
    }
}

@Suite struct PKCETests {
    // RFC 7636 Appendix B.
    static let rfcBytes: [UInt8] = [116, 24, 223, 180, 151, 153, 224, 37, 79, 250, 96, 125, 216, 173, 187, 186, 22, 212,
                                    37, 77, 105, 214, 191, 240, 91, 88, 5, 88, 83, 132, 141, 121]

    @Test func rfc7636AppendixBVector() throws {
        let pkce = try PKCE(randomBytes: Self.rfcBytes)
        #expect(pkce.verifier == "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk")
        #expect(pkce.challenge == "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM")
        #expect(PKCE.method == "S256")
    }

    @Test func verifierShapeIsChecked() {
        #expect(throws: PKCE.Problem.badVerifier) { try PKCE(verifier: String(repeating: "a", count: 42)) }
        #expect(throws: PKCE.Problem.badVerifier) { try PKCE(verifier: String(repeating: "a", count: 129)) }
        #expect(throws: PKCE.Problem.badVerifier) { try PKCE(verifier: String(repeating: "a", count: 42) + "+") }
        #expect(throws: PKCE.Problem.badVerifier) { try PKCE(randomBytes: [1, 2, 3]) }
        #expect((try? PKCE(verifier: String(repeating: "a", count: 128))) != nil)
        #expect((try? PKCE(verifier: String(repeating: "-._~", count: 11))) != nil)
    }

    @Test func base64URLHasNoPaddingOrUnsafeCharacters() {
        #expect(Base64URL.encode([0xfb, 0xff]) == "-_8")
        #expect(Base64URL.encode([]) == "")
    }
}

@Suite struct SignInTests {
    let connection = try! Connection.parse("https://wpm.example.org").get()
    let configuration = SignInConfiguration()
    let pkce = try! PKCE(randomBytes: PKCETests.rfcBytes)

    @Test func authorizeURLCarriesCodeFlowAndPKCE() throws {
        let url = SignIn.authorizeURL(connection: connection, configuration: configuration, pkce: pkce, state: "s1")
        let parts = try #require(URLComponents(url: url, resolvingAgainstBaseURL: false))
        #expect(parts.path == "/api/auth/authorize")
        let items = Dictionary(uniqueKeysWithValues: (parts.queryItems ?? []).map { ($0.name, $0.value ?? "") })
        #expect(items == [
            "response_type": "code", "client_id": "wpm-mobile", "redirect_uri": "wpm-mobile://signin",
            "scope": "openid", "state": "s1", "code_challenge": "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM",
            "code_challenge_method": "S256",
        ])
        #expect(!url.absoluteString.contains(pkce.verifier), "the verifier never leaves the app before the exchange")
    }

    @Test func callbackGivesTheCode() {
        let url = URL(string: "wpm-mobile://signin?code=abc&state=s1")!
        #expect(SignIn.code(from: url, configuration: configuration, expectedState: "s1") == .success("abc"))
    }

    @Test(arguments: [
        ("wpm-mobile://signin?code=abc&state=other", SignIn.CallbackProblem.stateMismatch),
        ("wpm-mobile://signin?code=abc", .stateMismatch),
        ("wpm-mobile://signin?state=s1", .noCode),
        ("wpm-mobile://signin?state=s1&code=", .noCode),
        ("wpm-mobile://elsewhere?code=abc&state=s1", .wrongRedirect),
        ("https://signin?code=abc&state=s1", .wrongRedirect),
        ("wpm-mobile://signin?state=s1&error=access_denied&error_description=No", .refused(error: "access_denied", description: "No")),
    ])
    func callbackProblems(_ text: String, _ problem: SignIn.CallbackProblem) {
        #expect(SignIn.code(from: URL(string: text)!, configuration: configuration, expectedState: "s1") == .failure(problem))
    }

    @Test func aRefusalWithAWrongStateIsAStateMismatch() {
        let url = URL(string: "wpm-mobile://signin?state=forged&error=access_denied")!
        #expect(SignIn.code(from: url, configuration: configuration, expectedState: "s1") == .failure(.stateMismatch))
    }

    @Test func tokenRequestIsAFormPostWithTheVerifier() throws {
        let r = SignIn.tokenRequest(connection: connection, configuration: configuration, code: "a b&c", pkce: pkce)
        #expect(r.method == "POST")
        #expect(r.url.absoluteString == "https://wpm.example.org/api/auth/token")
        #expect(r.headers["Content-Type"] == "application/x-www-form-urlencoded")
        #expect(r.headers["Authorization"] == nil)
        let body = String(decoding: try #require(r.body), as: UTF8.self)
        #expect(body == "grant_type=authorization_code&code=a%20b%26c&redirect_uri=wpm-mobile%3A%2F%2Fsignin"
            + "&client_id=wpm-mobile&code_verifier=dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk")
    }

    @Test func tokenResponseDecodes() throws {
        let data = Data(#"{"access_token":"v4.public.x","token_type":"Bearer","expires_in":3600}"#.utf8)
        let token = try JSONDecoder().decode(SignIn.TokenResponse.self, from: data)
        #expect(token.accessToken == "v4.public.x")
    }

    @Test func pathsAreConfiguration() throws {
        var custom = configuration
        custom.authorizePath = "/sso/authorize"
        let url = SignIn.authorizeURL(connection: connection, configuration: custom, pkce: pkce, state: "s")
        #expect(url.path == "/sso/authorize")
    }
}

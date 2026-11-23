import Foundation
import SwiftUI
import Testing
@testable import WPMCore
@testable import WPMUI

private let sourcesDir = URL(fileURLWithPath: #filePath)
    .deletingLastPathComponent().deletingLastPathComponent().deletingLastPathComponent()
    .appendingPathComponent("Sources/WPMUI")

private func catalog() throws -> [String: Any] {
    let data = try Data(contentsOf: sourcesDir.appendingPathComponent("Resources/Localizable.xcstrings"))
    let root = try #require(try JSONSerialization.jsonObject(with: data) as? [String: Any])
    return try #require(root["strings"] as? [String: Any])
}

@Suite struct StringsTests {
    /// Strings are content (WPM-R284): every key the code asks for is in the catalog with an English value.
    @Test func everyKeyInCodeIsInTheCatalog() throws {
        let strings = try catalog()
        let files = try FileManager.default.contentsOfDirectory(at: sourcesDir, includingPropertiesForKeys: nil)
            .filter { $0.pathExtension == "swift" }
        let pattern = try Regex(#"L\("([A-Za-z.]+)""#)
        var used: Set<String> = []
        for file in files {
            let text = try String(contentsOf: file, encoding: .utf8)
            for match in text.matches(of: pattern) {
                if let key = match.output[1].substring { used.insert(String(key)) }
            }
        }
        #expect(used.count > 50)
        for key in used { #expect(strings[key] != nil, "missing \(key)") }
        for kind in LeaveRules.kinds { #expect(strings["kind.\(kind)"] != nil, "missing kind.\(kind)") }
        for status in LeaveRules.statuses { #expect(strings["status.\(status)"] != nil, "missing status.\(status)") }
    }

    @Test func noCatalogEntryIsUnused() throws {
        let strings = try catalog()
        var text = ""
        for file in try FileManager.default.contentsOfDirectory(at: sourcesDir, includingPropertiesForKeys: nil)
        where file.pathExtension == "swift" {
            text += try String(contentsOf: file, encoding: .utf8)
        }
        for key in strings.keys where !key.hasPrefix("kind.") && !key.hasPrefix("status.") {
            #expect(text.contains("\"\(key)\""), "unused \(key)")
        }
    }

    @Test func wordsResolveFromTheBundle() {
        #expect(Words.kind("annual") == "Annual leave")
        #expect(Words.kind("sabbatical") == "sabbatical", "an unknown kind is shown as sent, never as a key")
        #expect(Words.status("approved") == "Approved")
        #expect(Words.figure(.notRecorded) == "Not recorded")
        #expect(Words.days(1) == "1 calendar day")
        #expect(Words.days(5) == "5 calendar days")
        #expect(Words.figure(.days(0)) == "0 calendar days")
    }

    @Test func errorsUseTheServicesWords() {
        #expect(Words.error(.refused(status: 422, problem: ProblemDetails(detail: "over balance"))) == "over balance")
        #expect(Words.error(.refused(status: 500, problem: nil)) == "The service refused the request.")
    }

    @Test func aDateIsTheCalendarDateWithNoShift() {
        let text = Words.date(CalendarDate(iso: "2026-01-01")!)
        #expect(text.contains("2026") || text.contains("26"))
    }
}

@MainActor
@Suite struct SessionTests {
    func session(token: String? = nil, transport: any Transport = StubTransport { _ in APIResponse(status: 200, body: Data()) })
        -> (Session, InMemoryTokenStore, UserDefaults) {
        let defaults = UserDefaults(suiteName: "wpm-test-\(UUID().uuidString)")!
        let tokens = InMemoryTokenStore(token)
        return (Session(tokens: tokens, defaults: defaults, transport: transport), tokens, defaults)
    }

    @Test func connectThenSignInThenSignOut() {
        let (s, tokens, defaults) = session()
        #expect(s.phase == .needsConnection)
        #expect(s.connect("http://wpm.example.org") == .insecureScheme)
        #expect(s.phase == .needsConnection)
        #expect(s.connect("https://wpm.example.org") == nil)
        #expect(defaults.string(forKey: Session.addressKey) == "https://wpm.example.org")
        #expect(s.phase == .signedOut)
        s.signIn(token: "  v4.public.t \n")
        #expect(s.phase == .signedIn)
        #expect(tokens.read() == "v4.public.t")
        s.signOut()
        #expect(tokens.read() == nil, "sign out deletes the token")
        #expect(s.phase == .signedOut)
    }

    @Test func aTokenWithNoSavedAddressIsDeleted() {
        let (s, tokens, _) = session(token: "stale")
        #expect(s.phase == .needsConnection)
        #expect(tokens.read() == nil)
    }

    @Test func changingTheAddressDeletesTheToken() {
        let (s, tokens, _) = session()
        s.connect("https://a.example.org")
        s.signIn(token: "t")
        s.connect("https://b.example.org")
        #expect(tokens.read() == nil)
        #expect(s.phase == .signedOut)
    }

    @Test func a401SignsOutAndSaysSo() {
        let (s, tokens, _) = session()
        s.connect("https://a.example.org")
        s.signIn(token: "t")
        s.handle(.unauthorized)
        #expect(tokens.read() == nil)
        #expect(s.notice == "You have been signed out. Sign in again to continue.")
        s.signIn(token: "t2")
        s.handle(.unreachable)
        #expect(s.phase == .signedIn, "only a 401 signs out")
    }

    @Test func disconnectForgetsBoth() {
        let (s, tokens, defaults) = session()
        s.connect("https://a.example.org")
        s.signIn(token: "t")
        s.disconnect()
        #expect(tokens.read() == nil)
        #expect(defaults.string(forKey: Session.addressKey) == nil)
        #expect(s.phase == .needsConnection)
    }

    @Test func theTimeOffModelLoadsCancelsAndSignsOutOn401() async throws {
        let calls = Counter()
        let transport = StubTransport { request in
            calls.increment()
            if request.url.path.hasSuffix("/cancel") { return APIResponse(status: 401, body: Data()) }
            return APIResponse(status: 200, body: Data(timeOffJSON.utf8))
        }
        let (s, _, _) = session(transport: transport)
        s.connect("https://a.example.org")
        s.signIn(token: "t")
        let instant = Date(timeIntervalSince1970: 1_795_430_000) // 2026-11-23 in every time zone near UTC
        let model = TimeOffModel(session: s, now: { instant })
        await model.load()
        guard case .loaded(let timeOff) = model.state else { Issue.record("not loaded"); return }
        #expect(model.year == 2026)
        #expect(!model.canCancel(timeOff.requests[0]), "approved and over")
        #expect(model.canCancel(timeOff.requests[2]), "awaiting a decision")
        await model.cancel(timeOff.requests[0])
        #expect(calls.value == 1, "a request that may not be cancelled is never sent")
        await model.cancel(timeOff.requests[2])
        #expect(s.phase == .signedOut)
    }

    @Test func aDraftWithAProblemIsNotSent() async {
        let calls = Counter()
        let (s, _, _) = session(transport: StubTransport { _ in calls.increment(); return APIResponse(status: 200, body: Data()) })
        s.connect("https://a.example.org")
        s.signIn(token: "t")
        let model = TimeOffModel(session: s)
        let problem = await model.submit(LeaveDraft(kind: "annual", startOn: CalendarDate(iso: "2026-07-03")!,
                                                    endOn: CalendarDate(iso: "2026-07-01")!))
        #expect(problem == "The last day must be on or after the first day.")
        #expect(calls.value == 0)
    }

    @Test func theTimeOffScreenRenders() throws {
        let (s, _, _) = session()
        s.connect("https://a.example.org")
        s.signIn(token: "t")
        let timeOff = try JSONDecoder().decode(TimeOff.self, from: Data(timeOffJSON.utf8))
        let model = TimeOffModel(session: s)
        let view = TimeOffContent(timeOff: timeOff, model: model) { _ in }.frame(width: 390, height: 1600)
        let renderer = ImageRenderer(content: view)
        #expect(renderer.cgImage != nil)
    }
}

final class Counter: @unchecked Sendable {
    private let lock = NSLock()
    private var n = 0
    func increment() { lock.withLock { n += 1 } }
    var value: Int { lock.withLock { n } }
}

final class StubTransport: Transport, @unchecked Sendable {
    let respond: @Sendable (APIRequest) throws -> APIResponse
    init(_ respond: @escaping @Sendable (APIRequest) throws -> APIResponse) { self.respond = respond }
    func send(_ request: APIRequest) async throws -> APIResponse { try respond(request) }
}

let timeOffJSON = #"""
{"unit":"calendar_days","as_of":"2026-11-23","year":2026,
 "balances":[{"kind":"annual","year":2026,"entitled":25,"used":10,"taken":8,"booked":2,"requested":3,"remaining":15,"remaining_if_requested_approved":12},
             {"kind":"unpaid","year":2026,"entitled":null,"used":0,"taken":0,"booked":0,"requested":1,"remaining":null,"remaining_if_requested_approved":null}],
 "requests":[{"pid":"p1","kind":"annual","start_on":"2026-08-03","end_on":"2026-08-07","days":5,"status":"approved","reason":"Family visit"},
             {"pid":"p2","kind":"sick","start_on":"2026-09-14","end_on":"2026-09-15","days":2,"status":"approved","reason":null},
             {"pid":"p3","kind":"annual","start_on":"2026-12-21","end_on":"2026-12-23","days":3,"status":"requested","reason":null}],
 "history":[{"year":2025,"absences":[{"kind":"annual","absences":4,"days":22}]}],
 "derivation":"remaining = days allowed less the days already subtracted for approved requests."}
"""#

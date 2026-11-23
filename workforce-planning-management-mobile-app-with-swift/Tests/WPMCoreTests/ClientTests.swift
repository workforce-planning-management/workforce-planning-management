import Foundation
import Testing
@testable import WPMCore

/// A realistic answer from `GET /api/me/time-off`, in the shape `my_time_off` builds.
let timeOffJSON = #"""
{
  "unit": "calendar_days",
  "as_of": "2026-11-23",
  "year": 2026,
  "balances": [
    {"kind": "annual", "year": 2026, "entitled": 25, "used": 10, "taken": 8, "booked": 2, "requested": 3,
     "remaining": 15, "remaining_if_requested_approved": 12},
    {"kind": "unpaid", "year": 2026, "entitled": null, "used": 0, "taken": 0, "booked": 0, "requested": 1,
     "remaining": null, "remaining_if_requested_approved": null}
  ],
  "requests": [
    {"pid": "0f9f3a52-1111-4000-8000-000000000001", "kind": "annual", "start_on": "2026-08-03",
     "end_on": "2026-08-07", "days": 5, "status": "approved", "reason": "Family visit"},
    {"pid": "0f9f3a52-1111-4000-8000-000000000002", "kind": "sick", "start_on": "2026-09-14",
     "end_on": "2026-09-15", "days": 2, "status": "approved", "reason": null},
    {"pid": "0f9f3a52-1111-4000-8000-000000000003", "kind": "annual", "start_on": "2026-12-21",
     "end_on": "2026-12-23", "days": 3, "status": "requested", "reason": null}
  ],
  "history": [
    {"year": 2025, "absences": [{"kind": "annual", "absences": 4, "days": 22}, {"kind": "sick", "absences": 1, "days": 3}]}
  ],
  "derivation": "remaining = days allowed less the days the service has already subtracted for approved requests."
}
"""#

final class StubTransport: Transport, @unchecked Sendable {
    private let lock = NSLock()
    private var sent: [APIRequest] = []
    let respond: @Sendable (APIRequest) throws -> APIResponse

    init(_ respond: @escaping @Sendable (APIRequest) throws -> APIResponse) { self.respond = respond }

    var requests: [APIRequest] { lock.withLock { sent } }

    func send(_ request: APIRequest) async throws -> APIResponse {
        lock.withLock { sent.append(request) }
        return try respond(request)
    }
}

@Suite struct ClientTests {
    let connection = try! Connection.parse("https://wpm.example.org").get()

    func client(_ respond: @escaping @Sendable (APIRequest) throws -> APIResponse) -> (WPMClient, StubTransport) {
        let transport = StubTransport(respond)
        return (WPMClient(connection: connection, token: "v4.public.t", transport: transport), transport)
    }

    @Test func timeOffDecodesWithUnknownsKept() async throws {
        let (c, t) = client { _ in APIResponse(status: 200, body: Data(timeOffJSON.utf8)) }
        let timeOff = try await c.timeOff(year: 2026)
        #expect(timeOff.unit == "calendar_days")
        #expect(timeOff.asOf.iso == "2026-11-23")
        #expect(timeOff.balances.count == 2)
        #expect(timeOff.balances[0].remaining == 15)
        #expect(timeOff.balances[1].entitled == nil)
        #expect(timeOff.balances[1].remaining == nil, "no allowance is unknown, not zero")
        #expect(DayFigure(timeOff.balances[1].remaining) == .notRecorded)
        #expect(timeOff.requests[1].kind == "sick" && timeOff.requests[1].reason == nil)
        #expect(timeOff.history[0].absences[0] == AbsenceTotal(kind: "annual", absences: 4, days: 22))
        #expect(t.requests.first?.url.absoluteString == "https://wpm.example.org/api/me/time-off?year=2026")
    }

    @Test func everyRequestCarriesVersionAcceptAndBearer() {
        let draft = LeaveDraft(kind: "annual", startOn: CalendarDate(iso: "2026-07-01")!,
                               endOn: CalendarDate(iso: "2026-07-02")!)
        for r in [Endpoints.timeOff(connection, token: "tok", year: nil),
                  Endpoints.requestLeave(connection, token: "tok", draft: draft),
                  Endpoints.cancelLeave(connection, token: "tok", pid: "p")] {
            #expect(r.headers["Accepts-version"] == "1.0")
            #expect(r.headers["Accept"] == "application/json")
            #expect(r.headers["Authorization"] == "Bearer tok")
        }
    }

    @Test func timeOffWithNoYearAsksForTheCurrentOne() {
        #expect(Endpoints.timeOff(connection, token: "t", year: nil).url.absoluteString
            == "https://wpm.example.org/api/me/time-off")
    }

    @Test func requestLeaveBody() throws {
        let draft = LeaveDraft(kind: "annual", startOn: CalendarDate(iso: "2026-07-01")!,
                               endOn: CalendarDate(iso: "2026-07-03")!, reason: " Wedding ")
        let r = Endpoints.requestLeave(connection, token: "t", draft: draft)
        #expect(r.method == "POST")
        #expect(r.url.path == "/api/me/leave-requests")
        #expect(r.headers["Content-Type"] == "application/json")
        #expect(String(decoding: try #require(r.body), as: UTF8.self)
            == #"{"end_on":"2026-07-03","kind":"annual","reason":"Wedding","start_on":"2026-07-01"}"#)
    }

    @Test func sickLeaveBodyHasNoReasonKeyAtAll() throws {
        let draft = LeaveDraft(kind: "sick", startOn: CalendarDate(iso: "2026-07-01")!,
                               endOn: CalendarDate(iso: "2026-07-01")!, reason: "flu")
        let body = String(decoding: try #require(Endpoints.requestLeave(connection, token: "t", draft: draft).body),
                          as: UTF8.self)
        #expect(body == #"{"end_on":"2026-07-01","kind":"sick","start_on":"2026-07-01"}"#)
    }

    @Test func cancelPathEscapesThePid() {
        #expect(Endpoints.cancelLeave(connection, token: "t", pid: "0f9f3a52-1111").url.absoluteString
            == "https://wpm.example.org/api/me/leave-requests/0f9f3a52-1111/cancel")
        #expect(Endpoints.cancelLeave(connection, token: "t", pid: "a/../b").url.absoluteString
            == "https://wpm.example.org/api/me/leave-requests/a%2F..%2Fb/cancel")
    }

    @Test func createdDecodes() async throws {
        let (c, _) = client { _ in
            APIResponse(status: 200, body: Data(#"{"pid":"p1","days":3,"negative_balance":false}"#.utf8))
        }
        let draft = LeaveDraft(kind: "annual", startOn: CalendarDate(iso: "2026-07-01")!,
                               endOn: CalendarDate(iso: "2026-07-03")!)
        #expect(try await c.requestLeave(draft) == LeaveRequestCreated(pid: "p1", days: 3, negativeBalance: false))
    }

    @Test func a401IsUnauthorized() async {
        let (c, _) = client { _ in APIResponse(status: 401, body: Data()) }
        await #expect(throws: APIError.unauthorized) { try await c.timeOff(year: nil) }
    }

    @Test func aRefusalCarriesTheServicesWords() async {
        let body = #"{"type":"about:blank","title":"Unprocessable Entity","status":422,"code":"unprocessable","detail":"requested 5 calendar days exceeds remaining balance 2 calendar days"}"#
        let (c, _) = client { _ in APIResponse(status: 422, body: Data(body.utf8)) }
        do {
            try await c.cancelLeave(pid: "p")
            Issue.record("expected a refusal")
        } catch {
            guard case .refused(422, let problem?) = error else { Issue.record("got \(error)"); return }
            #expect(problem.code == "unprocessable")
            #expect(problem.message == "requested 5 calendar days exceeds remaining balance 2 calendar days")
        }
    }

    @Test func aRefusalWithNoBodyHasNoProblem() async {
        let (c, _) = client { _ in APIResponse(status: 500, body: Data("<html>".utf8)) }
        await #expect(throws: APIError.refused(status: 500, problem: nil)) { try await c.timeOff(year: nil) }
    }

    @Test func aBodyThatIsNotTheDocumentedShapeIsUnreadable() async {
        let (c, _) = client { _ in APIResponse(status: 200, body: Data(#"{"unit":"calendar_days"}"#.utf8)) }
        await #expect(throws: APIError.unreadable) { try await c.timeOff(year: nil) }
    }

    @Test func aTransportFailureIsUnreachable() async {
        let (c, _) = client { _ in throw URLError(.notConnectedToInternet) }
        await #expect(throws: APIError.unreachable) { try await c.timeOff(year: nil) }
    }

    @Test func problemMessagePrefersDetailThenTitle() {
        #expect(ProblemDetails(title: "T", detail: "D").message == "D")
        #expect(ProblemDetails(title: "T", detail: "  ").message == "T")
        #expect(ProblemDetails().message == nil)
    }
}

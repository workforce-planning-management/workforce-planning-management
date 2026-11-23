// The state behind "My time off" (spec/mobile-apps.md, WPM-R282, WPM-R283). The answer is held in memory only
// (WPM-D227). A write is sent once per tap: a second tap while one is in flight does nothing.
import Foundation
import Observation
import WPMCore

@MainActor
@Observable
public final class TimeOffModel {
    public enum State: Equatable {
        case loading
        case loaded(TimeOff)
        case failed(String)
    }

    public private(set) var state: State = .loading
    /// The year shown; `nil` until the service says which year is current.
    public private(set) var year: Int?
    public private(set) var sending = false
    /// A refusal of a cancel, in the service's words.
    public var actionProblem: String?

    @ObservationIgnored let session: Session
    @ObservationIgnored let now: @Sendable () -> Date

    public init(session: Session, now: @escaping @Sendable () -> Date = Date.init) {
        self.session = session
        self.now = now
    }

    public var today: CalendarDate { CalendarDate(now(), in: .current) }

    public func load() async {
        guard let client = session.client else { return }
        if case .loaded = state {} else { state = .loading }
        do {
            let timeOff = try await client.timeOff(year: year)
            year = timeOff.year
            state = .loaded(timeOff)
        } catch {
            session.handle(error)
            state = .failed(Words.error(error))
        }
    }

    public func show(year newYear: Int) async {
        year = newYear
        state = .loading
        await load()
    }

    public func canCancel(_ request: LeaveRequest) -> Bool {
        LeaveRules.canCancel(status: request.status, startOn: request.startOn, today: today)
    }

    public func cancel(_ request: LeaveRequest) async {
        guard !sending, let client = session.client, canCancel(request) else { return }
        sending = true
        defer { sending = false }
        actionProblem = nil
        do {
            try await client.cancelLeave(pid: request.pid)
            await load()
        } catch {
            session.handle(error)
            actionProblem = Words.error(error)
        }
    }

    /// Sends a request. Returns `nil` on success, or the problem to show.
    public func submit(_ draft: LeaveDraft) async -> String? {
        if let first = draft.problems.first { return Words.draft(first) }
        guard !sending, let client = session.client else { return nil }
        sending = true
        defer { sending = false }
        do {
            _ = try await client.requestLeave(draft)
            await load()
            return nil
        } catch {
            session.handle(error)
            return Words.error(error)
        }
    }
}

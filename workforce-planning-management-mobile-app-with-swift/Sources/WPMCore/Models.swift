// What `/api/me/time-off` and the leave routes return (spec/mobile-apps.md, WPM-R282, WPM-R283). The shapes are the
// service's (`src/controllers/me.rs`, `my_time_off`; `src/controllers/workforce.rs`, `create_leave_for`). Figures are
// calendar days. An absent allowance is `nil`, never 0.
import Foundation

public struct TimeOff: Codable, Equatable, Sendable {
    public let unit: String
    public let asOf: CalendarDate
    public let year: Int
    public let balances: [LeaveBalance]
    public let requests: [LeaveRequest]
    public let history: [AbsenceYear]
    public let derivation: String

    enum CodingKeys: String, CodingKey {
        case unit, year, balances, requests, history, derivation
        case asOf = "as_of"
    }

    public init(unit: String, asOf: CalendarDate, year: Int, balances: [LeaveBalance], requests: [LeaveRequest],
                history: [AbsenceYear], derivation: String) {
        self.unit = unit
        self.asOf = asOf
        self.year = year
        self.balances = balances
        self.requests = requests
        self.history = history
        self.derivation = derivation
    }
}

public struct LeaveBalance: Codable, Equatable, Sendable, Identifiable {
    public let kind: String
    public let year: Int
    /// Calendar days allowed; `nil` when no allowance is recorded.
    public let entitled: Int?
    public let used: Int
    public let taken: Int
    public let booked: Int
    public let requested: Int
    /// `entitled - used`; `nil` when there is no allowance (unknown, not zero).
    public let remaining: Int?
    public let remainingIfRequestedApproved: Int?

    public var id: String { "\(kind)-\(year)" }

    enum CodingKeys: String, CodingKey {
        case kind, year, entitled, used, taken, booked, requested, remaining
        case remainingIfRequestedApproved = "remaining_if_requested_approved"
    }

    public init(kind: String, year: Int, entitled: Int?, used: Int, taken: Int, booked: Int, requested: Int,
                remaining: Int?, remainingIfRequestedApproved: Int?) {
        self.kind = kind
        self.year = year
        self.entitled = entitled
        self.used = used
        self.taken = taken
        self.booked = booked
        self.requested = requested
        self.remaining = remaining
        self.remainingIfRequestedApproved = remainingIfRequestedApproved
    }
}

public struct LeaveRequest: Codable, Equatable, Sendable, Identifiable {
    public let pid: String
    public let kind: String
    public let startOn: CalendarDate
    public let endOn: CalendarDate
    public let days: Int
    public let status: String
    /// Never present for sick leave (WPM-D73).
    public let reason: String?

    public var id: String { pid }

    enum CodingKeys: String, CodingKey {
        case pid, kind, days, status, reason
        case startOn = "start_on"
        case endOn = "end_on"
    }

    public init(pid: String, kind: String, startOn: CalendarDate, endOn: CalendarDate, days: Int, status: String,
                reason: String?) {
        self.pid = pid
        self.kind = kind
        self.startOn = startOn
        self.endOn = endOn
        self.days = days
        self.status = status
        self.reason = reason
    }
}

public struct AbsenceYear: Codable, Equatable, Sendable, Identifiable {
    public let year: Int
    public let absences: [AbsenceTotal]
    public var id: Int { year }

    public init(year: Int, absences: [AbsenceTotal]) {
        self.year = year
        self.absences = absences
    }
}

public struct AbsenceTotal: Codable, Equatable, Sendable, Identifiable {
    public let kind: String
    /// How many absences.
    public let absences: Int
    /// Calendar days in them.
    public let days: Int
    public var id: String { kind }

    public init(kind: String, absences: Int, days: Int) {
        self.kind = kind
        self.absences = absences
        self.days = days
    }
}

/// `POST /api/me/leave-requests` answers with the new request's pid, the calendar days it spans and whether it takes
/// the balance below zero.
public struct LeaveRequestCreated: Codable, Equatable, Sendable {
    public let pid: String
    public let days: Int
    public let negativeBalance: Bool

    enum CodingKeys: String, CodingKey {
        case pid, days
        case negativeBalance = "negative_balance"
    }
}

/// A figure that may be unknown. The screens show `.notRecorded` in words, never as 0.
public enum DayFigure: Equatable, Sendable {
    case days(Int)
    case notRecorded

    public init(_ value: Int?) {
        self = value.map(DayFigure.days) ?? .notRecorded
    }
}

/// The service's problem details (RFC 9457) for a refused request.
public struct ProblemDetails: Codable, Equatable, Sendable {
    public let type: String?
    public let title: String?
    public let status: Int?
    public let code: String?
    public let detail: String?

    public init(type: String? = nil, title: String? = nil, status: Int? = nil, code: String? = nil,
                detail: String? = nil) {
        self.type = type
        self.title = title
        self.status = status
        self.code = code
        self.detail = detail
    }

    /// What to show the person: the service's own words, `detail` first, then `title`; `nil` when it said nothing.
    public var message: String? {
        if let detail, !detail.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty { return detail }
        if let title, !title.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty { return title }
        return nil
    }
}

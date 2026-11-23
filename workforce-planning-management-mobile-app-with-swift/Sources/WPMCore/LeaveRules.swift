// The leave rules the screens show (spec/mobile-apps.md, WPM-R283). Each mirrors the service's domain crate, which
// stays the authority: `time_off::can_worker_cancel`, `leave::day_span` and `leave::validate_reason`.
import Foundation

public enum LeaveRules {
    /// The service's closed list of leave kinds (`tokens::LEAVE_KINDS`), in its order.
    public static let kinds: [String] = ["annual", "public_holiday", "sick", "parental", "unpaid", "other"]

    /// Request statuses (`tokens::LEAVE_STATUSES`).
    public static let statuses: [String] = ["requested", "approved", "rejected", "cancelled"]

    /// The longest leave the service accepts, in calendar days.
    public static let maxSpanDays = 366

    /// The longest reason the app sends, in characters.
    public static let reasonMaxLength = 500

    /// Whether a request of `kind` may carry a reason. Sick leave records that it is sick leave and its dates, never
    /// why (WPM-D73).
    public static func reasonAllowed(_ kind: String) -> Bool { kind != "sick" }

    /// Whether the worker may cancel a request: one awaiting a decision (even if its dates passed), or one approved
    /// that has not started.
    public static func canCancel(status: String, startOn: CalendarDate, today: CalendarDate) -> Bool {
        switch status {
        case "requested": return true
        case "approved": return startOn > today
        default: return false
        }
    }

    public enum SpanError: Error, Equatable, Sendable {
        case endBeforeStart
        case tooLong(days: Int)
    }

    /// The calendar days from `start` to `end`, both included, as the service counts them.
    public static func daySpan(start: CalendarDate, end: CalendarDate) -> Result<Int, SpanError> {
        if end < start { return .failure(.endBeforeStart) }
        let days = end.dayNumber - start.dayNumber + 1
        if days > maxSpanDays { return .failure(.tooLong(days: days)) }
        return .success(days)
    }
}

/// What the person has filled in on the request form.
public struct LeaveDraft: Equatable, Sendable {
    public var kind: String
    public var startOn: CalendarDate
    public var endOn: CalendarDate
    public var reason: String

    public init(kind: String, startOn: CalendarDate, endOn: CalendarDate, reason: String = "") {
        self.kind = kind
        self.startOn = startOn
        self.endOn = endOn
        self.reason = reason
    }

    public enum Problem: Equatable, Sendable {
        case unknownKind
        case endBeforeStart
        case tooLong(days: Int)
        case reasonTooLong(max: Int)
    }

    /// Shape checks only; the service decides the rest (a balance, for example).
    public var problems: [Problem] {
        var found: [Problem] = []
        if !LeaveRules.kinds.contains(kind) { found.append(.unknownKind) }
        switch LeaveRules.daySpan(start: startOn, end: endOn) {
        case .success: break
        case .failure(.endBeforeStart): found.append(.endBeforeStart)
        case .failure(.tooLong(let days)): found.append(.tooLong(days: days))
        }
        if LeaveRules.reasonAllowed(kind), trimmedReason.count > LeaveRules.reasonMaxLength {
            found.append(.reasonTooLong(max: LeaveRules.reasonMaxLength))
        }
        return found
    }

    /// The days the dates span, or `nil` when they are not a valid span.
    public var days: Int? { try? LeaveRules.daySpan(start: startOn, end: endOn).get() }

    var trimmedReason: String { reason.trimmingCharacters(in: .whitespacesAndNewlines) }

    /// The reason to send: none for sick leave, and none when it is blank.
    public var reasonToSend: String? {
        guard LeaveRules.reasonAllowed(kind) else { return nil }
        let text = trimmedReason
        return text.isEmpty ? nil : text
    }
}

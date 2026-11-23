// Strings are content (spec/mobile-apps.md, WPM-R284): every word is in Resources/Localizable.xcstrings, none in code.
// The language follows the device; a missing translation falls back to English, never to a key.
import Foundation
import WPMCore

public enum WPMStrings {
    /// The bundle that holds the string catalog.
    public static var bundle: Bundle { .module }
}

/// The string for `key` in the device's language.
func L(_ key: String) -> String {
    NSLocalizedString(key, tableName: nil, bundle: .module, value: "", comment: "")
}

/// The string for `key`, with `args` put in its placeholders (and a plural chosen where the catalog has one).
func L(_ key: String, _ args: any CVarArg...) -> String {
    String(format: L(key), locale: Locale.current, arguments: args)
}

enum Words {
    /// The word for a leave kind; a kind the app does not know is shown as the service sent it.
    static func kind(_ kind: String) -> String { known("kind.\(kind)") ?? kind }

    /// The word for a request status; an unknown status is shown as the service sent it.
    static func status(_ status: String) -> String { known("status.\(status)") ?? status }

    static func known(_ key: String) -> String? {
        let text = L(key)
        return text.isEmpty || text == key ? nil : text
    }

    /// "N calendar days" (WPM-R282: every figure says calendar days).
    static func days(_ n: Int) -> String { L("days.count", n) }

    static func figure(_ figure: DayFigure) -> String {
        switch figure {
        case .days(let n): return days(n)
        case .notRecorded: return L("timeOff.notRecorded")
        }
    }

    /// A calendar date in the device's locale and calendar, with no time zone shift.
    static func date(_ date: CalendarDate) -> String {
        var gregorian = Calendar(identifier: .gregorian)
        gregorian.timeZone = TimeZone(identifier: "UTC") ?? .current
        guard let instant = date.date(in: gregorian) else { return date.iso }
        return instant.formatted(Date.FormatStyle(date: .abbreviated, time: .omitted, locale: .current,
                                                  calendar: .current, timeZone: gregorian.timeZone))
    }

    static func range(_ start: CalendarDate, _ end: CalendarDate) -> String {
        start == end ? date(start) : L("dates.range", date(start), date(end))
    }

    /// The service's words for a refusal, or ours when it gave none.
    static func error(_ error: APIError) -> String {
        switch error {
        case .unauthorized: return L("error.signedOut")
        case .refused(_, let problem): return problem?.message ?? L("error.refused")
        case .unreadable: return L("error.unreadable")
        case .unreachable: return L("error.unreachable")
        }
    }

    static func connection(_ problem: Connection.Problem) -> String {
        switch problem {
        case .empty: return L("connect.error.empty")
        case .notAnAddress: return L("connect.error.notAnAddress")
        case .insecureScheme: return L("connect.error.insecure")
        case .credentialsInAddress: return L("connect.error.credentials")
        case .queryOrFragment: return L("connect.error.query")
        }
    }

    static func draft(_ problem: LeaveDraft.Problem) -> String {
        switch problem {
        case .unknownKind: return L("request.error.kind")
        case .endBeforeStart: return L("request.error.endBeforeStart")
        case .tooLong(let days): return L("request.error.tooLong", days, LeaveRules.maxSpanDays)
        case .reasonTooLong(let max): return L("request.error.reasonTooLong", max)
        }
    }
}

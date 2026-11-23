import Testing
@testable import WPMCore

private func d(_ text: String) -> CalendarDate { CalendarDate(iso: text)! }

@Suite struct LeaveRulesTests {
    // The same table as the service's `time_off::can_worker_cancel` tests.
    @Test(arguments: [
        ("requested", "2026-01-05", true),   // awaiting a decision, even if the dates passed
        ("requested", "2026-06-15", true),
        ("requested", "2026-07-01", true),
        ("approved", "2026-06-16", true),    // approved and not started
        ("approved", "2026-06-15", false),   // starts today: under way
        ("approved", "2026-06-01", false),   // started or over
        ("rejected", "2026-07-01", false),
        ("cancelled", "2026-07-01", false),
        ("unknown", "2026-07-01", false),
        ("", "2026-07-01", false),
    ])
    func cancelRule(status: String, start: String, expected: Bool) {
        #expect(LeaveRules.canCancel(status: status, startOn: d(start), today: d("2026-06-15")) == expected)
    }

    @Test func daySpanIsInclusive() {
        #expect(LeaveRules.daySpan(start: d("2026-06-15"), end: d("2026-06-15")) == .success(1))
        #expect(LeaveRules.daySpan(start: d("2026-06-15"), end: d("2026-06-19")) == .success(5))
        #expect(LeaveRules.daySpan(start: d("2024-02-28"), end: d("2024-03-01")) == .success(3))
        #expect(LeaveRules.daySpan(start: d("2026-12-31"), end: d("2027-01-01")) == .success(2))
    }

    @Test func daySpanCapIs366CalendarDays() {
        #expect(LeaveRules.daySpan(start: d("2024-01-01"), end: d("2024-12-31")) == .success(366))
        #expect(LeaveRules.daySpan(start: d("2026-01-01"), end: d("2027-01-01")) == .success(366))
        #expect(LeaveRules.daySpan(start: d("2026-01-01"), end: d("2027-01-02")) == .failure(.tooLong(days: 367)))
        #expect(LeaveRules.daySpan(start: d("2026-06-16"), end: d("2026-06-15")) == .failure(.endBeforeStart))
    }

    @Test(arguments: LeaveRules.kinds)
    func onlySickLeaveHasNoReason(kind: String) {
        #expect(LeaveRules.reasonAllowed(kind) == (kind != "sick"))
    }

    @Test func kindsAndStatusesAreTheServicesClosedLists() {
        #expect(LeaveRules.kinds == ["annual", "public_holiday", "sick", "parental", "unpaid", "other"])
        #expect(LeaveRules.statuses == ["requested", "approved", "rejected", "cancelled"])
    }

    @Test func aDraftChecksShapeOnly() {
        #expect(LeaveDraft(kind: "annual", startOn: d("2026-07-01"), endOn: d("2026-07-03")).problems.isEmpty)
        #expect(LeaveDraft(kind: "holiday", startOn: d("2026-07-01"), endOn: d("2026-07-03")).problems == [.unknownKind])
        #expect(LeaveDraft(kind: "annual", startOn: d("2026-07-03"), endOn: d("2026-07-01")).problems == [.endBeforeStart])
        #expect(LeaveDraft(kind: "annual", startOn: d("2026-01-01"), endOn: d("2027-06-01")).problems
            == [.tooLong(days: 517)])
        let long = String(repeating: "a", count: 501)
        #expect(LeaveDraft(kind: "other", startOn: d("2026-07-01"), endOn: d("2026-07-01"), reason: long).problems
            == [.reasonTooLong(max: 500)])
        #expect(LeaveDraft(kind: "other", startOn: d("2026-07-01"), endOn: d("2026-07-01"),
                           reason: String(repeating: "a", count: 500)).problems.isEmpty)
    }

    @Test func aSickDraftNeverSendsAReasonEvenALongOne() {
        let draft = LeaveDraft(kind: "sick", startOn: d("2026-07-01"), endOn: d("2026-07-02"),
                               reason: String(repeating: "x", count: 600))
        #expect(draft.reasonToSend == nil)
        #expect(draft.problems.isEmpty)
    }

    @Test func aBlankReasonIsNotSentAndOneIsTrimmed() {
        #expect(LeaveDraft(kind: "annual", startOn: d("2026-07-01"), endOn: d("2026-07-01"), reason: "  \n ").reasonToSend == nil)
        #expect(LeaveDraft(kind: "annual", startOn: d("2026-07-01"), endOn: d("2026-07-01"), reason: " Wedding ").reasonToSend
            == "Wedding")
    }

    @Test func draftDays() {
        #expect(LeaveDraft(kind: "annual", startOn: d("2026-07-01"), endOn: d("2026-07-05")).days == 5)
        #expect(LeaveDraft(kind: "annual", startOn: d("2026-07-05"), endOn: d("2026-07-01")).days == nil)
    }

    @Test func anUnknownFigureIsNotZero() {
        #expect(DayFigure(nil) == .notRecorded)
        #expect(DayFigure(0) == .days(0))
        #expect(DayFigure(0) != .notRecorded)
    }
}

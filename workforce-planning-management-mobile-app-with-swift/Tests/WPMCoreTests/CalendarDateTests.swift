import Foundation
import Testing
@testable import WPMCore

@Suite struct CalendarDateTests {
    @Test(arguments: ["2026-01-01", "2024-02-29", "2000-02-29", "1970-01-01", "9999-12-31", "0001-01-01"])
    func roundTrips(_ text: String) throws {
        let date = try #require(CalendarDate(iso: text))
        #expect(date.iso == text)
        #expect(CalendarDate(dayNumber: date.dayNumber) == date)
    }

    @Test(arguments: ["2026-02-29", "1900-02-29", "2026-13-01", "2026-00-10", "2026-04-31", "2026-1-01", "26-01-01",
                      "2026-01-01T00:00:00Z", "", "２０２６-01-01", "2026/01/01", "0000-01-01", " 2026-01-01"])
    func refusesWhatIsNotADate(_ text: String) {
        #expect(CalendarDate(iso: text) == nil)
    }

    @Test func dayNumbersMatchKnownDates() throws {
        #expect(try #require(CalendarDate(iso: "1970-01-01")).dayNumber == 0)
        #expect(try #require(CalendarDate(iso: "2000-03-01")).dayNumber == 11017)
        #expect(try #require(CalendarDate(iso: "1969-12-31")).dayNumber == -1)
    }

    @Test func everyDayOfFourHundredYearsRoundTrips() throws {
        var date = try #require(CalendarDate(iso: "2000-01-01"))
        let first = date.dayNumber
        for offset in 0..<146_097 {
            #expect(date.dayNumber == first + offset)
            let next = try #require(date.adding(days: 1))
            #expect(next > date)
            date = next
        }
        #expect(date.iso == "2400-01-01")
    }

    @Test func leapYears() {
        #expect(CalendarDate.isLeap(2024))
        #expect(CalendarDate.isLeap(2000))
        #expect(!CalendarDate.isLeap(1900))
        #expect(!CalendarDate.isLeap(2026))
    }

    @Test func todayIsTheDateInTheCalendarsTimeZoneNotUTC() throws {
        // 2026-11-23T23:30:00Z is already the 24th in Tokyo and still the 23rd in Los Angeles.
        let instant = Date(timeIntervalSince1970: 1_795_476_600)
        var tokyo = Calendar(identifier: .gregorian)
        tokyo.timeZone = try #require(TimeZone(identifier: "Asia/Tokyo"))
        var losAngeles = Calendar(identifier: .gregorian)
        losAngeles.timeZone = try #require(TimeZone(identifier: "America/Los_Angeles"))
        #expect(CalendarDate(instant, in: tokyo).iso == "2026-11-24")
        #expect(CalendarDate(instant, in: losAngeles).iso == "2026-11-23")
    }

    @Test func aDateFromANonGregorianCalendarIsStillGregorian() throws {
        var buddhist = Calendar(identifier: .buddhist)
        buddhist.timeZone = try #require(TimeZone(identifier: "UTC"))
        let instant = Date(timeIntervalSince1970: 1_795_476_600)
        #expect(CalendarDate(instant, in: buddhist).iso == "2026-11-23")
    }

    @Test func codableIsTheIsoString() throws {
        let date = try #require(CalendarDate(iso: "2026-07-04"))
        let data = try JSONEncoder().encode([date])
        #expect(String(decoding: data, as: UTF8.self) == "[\"2026-07-04\"]")
        #expect(try JSONDecoder().decode([CalendarDate].self, from: data) == [date])
        #expect(throws: DecodingError.self) { try JSONDecoder().decode([CalendarDate].self, from: Data("[\"2026-02-30\"]".utf8)) }
    }
}

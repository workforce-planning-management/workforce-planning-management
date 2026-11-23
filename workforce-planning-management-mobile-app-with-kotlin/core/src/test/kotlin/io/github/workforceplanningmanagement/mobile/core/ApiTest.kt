package io.github.workforceplanningmanagement.mobile.core

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test
import java.io.IOException
import java.time.LocalDate

class ApiTest {
    private val timeOffJson = javaClass.getResource("/time-off.json")!!.readText()
    private val sent = mutableListOf<HttpRequest>()
    private fun api(response: HttpResponse) = WpmApi("https://wpm.example.org", "v4.public.t", Transport { sent += it; response })

    @Test fun everyRequestSaysTheVersionAndCarriesTheToken() {
        val h = api(HttpResponse(200, "")).headers()
        assertEquals("1.0", h["Accepts-version"])
        assertEquals("application/json", h["Accept"])
        assertEquals("Bearer v4.public.t", h["Authorization"])
        assertNull(h["Content-Type"])
    }

    @Test fun theTimeOffIsDecoded() {
        val r = api(HttpResponse(200, timeOffJson)).timeOff(2026) as ApiResult.Ok
        assertEquals("GET", sent.single().method)
        assertEquals("https://wpm.example.org/api/me/time-off?year=2026", sent.single().url)
        val t = r.value
        assertEquals("calendar_days", t.unit)
        assertEquals(LocalDate.of(2026, 11, 23), t.asOf)
        assertEquals(2, t.balances.size)
        val annual = t.balances[0]
        assertEquals(Figure.Days(15), annual.remainingFigure())
        assertEquals(Figure.Days(25), annual.entitledFigure())
        assertEquals(12, annual.remainingIfRequestedApproved)
        val unpaid = t.balances[1]
        assertEquals("unknown, never zero", Figure.NotRecorded, unpaid.remainingFigure())
        assertEquals(Figure.NotRecorded, unpaid.entitledFigure())
        assertEquals(3, t.requests.size)
        assertNull("sick leave carries no reason", t.requests[1].reason)
        assertEquals(LocalDate.of(2026, 8, 10), t.requests[0].endOn)
        assertEquals(2025, t.history.single().year)
        assertEquals(Absence("annual", 3, 21), t.history.single().absences[0])
        assertTrue(t.derivation.startsWith("remaining ="))
    }

    @Test fun aMinimalTimeOffIsDecoded() {
        val t = decodeTimeOff("""{"unit":"calendar_days","as_of":"2026-01-01","year":2026}""")
        assertEquals(emptyList<Balance>(), t.balances)
        assertEquals("", t.derivation)
    }

    @Test fun aLeaveRequestIsPostedAsJson() {
        val p = LeaveRequestPayload("annual", LocalDate.of(2026, 12, 21), LocalDate.of(2026, 12, 23))
        assertEquals(ApiResult.Ok(Unit), api(HttpResponse(201, "{}")).requestLeave(p))
        val r = sent.single()
        assertEquals("POST", r.method)
        assertEquals("https://wpm.example.org/api/me/leave-requests", r.url)
        assertEquals("application/json", r.headers["Content-Type"])
        assertEquals("""{"kind":"annual","start_on":"2026-12-21","end_on":"2026-12-23"}""", r.body)
    }

    @Test fun aCancelPostsToThePid() {
        assertEquals(ApiResult.Ok(Unit), api(HttpResponse(200, "{}")).cancel("0b8f4f6e-7c55"))
        assertEquals("https://wpm.example.org/api/me/leave-requests/0b8f4f6e-7c55/cancel", sent.single().url)
        api(HttpResponse(200, "{}")).cancel("a/b")
        assertEquals("a path character is escaped", "https://wpm.example.org/api/me/leave-requests/a%2Fb/cancel", sent.last().url)
    }

    @Test fun a401SignsOut() {
        assertEquals(ApiResult.SignedOut, api(HttpResponse(401, "")).timeOff(2026))
    }

    @Test fun aRefusalCarriesTheServicesWords() {
        val body = """{"type":"about:blank","title":"Unprocessable Entity","status":422,"code":"unprocessable",
            "detail":"leave spans 400 calendar days; the cap is 366 calendar days"}"""
        val r = api(HttpResponse(422, body)).timeOff(2026) as ApiResult.Refused
        assertEquals(422, r.status)
        assertEquals("unprocessable", r.problem?.code)
        assertEquals("leave spans 400 calendar days; the cap is 366 calendar days", r.problem?.detail)
    }

    @Test fun aRefusalWithoutProblemDetailsHasNone() {
        assertEquals(ApiResult.Refused(502, null), api(HttpResponse(502, "<html>Bad gateway</html>")).timeOff(2026))
        assertEquals(ApiResult.Refused(404, null), api(HttpResponse(404, "{}")).timeOff(2026))
    }

    @Test fun nothingBackIsUnreachable() {
        val api = WpmApi("https://wpm.example.org", "t", Transport { throw IOException("offline") })
        assertEquals(ApiResult.Unreachable(), api.timeOff(2026))
    }

    @Test fun anUnreadableBodyIsNotShownAsData() {
        assertEquals(ApiResult.Unreachable(unreadable = true), api(HttpResponse(200, "not json")).timeOff(2026))
        assertEquals(ApiResult.Unreachable(unreadable = true),
            api(HttpResponse(200, """{"unit":"calendar_days","as_of":"not a date","year":2026}""")).timeOff(2026))
    }
}

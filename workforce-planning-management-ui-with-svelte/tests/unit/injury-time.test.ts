import { describe, expect, it } from "vitest";
import { resolveInjuryTime } from "#lib/injury-time.js";

describe("injury timestamp resolution (WPM-D205, WPM-R221)", () => {
  it("keeps an omitted timestamp absent and refuses a zone the browser cannot resolve", () => {
    expect(resolveInjuryTime("", "")).toEqual({ kind: "empty" });
    expect(resolveInjuryTime("2026-04-05T01:45", "Australia/Unknown")).toEqual({
      kind: "invalidZone",
    });
  });

  it("resolves a half-hour overlap into two instants", () => {
    expect(
      resolveInjuryTime("2026-04-05T01:45", "Australia/Lord_Howe"),
    ).toEqual({
      kind: "repeated",
      candidates: [
        { instant: "2026-04-04T14:45:00.000Z", offset: "UTC+11:00" },
        { instant: "2026-04-04T15:15:00.000Z", offset: "UTC+10:30" },
      ],
    });
  });

  it("refuses a skipped date and an instant outside the API's four-digit year range", () => {
    expect(resolveInjuryTime("2011-12-30T12:00", "Pacific/Apia")).toEqual({
      kind: "skipped",
    });
    expect(resolveInjuryTime("9999-12-31T23:30", "America/New_York")).toEqual({
      kind: "invalidTime",
    });
  });
});

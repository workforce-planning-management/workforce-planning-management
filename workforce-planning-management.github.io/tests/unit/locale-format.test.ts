import { describe, expect, it } from "vitest";
import {
  DASH,
  OPEN_END,
  arrow,
  formatDate,
  formatDateRange,
  formatInteger,
  formatNumber,
  isCivilDate,
} from "../../src/lib/locale-format.js";

describe("formatNumber and formatInteger", () => {
  it("shows an unknown or non-finite value as a dash, never zero", () => {
    for (const v of [null, undefined, Number.NaN, Number.POSITIVE_INFINITY]) expect(formatNumber(v, "en")).toBe(DASH);
    expect(formatInteger(null, "en")).toBe(DASH);
  });
  it("groups in the locale", () => {
    expect(formatNumber(1234.5, "en")).toBe("1,234.5");
    expect(formatNumber(1234.5, "de")).toBe("1.234,5");
    expect(formatInteger(1234.6, "en")).toBe("1,235");
    expect(formatNumber(0, "en")).toBe("0");
  });
  it("uses English for a locale Intl rejects", () => {
    expect(formatNumber(1234, "not a locale!")).toBe("1,234");
  });
});

describe("isCivilDate", () => {
  it("accepts a real civil date only", () => {
    expect(isCivilDate("2026-10-30")).toBe(true);
    expect(isCivilDate("2026-02-30")).toBe(false);
    expect(isCivilDate("2026-10-30T00:00:00Z")).toBe(false);
    expect(isCivilDate(20261030)).toBe(false);
    expect(isCivilDate("2026-13-01")).toBe(false);
  });
});

describe("formatDate", () => {
  it("shows an absent value as a dash", () => {
    expect(formatDate(null, "en")).toBe(DASH);
    expect(formatDate(undefined, "en")).toBe(DASH);
    expect(formatDate("", "en")).toBe(DASH);
  });
  it("formats a civil date as the same day everywhere", () => {
    expect(formatDate("2026-10-11", "en-GB")).toBe("11 Oct 2026");
    expect(formatDate("2026-10-11", "de-DE")).toBe("11.10.2026");
    expect(formatDate("2026-10-11", "en", "long")).toBe("October 11, 2026");
  });
  it("shows a date that does not exist, or is not a date, as written", () => {
    expect(formatDate("2026-02-30", "en")).toBe("2026-02-30");
    expect(formatDate("soon", "en")).toBe("soon");
    expect(formatDate("2026-99-99T00:00:00Z", "en")).toBe("2026-99-99T00:00:00Z");
  });
  it("formats an instant", () => {
    expect(formatDate("2026-10-11T12:00:00Z", "en-GB")).toMatch(/^1[12] Oct 2026$/);
  });
  it("uses English for a locale Intl rejects", () => {
    expect(formatDate("2026-10-11", "not a locale!")).toBe("Oct 11, 2026");
  });
});

describe("formatDateRange", () => {
  it("formats two civil dates as one range", () => {
    expect(formatDateRange("2026-10-01", "2026-10-30", "en-GB")).toMatch(/^1\s*–\s*30 Oct 2026$/);
  });
  it("formats two instants as one range", () => {
    expect(formatDateRange("2026-10-01T00:00:00Z", "2026-10-30T00:00:00Z", "en-GB")).toContain("Oct 2026");
  });
  it("marks a missing end", () => {
    expect(formatDateRange("2026-10-01", null, "en-GB")).toBe(`1 Oct 2026 – ${OPEN_END}`);
    expect(formatDateRange("", "2026-10-30", "en-GB")).toBe(`${OPEN_END} – 30 Oct 2026`);
    expect(formatDateRange(undefined, undefined, "en-GB")).toBe(`${OPEN_END} – ${OPEN_END}`);
  });
  it("shows each end on its own when the range is reversed, mixed or not a date", () => {
    expect(formatDateRange("2026-10-30", "2026-10-01", "en-GB")).toBe("30 Oct 2026 – 1 Oct 2026");
    expect(formatDateRange("2026-10-01", "2026-10-30T00:00:00Z", "en-GB")).toBe("1 Oct 2026 – 30 Oct 2026");
    expect(formatDateRange("soon", "later", "en-GB")).toBe("soon – later");
  });
});

describe("arrow", () => {
  it("points along the writing direction", () => {
    expect(arrow("en-001")).toBe("→");
    expect(arrow("ar-001")).toBe("←");
    expect(arrow("ur")).toBe("←");
  });
});

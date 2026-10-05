import { describe, expect, it } from "vitest";
import { delta, paddedRange, trendDates } from "../../src/lib/ceo";

describe("CEO dashboard helpers", () => {
  it("plots month-ends, then today, oldest first", () => {
    expect(trendDates("2026-10-05", 3)).toEqual([
      "2026-07-31",
      "2026-08-31",
      "2026-09-30",
      "2026-10-05",
    ]);
    expect(trendDates("2026-10-05")).toHaveLength(7);
  });

  it("handles a year boundary and short months", () => {
    expect(trendDates("2026-02-10", 3)).toEqual([
      "2025-11-30",
      "2025-12-31",
      "2026-01-31",
      "2026-02-10",
    ]);
  });

  it("describes a change and never divides by zero", () => {
    expect(delta(110, 100)).toEqual({ abs: 10, pct: 10, direction: "up" });
    expect(delta(90, 100).direction).toBe("down");
    expect(delta(100, 100).direction).toBe("flat");
    expect(delta(5, 0).pct).toBeNull();
  });

  it("pads a range, and gives a flat series a band", () => {
    const [lo, hi] = paddedRange([10, 20]);
    expect(lo).toBeLessThan(10);
    expect(hi).toBeGreaterThan(20);
    expect(paddedRange([7, 7])).toEqual([6, 8]);
    expect(paddedRange([])).toEqual([0, 1]);
  });
});

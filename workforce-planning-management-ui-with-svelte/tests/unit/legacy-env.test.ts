// The settings rename (WPM-R278, WPM-D221): the UI's server reads the new name, then WPM_, then HCM_, warning once.

import { describe, expect, it, vi } from "vitest";
import { withLegacy } from "../../src/env";

describe("withLegacy", () => {
  const name = "WORKFORCE_PLANNING_MANAGEMENT_API_URL";

  it("uses the new name when it is set", () => {
    expect(withLegacy(name, "http://new", { WPM_API_URL: "http://old" })).toBe(
      "http://new",
    );
  });

  it("falls back to WPM_, then HCM_, and treats blanks as unset", () => {
    expect(
      withLegacy(name, "", {
        WPM_API_URL: "http://wpm",
        HCM_API_URL: "http://hcm",
      }),
    ).toBe("http://wpm");
    expect(
      withLegacy(name, " ", { WPM_API_URL: " ", HCM_API_URL: "http://hcm" }),
    ).toBe("http://hcm");
    expect(withLegacy(name, undefined, {})).toBe("");
  });

  it("warns once per old variable, and never for a name that is not ours", () => {
    const warn = vi.spyOn(console, "warn").mockImplementation(() => {});
    withLegacy("WORKFORCE_PLANNING_MANAGEMENT_PUBLIC_URL", "", {
      WPM_PUBLIC_URL: "https://a",
    });
    withLegacy("WORKFORCE_PLANNING_MANAGEMENT_PUBLIC_URL", "", {
      WPM_PUBLIC_URL: "https://a",
    });
    expect(warn).toHaveBeenCalledTimes(1);
    expect(withLegacy("AUTH_API_URL", "", { WPM_AUTH_API_URL: "x" })).toBe("");
    warn.mockRestore();
  });
});

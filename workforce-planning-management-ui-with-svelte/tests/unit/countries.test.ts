// The country picker's list (WPM-D152 question 3, WPM-T382): the same 249 ISO 3166-1 codes as the service, named in the
// page's locale by the browser, sorted in that locale, with the deployer's added codes and a held value never lost.

import { readFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import {
  countryName,
  countryOptions,
  ISO_3166_1,
} from "../../src/lib/countries";

describe("countries", () => {
  it("holds the service's ISO 3166-1 list", () => {
    const rust = readFileSync(
      join(
        __dirname,
        "../../../workforce-planning-management-service-with-rust/src/rules/data/iso-3166-1-alpha-2.txt",
      ),
      "utf8",
    )
      .split("\n")
      .map((l) => l.trim())
      .filter((l) => l && !l.startsWith("#"));
    expect([...ISO_3166_1]).toEqual(rust);
    expect(ISO_3166_1).toHaveLength(249);
  });

  it("names a country in the page's locale and sorts by that name", () => {
    expect(countryName("DE", "de-001")).toBe("Deutschland");
    expect(countryName("DE", "en-001")).toBe("Germany");
    expect(countryName("JP", "zh-001")).toBe("日本");
    const fr = countryOptions("fr-001").map((o) => o.code);
    // In French, "Allemagne" (DE) sorts before "Autriche" (AT) and both before "Espagne" (ES).
    expect(fr.indexOf("DE")).toBeLessThan(fr.indexOf("AT"));
    expect(fr.indexOf("AT")).toBeLessThan(fr.indexOf("ES"));
    expect(countryOptions("en-001")).toHaveLength(249);
  });

  it("adds the deployer's codes and keeps a held value", () => {
    const options = countryOptions(
      "en-001",
      [{ code: "XK", name: "Kosovo (deployer)" }],
      "qz",
    );
    const xk = options.find((o) => o.code === "XK");
    expect(xk?.added).toBe(true);
    expect(xk?.name).toBeTruthy();
    expect(options.find((o) => o.code === "QZ")).toBeDefined();
    expect(options).toHaveLength(251);
    // A code the browser cannot name falls back to the deployer's name, else the code.
    expect(countryName("QZ", "en-001", "Somewhere")).toBe("Somewhere");
    expect(countryName("QZ", "en-001")).toBe("QZ");
  });
});

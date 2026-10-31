// The address order and label table (WPM-D152 question 1, WPM-T382): each country's lines, its labels, the neutral
// order with no country, the free-text block for a country the table lacks, and nothing held is ever hidden.

import { readFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import {
  ADDRESS_FIELDS,
  ADDRESS_FORMATS,
  addressFormat,
  addressLayout,
  NEUTRAL_LINES,
} from "../../src/lib/address-formats";

const fields = (country: string | null, held = {}) =>
  addressLayout(country, held).lines.map((line) => line.map((p) => p.field));
const labels = (country: string) =>
  Object.fromEntries(
    addressLayout(country)
      .lines.flat()
      .map((p) => [p.field, p.label]),
  );

describe("address formats", () => {
  it("holds several orders: small to large, large to small, the postal code first, between or last", () => {
    expect(fields("US")).toEqual([
      ["address_line1"],
      ["address_line2"],
      ["city", "region", "postcode"],
    ]);
    expect(fields("GB")).toEqual([
      ["address_line1"],
      ["address_line2"],
      ["city"],
      ["postcode"],
    ]);
    expect(fields("DE")).toEqual([
      ["address_line1"],
      ["address_line2"],
      ["postcode", "city"],
    ]);
    expect(fields("JP")).toEqual([
      ["postcode"],
      ["region"],
      ["city"],
      ["address_line1"],
      ["address_line2"],
    ]);
    expect(fields("CN")).toEqual([
      ["postcode"],
      ["region", "city"],
      ["address_line1"],
      ["address_line2"],
    ]);
    expect(fields("KR")).toEqual([
      ["region", "city"],
      ["address_line1"],
      ["address_line2"],
      ["postcode"],
    ]);
    expect(fields("IN")).toEqual([
      ["address_line1"],
      ["address_line2"],
      ["city", "postcode"],
      ["region"],
    ]);
    expect(fields("BR")).toEqual([
      ["address_line1"],
      ["address_line2"],
      ["city", "region"],
      ["postcode"],
    ]);
  });

  it("names the parts as the country does", () => {
    expect(labels("US")).toMatchObject({
      city: "details.address.locality.city",
      region: "details.address.region.state",
      postcode: "details.address.postal.zip",
    });
    expect(labels("GB")).toMatchObject({
      city: "details.address.locality.post_town",
      postcode: "details.address.postal.postal",
    });
    expect(labels("JP").region).toBe("details.address.region.prefecture");
    expect(labels("CN").region).toBe("details.address.region.province");
    expect(labels("KR").region).toBe("details.address.region.do_si");
    expect(labels("IN").postcode).toBe("details.address.postal.pin");
    expect(labels("US").address_line1).toBe("details.contact.address_line1");
  });

  it("uses the neutral order with no country, and the free-text block for a country the table lacks", () => {
    const neutral = addressLayout("");
    expect(neutral.mode).toBe("neutral");
    expect(neutral.block).toBe(false);
    expect(fields(null)).toEqual(NEUTRAL_LINES);
    expect(neutral.lines.flat().map((p) => p.label)).toEqual(
      ADDRESS_FIELDS.map((f) => `details.contact.${f}`),
    );
    const lacking = addressLayout("NP");
    expect(lacking.mode).toBe("block");
    expect(lacking.block).toBe(true);
    expect(lacking.lines).toEqual([]);
    expect(addressLayout("jp").mode).toBe("table");
    expect(addressFormat(" us ")?.country).toBe("US");
  });

  it("never hides a field that holds a value", () => {
    // GB's format has no region; a held one is still shown, last, with the neutral label.
    const gb = addressLayout("GB", { region: "Yorkshire" });
    expect(gb.lines.at(-1)).toEqual([
      { field: "region", label: "details.contact.region" },
    ]);
    // Under the block, held structured fields stay.
    expect(fields("NP", { city: "Kathmandu", postcode: "" })).toEqual([
      ["city"],
    ]);
    // A table country still shows a held block.
    expect(addressLayout("JP", { address_text: "x" }).block).toBe(true);
    expect(addressLayout("JP", { address_text: "  " }).block).toBe(false);
  });

  it("labels every entry with a source and a status, uses each part once, and every label key exists", () => {
    const en = JSON.parse(
      readFileSync(
        join(__dirname, "../../content/locales/en-001/ui.json"),
        "utf8",
      ),
    ) as Record<string, unknown>;
    const get = (key: string) =>
      key
        .split(".")
        .reduce<unknown>(
          (node, part) => (node as Record<string, unknown> | undefined)?.[part],
          en,
        );
    const codes = ADDRESS_FORMATS.map((f) => f.country);
    expect(new Set(codes).size).toBe(codes.length);
    expect(codes.length).toBeGreaterThanOrEqual(10);
    for (const format of ADDRESS_FORMATS) {
      expect(format.country).toMatch(/^[A-Z]{2}$/);
      expect(format.status).toBe("unverified");
      expect(format.source).toContain("libaddressinput");
      expect(format.source).toContain("not read");
      const used = format.lines.flat();
      expect(new Set(used).size, format.country).toBe(used.length);
      expect(used.includes("region"), format.country).toBe(
        format.region !== undefined,
      );
      for (const placed of addressLayout(format.country).lines.flat()) {
        expect(
          typeof get(placed.label),
          `${format.country} ${placed.label}`,
        ).toBe("string");
      }
    }
  });
});

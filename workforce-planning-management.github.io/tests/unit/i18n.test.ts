import { describe, expect, it } from "vitest";
import { catalogues, fill, ui } from "../../src/lib/i18n.js";

describe("ui", () => {
  it("gives a locale its own catalogue", () => {
    expect(ui("zh-tw")).toBe(catalogues["zh-tw"]);
    expect(ui("pt-br")).toBe(catalogues["pt-br"]);
  });
  it("falls back by language, so cy-gb reads cy", () => {
    expect(ui("cy-gb")).toBe(catalogues.cy);
    expect(ui("IT-001")).toBe(catalogues.it);
  });
  it("falls back to English for an unknown, null or missing locale", () => {
    expect(ui("xx-yy")).toBe(catalogues.en);
    expect(ui(null)).toBe(catalogues.en);
    expect(ui(undefined)).toBe(catalogues.en);
  });
});

describe("fill", () => {
  it("fills placeholders, every occurrence", () => {
    expect(fill("{a} and {a} and {b}", { a: "1", b: "2" })).toBe("1 and 1 and 2");
  });
  it("leaves an unknown placeholder as written", () => {
    expect(fill("{a} {missing}", { a: "x" })).toBe("x {missing}");
  });
});

import { describe, expect, it } from "vitest";
import {
  BRIEF_LOCALES,
  COMPARE_SLUGS,
  DOC_PAGES,
  HOME_LOCALES,
  LOCALE_LABELS,
  briefRoute,
  hasHome,
  isBriefLocale,
  isRtl,
  localeLanding,
  localeOfPath,
  pickerLocales,
  routeOfRepoPath,
  searchAlternates,
} from "../../src/lib/site.js";

const never = () => false;
const always = () => true;
const homeless = BRIEF_LOCALES.find((l) => !hasHome(l));

describe("locale lists", () => {
  it("labels every brief locale and lists no locale twice", () => {
    for (const l of BRIEF_LOCALES) expect(LOCALE_LABELS[l]).toBeTruthy();
    expect(new Set(BRIEF_LOCALES).size).toBe(BRIEF_LOCALES.length);
    for (const h of HOME_LOCALES) expect(isBriefLocale(h)).toBe(true);
  });
  it("offers cy-001 in the picker and cy-gb only on its own pages", () => {
    expect(pickerLocales(null)).not.toContain("cy-gb");
    expect(pickerLocales("cy-gb")).toContain("cy-gb");
    expect(pickerLocales(null)).toContain("cy-001");
  });
  it("has a search page in each locale", () => {
    const alt = searchAlternates();
    for (const l of BRIEF_LOCALES) expect(alt[l]).toBe(`/${l}/search/`);
  });
});

describe("isBriefLocale, isRtl, hasHome, localeLanding", () => {
  it("knows its locales", () => {
    expect(isBriefLocale("it-001")).toBe(true);
    expect(isBriefLocale("xx")).toBe(false);
  });
  it("reads Arabic and Urdu right to left, by primary subtag", () => {
    expect(isRtl("ar-001")).toBe(true);
    expect(isRtl("UR_pk")).toBe(true);
    expect(isRtl("en-001")).toBe(false);
    expect(isRtl("")).toBe(false);
  });
  it("lands on the home when there is one, else on the briefs", () => {
    expect(hasHome("it-001")).toBe(true);
    expect(localeLanding("it-001")).toBe("/it-001/");
    if (homeless) expect(localeLanding(homeless)).toBe(`/${homeless}/for/`);
  });
});

describe("localeOfPath and briefRoute", () => {
  it("finds the locale in the first segment, after a base", () => {
    expect(localeOfPath("/it-001/for/x/")).toBe("it-001");
    expect(localeOfPath("/repo/it-001/", "/repo")).toBe("it-001");
    expect(localeOfPath("/it-001/", "/other")).toBe("it-001");
    expect(localeOfPath("/install/")).toBeNull();
    expect(localeOfPath("/")).toBeNull();
  });
  it("builds a brief route or the list route", () => {
    expect(briefRoute("it-001", "x")).toBe("/it-001/for/x/");
    expect(briefRoute("it-001", null)).toBe("/it-001/for/");
  });
});

describe("routeOfRepoPath", () => {
  it("routes the document pages and the spec", () => {
    for (const d of DOC_PAGES) expect(routeOfRepoPath(`${d.path}/`, never)).toBe(`/${d.slug}/`);
    expect(routeOfRepoPath("spec", never)).toBe("/spec/");
  });
  it("routes the comparisons", () => {
    expect(routeOfRepoPath("docs/compare", never)).toBe("/compare/");
    expect(routeOfRepoPath("docs/compare/index.md", never)).toBe("/compare/");
    const slug = COMPARE_SLUGS[0];
    expect(routeOfRepoPath(`docs/compare/${slug}/README.md`, never)).toBe(`/compare/${slug}/`);
    expect(routeOfRepoPath("docs/compare/unknown", never)).toBeNull();
  });
  it("routes a locale's start, path-to-production, privacy and home", () => {
    expect(routeOfRepoPath("locales/it-001/start/index.md", never)).toBe("/it-001/start/");
    expect(routeOfRepoPath("locales/it-001/path-to-production", never)).toBe("/it-001/path-to-production/");
    expect(routeOfRepoPath("locales/it-001/privacy/README.md", never)).toBe("/it-001/privacy/");
    expect(routeOfRepoPath("locales/it-001", never)).toBe("/it-001/");
    expect(routeOfRepoPath("locales/it-001/index.md", never)).toBe("/it-001/");
  });
  it("gives no route to a locale that does not exist or has no home", () => {
    expect(routeOfRepoPath("locales/xx/start", never)).toBeNull();
    expect(routeOfRepoPath("locales/xx/privacy", never)).toBeNull();
    expect(routeOfRepoPath("locales/xx/path-to-production", never)).toBeNull();
    expect(routeOfRepoPath("locales/xx/for", always)).toBeNull();
    if (homeless) expect(routeOfRepoPath(`locales/${homeless}`, never)).toBeNull();
  });
  it("routes a brief list, and a brief only if it exists", () => {
    expect(routeOfRepoPath("locales/it-001/for", never)).toBe("/it-001/for/");
    expect(routeOfRepoPath("locales/it-001/for/index.md", never)).toBe("/it-001/for/");
    expect(routeOfRepoPath("locales/it-001/for/README.md", never)).toBe("/it-001/for/");
    expect(routeOfRepoPath("locales/it-001/for/x", always)).toBe("/it-001/for/x/");
    expect(routeOfRepoPath("locales/it-001/for/x/", always)).toBe("/it-001/for/x/");
    expect(routeOfRepoPath("locales/it-001/for/x", never)).toBeNull();
  });
  it("gives no route to anything else", () => {
    expect(routeOfRepoPath("src/main.rs", always)).toBeNull();
  });
});

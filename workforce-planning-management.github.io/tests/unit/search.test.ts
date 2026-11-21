import { describe, expect, it } from "vitest";
import { search, searchTerms, type SearchDoc } from "../../src/lib/search.js";

const doc = (route: string, title: string, text: string, locale = "en"): SearchDoc => ({ route, title, text, locale });

describe("searchTerms", () => {
  it("folds case and compatibility forms, and splits on any whitespace", () => {
    expect(searchTerms("  Ｆoo\tBAR\nbaz ")).toEqual(["foo", "bar", "baz"]);
  });
  it("has no words for a query of spaces", () => {
    expect(searchTerms("   ")).toEqual([]);
  });
});

describe("search", () => {
  const docs = [
    doc("/a/", "Install", "Run the installer. The installer needs a database."),
    doc("/b/", "Database", "A database holds the data. Install it first."),
    doc("/c/", "Other", "Nothing relevant here."),
  ];

  it("returns nothing for an empty query", () => {
    expect(search(docs, "  ")).toEqual([]);
  });

  it("requires every word, in the title or the text", () => {
    expect(search(docs, "install database").map((h) => h.route)).toEqual(["/a/", "/b/"]);
    expect(search(docs, "install zebra")).toEqual([]);
  });

  it("ranks a title match first, then the page with more matches", () => {
    expect(search(docs, "install").map((h) => h.route)).toEqual(["/a/", "/b/"]);
    const more = [doc("/x/", "X", "cat"), doc("/y/", "Y", "cat cat cat")];
    expect(search(more, "cat").map((h) => h.route)).toEqual(["/y/", "/x/"]);
  });

  it("keeps the index order for ties", () => {
    const tie = [doc("/1/", "T", "word"), doc("/2/", "T", "word")];
    expect(search(tie, "word").map((h) => h.route)).toEqual(["/1/", "/2/"]);
  });

  it("matches a title-only word and gives a snippet from the start", () => {
    const [hit] = search([doc("/t/", "Zebra", "plain text")], "zebra");
    expect(hit).toEqual({ route: "/t/", title: "Zebra", locale: "en", snippet: "plain text" });
  });

  it("cuts the snippet with an ellipsis on each side that was cut", () => {
    const text = `${"a ".repeat(100)}needle${" b".repeat(100)}`;
    const [hit] = search([doc("/s/", "S", text)], "needle");
    expect(hit?.snippet.startsWith("… ")).toBe(true);
    expect(hit?.snippet.endsWith(" …")).toBe(true);
    expect(hit?.snippet).toContain("needle");
  });

  it("cuts only the end when the match is near the start", () => {
    const [hit] = search([doc("/s/", "S", `needle ${"b ".repeat(200)}`)], "needle");
    expect(hit?.snippet.startsWith("…")).toBe(false);
    expect(hit?.snippet.endsWith(" …")).toBe(true);
  });
});

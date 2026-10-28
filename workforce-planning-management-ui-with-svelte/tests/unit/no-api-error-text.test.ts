// No page shows API error text. A failed call is described by the catalogue's words for its status (`errorText` in
// `src/lib/write.ts`); the service's own message is English, written for developers, and may name a table or a rule. So a page,
// a component or a loader must not read an error's message or body; reading an `ApiError`'s status to decide what to do is fine.
// Server-side code that carries a response through is exempt.

import { readdirSync, readFileSync, statSync } from "node:fs";
import { join, relative } from "node:path";
import { describe, expect, it } from "vitest";

const SRC = join(__dirname, "../../src");

/** Where an `ApiError` is made, thrown and read on purpose. */
const EXEMPT = [
  /^lib\/api\//,
  /^lib\/write\.ts$/,
  /^lib\/server\//,
  /^hooks\.server\.ts$/,
  /\+server\.ts$/,
];

function files(dir: string): string[] {
  return readdirSync(dir).flatMap((name) => {
    const path = join(dir, name);
    if (statSync(path).isDirectory()) return files(path);
    return /\.(svelte|ts)$/.test(name) && !name.endsWith(".d.ts") ? [path] : [];
  });
}

/** The text of the block that opens at `open` (the index of its `{`), by counting braces. */
function block(source: string, open: number): string {
  let depth = 0;
  for (let i = open; i < source.length; i++) {
    if (source[i] === "{") depth++;
    if (source[i] === "}" && --depth === 0) return source.slice(open, i + 1);
  }
  return source.slice(open);
}

/** Uses of a caught error `name` that would show or carry its text. */
export function reads(name: string, text: string): boolean {
  const n = name.replace(/[$]/g, "\\$&");
  return new RegExp(
    `\\b${n}\\s*\\.\\s*(message|body|description|detail)\\b|String\\(\\s*${n}\\s*\\)|\\$\\{\\s*${n}\\s*\\}|\\{\\s*${n}\\s*\\}|\\b${n}\\s*\\?\\.\\s*(message|body)\\b`,
  ).test(text);
}

/** The places in `source` where a caught error's text is read. */
export function violations(source: string): string[] {
  const out: string[] = [];
  for (const m of source.matchAll(
    /catch\s*\(\s*([A-Za-z_$][\w$]*)\s*\)\s*\{/g,
  )) {
    const open = (m.index ?? 0) + m[0].length - 1;
    if (reads(m[1] ?? "", block(source, open))) out.push(`catch (${m[1]})`);
  }
  for (const m of source.matchAll(
    /\.catch\(\s*\(?\s*([A-Za-z_$][\w$]*)\s*\)?\s*=>/g,
  )) {
    const from = (m.index ?? 0) + m[0].length;
    if (reads(m[1] ?? "", source.slice(from, from + 240)))
      out.push(`.catch(${m[1]} =>`);
  }
  return out;
}

describe("no page shows API error text", () => {
  const all = files(SRC).map((f) => [relative(SRC, f), f] as const);

  it("scans the whole UI source", () => {
    expect(all.length).toBeGreaterThan(100);
  });

  it("no page, component or loader reads a caught error's message or body", () => {
    const bad = all
      .filter(([rel]) => !EXEMPT.some((e) => e.test(rel)))
      .flatMap(([rel, path]) =>
        violations(readFileSync(path, "utf8")).map((v) => `${rel}: ${v}`),
      );
    expect(bad, bad.join("\n")).toEqual([]);
  });

  it("a page that handles an ApiError reads its status, never its message or body", () => {
    const bad = all
      .filter(([rel]) => !EXEMPT.some((e) => e.test(rel)))
      .filter(([, path]) => /\bApiError\b/.test(readFileSync(path, "utf8")))
      .filter(([, path]) =>
        /\b(cause|e|err|error|ex)\s*\??\.\s*(message|body|description)\b/.test(
          readFileSync(path, "utf8"),
        ),
      )
      .map(([rel]) => rel);
    expect(bad, bad.join("\n")).toEqual([]);
  });

  describe("the detector itself", () => {
    it("finds a message read in a catch block", () => {
      expect(
        violations("try { a() } catch (e) { error = e.message; }"),
      ).toEqual(["catch (e)"]);
      expect(
        violations("try { a() } catch (err) { show(String(err)); }"),
      ).toEqual(["catch (err)"]);
      expect(
        violations("try { a() } catch (e) { msg = `bad: ${e}`; }"),
      ).toEqual(["catch (e)"]);
      expect(
        violations("try { a() } catch (e) { if (x) { y = e?.body; } }"),
      ).toEqual(["catch (e)"]);
    });

    it("finds one in a promise catch", () => {
      expect(violations("p.catch((e) => (error = e.message))")).toEqual([
        ".catch(e =>",
      ]);
    });

    it("lets a catch that only uses the catalogue through", () => {
      expect(
        violations("try { a() } catch (e) { error = errorText(e); }"),
      ).toEqual([]);
      expect(violations("try { a() } catch { error = 'x'; }")).toEqual([]);
      expect(violations("p.catch(() => null)")).toEqual([]);
      expect(
        violations(
          "try { a() } catch (e) { f = failure(e); } const e2 = { message: 1 };",
        ),
      ).toEqual([]);
    });

    it("does not mistake a similar name", () => {
      expect(violations("try { a() } catch (e) { x = err.message; }")).toEqual(
        [],
      );
    });
  });
});

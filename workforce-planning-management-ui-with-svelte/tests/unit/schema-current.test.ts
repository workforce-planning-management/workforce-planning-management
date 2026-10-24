// The generated types match the OpenAPI document they come from (WPM-T263). The document itself is kept
// current by a service test (`the_ui_copy_is_current`); this one keeps `schema.d.ts` current with it.
// To fix a failure: `pnpm api:types`.

import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import openapiTS, { astToString } from "openapi-typescript";
import { describe, expect, it } from "vitest";

const here = dirname(fileURLToPath(import.meta.url));
const api = join(here, "..", "..", "src", "lib", "api");

describe("schema.d.ts", () => {
  it("is what openapi-typescript makes of openapi.json today", async () => {
    const ast = await openapiTS(new URL(`file://${join(api, "openapi.json")}`));
    const have = readFileSync(join(api, "schema.d.ts"), "utf8");
    // The command-line tool puts a three-line comment and a blank line before the generated text.
    expect(have.endsWith(astToString(ast)), "run `pnpm api:types`").toBe(true);
  });

  it("states an answer for every call the UI types from it", () => {
    const doc = JSON.parse(readFileSync(join(api, "openapi.json"), "utf8"));
    const wpm = readFileSync(join(api, "wpm.ts"), "utf8");
    for (const m of wpm.matchAll(
      /Answer<\s*((?:"[^"]+"\s*\|?\s*)+),\s*"(\w+)"\s*>/g,
    )) {
      for (const p of m[1]!.matchAll(/"([^"]+)"/g)) {
        const op = doc.paths[p[1]!]?.[m[2]!];
        expect(op, `${m[2]} ${p[1]} is not in openapi.json`).toBeDefined();
      }
    }
  });
});

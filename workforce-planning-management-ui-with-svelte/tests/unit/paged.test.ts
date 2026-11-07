import { describe, expect, it } from "vitest";
import { readNext, readPages, type PageReader } from "../../src/lib/paged";

/** A service with `n` rows that hands out a cursor while more follow; `seen` records the cursors it was sent. */
const service =
  (n: number, seen: (string | null)[] = []): PageReader<number> =>
  async (limit, cursor) => {
    seen.push(cursor);
    const from = cursor === null ? 0 : Number(cursor.slice(1));
    const items = Array.from(
      { length: Math.max(0, Math.min(limit, n - from)) },
      (_, i) => from + i,
    );
    return { items, next: from + limit < n ? `c${from + limit}` : null };
  };

describe("paged lists", () => {
  it("reads the first page, and knows there is more from the cursor", async () => {
    expect(await readPages(service(7), 1, 3)).toEqual({
      items: [0, 1, 2],
      next: "c3",
      more: true,
    });
  });

  it("re-reads every page held by following each cursor, and stops at the last", async () => {
    const seen: (string | null)[] = [];
    expect((await readPages(service(7, seen), 3, 3)).items).toEqual([
      0, 1, 2, 3, 4, 5, 6,
    ]);
    expect(seen).toEqual([null, "c3", "c6"]);
    const short = await readPages(service(4), 5, 3);
    expect(short).toEqual({ items: [0, 1, 2, 3], next: null, more: false });
  });

  it("adds the page the cursor points to after what is held", async () => {
    const first = await readPages(service(7), 1, 3);
    const next = await readNext(service(7), first, 3);
    expect(next).toEqual({ items: [0, 1, 2, 3, 4, 5], next: "c6", more: true });
    expect((await readNext(service(7), next, 3)).more).toBe(false);
  });

  it("a full last page is the end when the service gives no cursor", async () => {
    expect((await readPages(service(6), 2, 3)).more).toBe(false);
  });

  it("asks for nothing past the end", async () => {
    const seen: (string | null)[] = [];
    const end = { items: [1], next: null, more: false };
    expect(await readNext(service(1, seen), end, 3)).toBe(end);
    expect(seen).toEqual([]);
  });
});

import { describe, expect, it } from "vitest";
import {
  hasMore,
  readNext,
  readPages,
  type PageReader,
} from "../../src/lib/paged";

/** A service with `n` rows and a page size of 3, which says its total unless told not to. */
const service =
  (n: number, says = true): PageReader<number> =>
  async (limit, offset) => ({
    items: Array.from(
      { length: Math.max(0, Math.min(limit, n - offset)) },
      (_, i) => offset + i,
    ),
    total: says ? n : null,
    limit,
    offset,
  });

describe("paged lists", () => {
  it("knows there is more from the total, and from a full page when there is no total", () => {
    expect(hasMore(3, 7, 3, 3)).toBe(true);
    expect(hasMore(7, 7, 1, 3)).toBe(false);
    expect(hasMore(3, null, 3, 3)).toBe(true);
    expect(hasMore(2, null, 2, 3)).toBe(false);
    expect(hasMore(0, 0, 0, 3)).toBe(false);
  });

  it("reads the first page, and says how many there are", async () => {
    expect(await readPages(service(7), 1, 3)).toEqual({
      items: [0, 1, 2],
      total: 7,
      more: true,
    });
  });

  it("re-reads every page held, and stops at a short one", async () => {
    expect((await readPages(service(7), 3, 3)).items).toEqual([
      0, 1, 2, 3, 4, 5, 6,
    ]);
    const short = await readPages(service(4), 5, 3);
    expect(short).toEqual({ items: [0, 1, 2, 3], total: 4, more: false });
  });

  it("adds the next page after what is held", async () => {
    const next = await readNext(service(7), [0, 1, 2], 3);
    expect(next).toEqual({ items: [0, 1, 2, 3, 4, 5], total: 7, more: true });
    expect((await readNext(service(7), next.items, 3)).more).toBe(false);
  });

  it("cannot tell the end from a full page when the total is missing", async () => {
    expect((await readPages(service(6, false), 1, 3)).more).toBe(true);
    expect((await readNext(service(6, false), [0, 1, 2], 3)).more).toBe(true);
    expect(
      (await readNext(service(6, false), [0, 1, 2, 3, 4, 5], 3)).more,
    ).toBe(false);
  });
});

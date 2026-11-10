import { describe, expect, it } from "vitest";
import { apiPage, nextCursor } from "../../src/lib/api/client";

const answering = (body: unknown, headers: Record<string, string>) =>
  (async () =>
    new Response(JSON.stringify(body), {
      status: 200,
      headers: { "content-type": "application/json", ...headers },
    })) as unknown as typeof fetch;

describe("apiPage", () => {
  it("reads the rows and the next page's cursor from the Link header", async () => {
    const page = await apiPage<number>("/x", {
      fetch: answering([1, 2], {
        link: '</api/x?status=open&limit=2&cursor=AbC-_9>; rel="next"',
      }),
    });
    expect(page).toEqual({ items: [1, 2], next: "AbC-_9" });
  });

  it("is the last page when there is no Link", async () => {
    expect(await apiPage<number>("/x", { fetch: answering([], {}) })).toEqual({
      items: [],
      next: null,
    });
  });

  it("ignores totals: the service sends none, and one sent is not believed", async () => {
    const page = await apiPage<number>("/x", {
      fetch: answering([1], { "x-total-count": "7" }),
    });
    expect(page).toEqual({ items: [1], next: null });
  });
});

describe("nextCursor", () => {
  it("takes only rel=next, among several links", () => {
    expect(
      nextCursor('</a?cursor=p>; rel="prev", </a?cursor=n>; rel="next"'),
    ).toBe("n");
    expect(nextCursor('</a?cursor=p>; rel="prev"')).toBeNull();
    expect(nextCursor('</a?cursor=n>; rel="nextish"')).toBeNull();
  });

  it("keeps a target whose query holds a comma", () => {
    expect(
      nextCursor('</api/workers?fields=pid,display_name&cursor=k>; rel="next"'),
    ).toBe("k");
  });

  it("is null for a next link with no cursor, or no header", () => {
    expect(nextCursor('</a?limit=2>; rel="next"')).toBeNull();
    expect(nextCursor(null)).toBeNull();
    expect(nextCursor("")).toBeNull();
  });
});

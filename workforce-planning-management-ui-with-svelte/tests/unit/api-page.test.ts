import { describe, expect, it } from "vitest";
import { apiPage } from "../../src/lib/api/client";

const answering = (body: unknown, headers: Record<string, string>) =>
  (async () =>
    new Response(JSON.stringify(body), {
      status: 200,
      headers: { "content-type": "application/json", ...headers },
    })) as unknown as typeof fetch;

describe("apiPage", () => {
  it("reads the rows and what the service said about the whole list", async () => {
    const page = await apiPage<number>("/x", {
      fetch: answering([1, 2], {
        "x-total-count": "7",
        "x-limit": "2",
        "x-offset": "4",
      }),
    });
    expect(page).toEqual({ items: [1, 2], total: 7, limit: 2, offset: 4 });
  });

  it("says nothing is known when a header is missing, never zero", async () => {
    const page = await apiPage<number>("/x", { fetch: answering([], {}) });
    expect(page).toEqual({ items: [], total: null, limit: null, offset: 0 });
  });

  it("treats a header that is not a number as unknown", async () => {
    const page = await apiPage<number>("/x", {
      fetch: answering([], { "x-total-count": "many" }),
    });
    expect(page.total).toBeNull();
  });
});

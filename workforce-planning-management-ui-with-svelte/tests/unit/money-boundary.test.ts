import { describe, expect, it, vi } from "vitest";
import { api, assertExactMinorUnits } from "#lib/api/client.js";
import { MAX_MINOR_UNITS } from "#lib/money.js";

describe("exact minor units at the API boundary (WPM-T552, WPM-D206)", () => {
  it("allows exact create, edit, and exported values", async () => {
    const amounts = ["JPY", "EUR", "KWD", "CLF"].map((currency) => ({
      currency,
      amount_minor: MAX_MINOR_UNITS,
    }));
    const fetch = vi.fn<typeof globalThis.fetch>(async (_input, init) => {
      expect(JSON.parse(String(init?.body))).toEqual({ amounts });
      return new Response(JSON.stringify({ export: amounts }), { status: 200 });
    });
    await expect(
      api<{ export: typeof amounts }>("/exact", {
        method: "POST",
        body: { amounts },
        fetch,
      }),
    ).resolves.toEqual({ export: amounts });
    expect(fetch).toHaveBeenCalledOnce();
  });

  it("sends no request for an unsafe write", async () => {
    const fetch = vi.fn<typeof globalThis.fetch>();
    await expect(
      api("/unsafe", {
        method: "POST",
        body: { amount_minor: MAX_MINOR_UNITS + 1 },
        fetch,
      }),
    ).rejects.toThrow("amount_minor is outside the exact JSON integer range");
    expect(fetch).not.toHaveBeenCalled();
  });

  it("refuses an unsafe read before a consumer can format or export it", async () => {
    const fetch = vi.fn<typeof globalThis.fetch>(
      async () =>
        new Response('{"salary_minor":9007199254740992}', { status: 200 }),
    );
    await expect(api("/unsafe", { fetch })).rejects.toThrow(
      "salary_minor is outside the exact JSON integer range",
    );
  });

  it("checks nested arrays and permits null unknowns", () => {
    expect(() =>
      assertExactMinorUnits({ rows: [{ fee_minor: null }] }),
    ).not.toThrow();
    expect(() =>
      assertExactMinorUnits({
        rows: [{ fee_minor: Number.MIN_SAFE_INTEGER - 1 }],
      }),
    ).toThrow("fee_minor is outside the exact JSON integer range");
  });

  it("accepts the service's typed wrapper of an aggregate, and checks the number inside it", () => {
    // The service sends a figure that may be withheld as `{ status, value }` under a `*_minor` key.
    expect(() =>
      assertExactMinorUnits({
        female_average_minor: { status: "shown", value: 3_900_000 },
        male_average_minor: { status: "withheld" },
      }),
    ).not.toThrow();
    expect(() =>
      assertExactMinorUnits({
        female_average_minor: { status: "shown", value: MAX_MINOR_UNITS + 1 },
      }),
    ).toThrow("female_average_minor is outside the exact JSON integer range");
    expect(() => assertExactMinorUnits({ salary_minor: "3900000" })).toThrow(
      "salary_minor is outside the exact JSON integer range",
    );
  });
});

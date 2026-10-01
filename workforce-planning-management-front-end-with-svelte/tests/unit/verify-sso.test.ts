// Unit tests for the /verify/sso BFF load function — mirrors
// tests/unit/verify.test.ts exactly, one query param renamed
// (`code` not `token`) and one error message. See that file's own
// comment for why this bypasses Playwright entirely.
import { describe, expect, it, vi } from "vitest";
import { load } from "../../src/routes/verify/sso/+page.server";

const cookies = { set: vi.fn() } as unknown as Parameters<
  typeof load
>[0]["cookies"];

describe("verify/sso load", () => {
  it("returns missingToken when no code is present", async () => {
    const result = await load({
      url: new URL("http://localhost/verify/sso"),
      fetch: vi.fn(),
      cookies,
    } as unknown as Parameters<typeof load>[0]);
    expect(result).toEqual({
      error: "missingToken",
      title: "Sign-in link — Workforce Planning Management",
    });
  });

  it("returns serviceUnavailable when the upstream fetch rejects", async () => {
    const result = await load({
      url: new URL("http://localhost/verify/sso?code=abc"),
      fetch: vi.fn().mockRejectedValue(new Error("fetch failed")),
      cookies,
    } as unknown as Parameters<typeof load>[0]);
    expect(result).toEqual({ error: "serviceUnavailable" });
  });

  it("returns invalidToken when the upstream responds not-ok", async () => {
    const result = await load({
      url: new URL("http://localhost/verify/sso?code=abc"),
      fetch: vi.fn().mockResolvedValue(new Response(null, { status: 401 })),
      cookies,
    } as unknown as Parameters<typeof load>[0]);
    expect(result).toEqual({
      error: "invalidToken",
      title: "Sign-in link — Workforce Planning Management",
    });
  });

  it("sets the session cookie and redirects home on success", async () => {
    const upstream = new Response(null, {
      status: 200,
      headers: { "set-cookie": "__Host-mxi_session=sid123; Path=/" },
    });
    await expect(
      load({
        url: new URL("http://localhost/verify/sso?code=abc"),
        fetch: vi.fn().mockResolvedValue(upstream),
        cookies,
      } as unknown as Parameters<typeof load>[0]),
    ).rejects.toMatchObject({ status: 303, location: "/" });
    expect(cookies.set).toHaveBeenCalledWith(
      "__Host-mxi_session",
      "sid123",
      expect.objectContaining({ httpOnly: true }),
    );
  });
});

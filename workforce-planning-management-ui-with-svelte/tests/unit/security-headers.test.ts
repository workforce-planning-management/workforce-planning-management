import { describe, expect, it } from "vitest";
import { securityHeaders } from "#lib/security-headers.js";

describe("securityHeaders", () => {
  it("always sets the baseline", () => {
    const h = securityHeaders(new URL("http://localhost/en-001/"), null);
    expect(h["x-content-type-options"]).toBe("nosniff");
    expect(h["x-frame-options"]).toBe("DENY");
    expect(h["referrer-policy"]).toBe("strict-origin-when-cross-origin");
    expect(h["permissions-policy"]).toContain("camera=()");
    expect(h["cross-origin-opener-policy"]).toBe("same-origin");
    expect(h["cross-origin-resource-policy"]).toBe("same-origin");
  });

  it("adds HSTS only over TLS, directly or as a proxy reports it", () => {
    const plain = securityHeaders(new URL("http://localhost/"), null);
    expect(plain["strict-transport-security"]).toBeUndefined();
    const direct = securityHeaders(new URL("https://example.test/"), null);
    expect(direct["strict-transport-security"]).toContain("max-age=");
    const proxied = securityHeaders(new URL("http://localhost/"), "HTTPS");
    expect(proxied["strict-transport-security"]).toContain("includeSubDomains");
    const http = securityHeaders(new URL("http://localhost/"), "http");
    expect(http["strict-transport-security"]).toBeUndefined();
  });

  it("lets the content manager's sign-in popup talk to its opener", () => {
    for (const path of [
      "/admin",
      "/admin/",
      "/en-001/admin/",
      "/cy-001/admin",
    ]) {
      expect(
        securityHeaders(new URL(`http://localhost${path}`), null)[
          "cross-origin-opener-policy"
        ],
      ).toBe("same-origin-allow-popups");
    }
    // Only the admin path: a page that merely contains the word is isolated.
    expect(
      securityHeaders(new URL("http://localhost/en-001/administrators"), null)[
        "cross-origin-opener-policy"
      ],
    ).toBe("same-origin");
  });

  it("never caches the BFF proxy, and leaves pages alone", () => {
    expect(
      securityHeaders(new URL("http://localhost/api/proxy/workers"), null)[
        "cache-control"
      ],
    ).toBe("no-store");
    expect(
      securityHeaders(new URL("http://localhost/en-001/"), null)[
        "cache-control"
      ],
    ).toBeUndefined();
  });
});

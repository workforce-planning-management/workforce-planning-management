// The content manager's page (WPM-T249): a pinned script with its integrity hash, and a policy of its own.

import { createHash } from "node:crypto";
import { describe, expect, it } from "vitest";
import { GET } from "../../src/routes/admin/+server";

async function page(path: string) {
  const response = await GET({
    url: new URL(`http://ui.test${path}`),
  } as unknown as Parameters<typeof GET>[0]);
  return { response, html: await response.text() };
}

describe("the content manager's page", () => {
  it("loads the CMS at an exact version with its integrity hash", async () => {
    const { html } = await page("/en-001/admin/");
    const tag = html.match(
      /<script src="([^"]+)" integrity="([^"]+)" crossorigin="anonymous"><\/script>/,
    );
    expect(tag, "the CMS script tag").not.toBeNull();
    expect(tag?.[1]).toMatch(
      /^https:\/\/unpkg\.com\/@sveltia\/cms@\d+\.\d+\.\d+\/dist\/sveltia-cms\.js$/,
    );
    expect(tag?.[2]).toMatch(/^sha384-[A-Za-z0-9+/]{64}$/);
  });

  it("sends a policy that allows the inline script by its hash and nothing inline otherwise", async () => {
    for (const path of ["/en-001/admin/", "/cy-001/admin/", "/ar-001/admin/"]) {
      const { response, html } = await page(path);
      const csp = response.headers.get("content-security-policy") ?? "";
      const inline = html.match(/<script>([\s\S]*?)<\/script>/)?.[1] ?? "";
      const hash = createHash("sha256").update(inline).digest("base64");
      const scriptSrc =
        csp.split("; ").find((d) => d.startsWith("script-src ")) ?? "";
      expect(scriptSrc, path).toContain(`'sha256-${hash}'`);
      expect(scriptSrc, path).not.toContain("'unsafe-inline'");
      expect(scriptSrc, path).not.toContain("'unsafe-eval'");
      expect(csp, path).toContain("frame-ancestors 'none'");
      expect(csp, path).toContain("object-src 'none'");
      expect(csp, path).toMatch(/connect-src [^;]*https:\/\/api\.github\.com/);
    }
  });
});

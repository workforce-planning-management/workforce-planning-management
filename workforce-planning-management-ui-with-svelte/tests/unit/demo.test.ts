// The demo (WPM-T118 follow-up): the example address opens a fictitious-data session, and that session can
// reach the demo service and nothing else.

import { afterEach, describe, expect, it, vi } from "vitest";
import { DEMO_STEPS, stepAt } from "../../src/lib/demoTour";

describe("the demo session", () => {
  it("is recognised by its prefix, and the example address by its text, not its case", async () => {
    const { isDemoSession, isDemoEmail, newDemoSessionId } =
      await import("../../src/lib/server/demo");
    expect(isDemoEmail("example@example.com")).toBe(true);
    expect(isDemoEmail("  Example@Example.COM ")).toBe(true);
    expect(isDemoEmail("example@example.org")).toBe(false);
    expect(isDemoEmail("someone@example.com")).toBe(false);
    const id = newDemoSessionId();
    expect(isDemoSession(id)).toBe(true);
    expect(isDemoSession("0123456789abcdef")).toBe(false);
    expect(isDemoSession(null)).toBe(false);
    expect(isDemoSession(undefined)).toBe(false);
    expect(newDemoSessionId()).not.toBe(id);
  });
});

describe("the proxy and a demo session", () => {
  afterEach(() => {
    vi.resetModules();
    vi.doUnmock("../../src/lib/server/config");
  });

  async function call(demoUrl: string, sessionId: string | null) {
    vi.resetModules();
    vi.doMock("../../src/lib/server/config", () => ({
      WPM_API_URL: "http://real.test",
      WPM_DEMO_API_URL: demoUrl,
      AUTH_API_URL: "http://auth.test",
      WPM_PUBLIC_URL: "",
    }));
    const { GET } =
      await import("../../src/routes/api/proxy/[...path]/+server");
    const seen: Array<{ url: string; auth: string | null }> = [];
    const fetchStub = (async (url: string | URL, init?: RequestInit) => {
      seen.push({
        url: String(url),
        auth: new Headers(init?.headers).get("authorization"),
      });
      // The session-to-token exchange would answer here; anything else is the forwarded call.
      return new Response(JSON.stringify({ token: "real-token" }), {
        status: 200,
      });
    }) as typeof fetch;
    const response = await GET({
      request: new Request("http://ui.test/api/proxy/workers"),
      params: { path: "workers" },
      url: new URL("http://ui.test/api/proxy/workers"),
      locals: { sessionId },
      fetch: fetchStub,
    } as unknown as Parameters<typeof GET>[0]);
    return { response, seen };
  }

  it("goes to the demo service, with no token and no exchange at the authentication service", async () => {
    const { response, seen } = await call("http://demo.test", "demo:abc");
    expect(response.status).toBe(200);
    expect(seen).toEqual([{ url: "http://demo.test/api/workers", auth: null }]);
  });

  it("makes up the demo user's memberships from the demo service's people, as an administrator of its organization", async () => {
    vi.resetModules();
    vi.doMock("../../src/lib/server/config", () => ({
      WPM_API_URL: "http://real.test",
      WPM_DEMO_API_URL: "http://demo.test",
      AUTH_API_URL: "http://auth.test",
      WPM_PUBLIC_URL: "",
    }));
    const { GET } =
      await import("../../src/routes/api/proxy/[...path]/+server");
    const people = [
      {
        pid: "w1",
        organization_ref: "organization:abc",
        employment_type: "contractor",
        status: "active",
        hired_on: "2026-01-05",
      },
      {
        pid: "w2",
        organization_ref: "organization:abc",
        employment_type: "permanent",
        status: "active",
        hired_on: "2025-02-03",
      },
    ];
    const asked: string[] = [];
    const fetchStub = (async (url: string | URL) => {
      asked.push(String(url));
      // The parent that contains the organization.
      if (String(url).includes("organization-confederations")) {
        return new Response(
          JSON.stringify([{ parent_organization_ref: "organization:parent" }]),
          { status: 200 },
        );
      }
      return new Response(JSON.stringify(people), { status: 200 });
    }) as typeof fetch;
    const ask = (path: string) =>
      GET({
        request: new Request(`http://ui.test/api/proxy/${path}`),
        params: { path },
        url: new URL(`http://ui.test/api/proxy/${path}`),
        locals: { sessionId: "demo:abc" },
        fetch: fetchStub,
      } as unknown as Parameters<typeof GET>[0]);
    const memberships = await (await ask("me/organizations")).json();
    // The organization with people, and the parent that contains it; "me" is in the first.
    expect(memberships).toHaveLength(2);
    expect(memberships[0]).toMatchObject({
      organization_ref: "organization:abc",
      worker_pid: "w2",
      role: "hr_admin",
    });
    expect(memberships[1]).toMatchObject({
      organization_ref: "organization:parent",
      worker_pid: null,
    });
    expect(await (await ask("me/organizations/scope")).json()).toEqual([
      "organization:abc",
      "organization:parent",
    ]);
    expect(asked.every((u) => u.startsWith("http://demo.test/"))).toBe(true);
  });

  it("opens nothing when no demo is configured: refused, and nothing is sent anywhere", async () => {
    const { response, seen } = await call("", "demo:abc");
    expect(response.status).toBe(401);
    expect(seen).toEqual([]);
  });

  it("leaves an ordinary session on the real service", async () => {
    const { seen } = await call("http://demo.test", "opaque-session-id");
    expect(seen.some((s) => s.url.startsWith("http://demo.test"))).toBe(false);
    expect(seen.some((s) => s.url === "http://real.test/api/workers")).toBe(
      true,
    );
  });
});

describe("the guided tour order", () => {
  it("lists every page once, and finds the step before and after", () => {
    const paths = DEMO_STEPS.map((s) => s.path);
    expect(new Set(paths).size).toBe(paths.length);
    const first = stepAt(paths[0]!)!;
    expect(first.previous).toBeNull();
    expect(first.next?.path).toBe(paths[1]);
    const last = stepAt(paths.at(-1)!)!;
    expect(last.next).toBeNull();
    expect(last.index + 1).toBe(last.total);
    expect(stepAt("/workers/some-id")?.index).toBe(paths.indexOf("/workers"));
    expect(stepAt("/signin")).toBeNull();
  });
});

describe("signing in as the example address", () => {
  afterEach(() => {
    vi.resetModules();
    vi.doUnmock("../../src/lib/server/config");
  });

  async function signIn(demoUrl: string, email: string) {
    vi.resetModules();
    vi.doMock("../../src/lib/server/config", () => ({
      WPM_API_URL: "http://real.test",
      WPM_DEMO_API_URL: demoUrl,
      AUTH_API_URL: "http://auth.test",
      WPM_PUBLIC_URL: "",
    }));
    const { actions } = await import("../../src/routes/signin/+page.server");
    const set: Array<[string, string]> = [];
    const asked: string[] = [];
    const form = new FormData();
    form.set("email", email);
    let outcome: unknown;
    try {
      outcome = await actions.default!({
        request: new Request("http://ui.test/en-001/signin", {
          method: "POST",
          body: form,
        }),
        fetch: (async (url: string | URL) => {
          asked.push(String(url));
          return new Response("{}", { status: 200 });
        }) as typeof fetch,
        url: new URL("http://ui.test/en-001/signin"),
        cookies: {
          set: (name: string, value: string) => set.push([name, value]),
        },
      } as unknown as Parameters<NonNullable<typeof actions.default>>[0]);
    } catch (thrown) {
      outcome = thrown;
    }
    return { outcome, set, asked };
  }

  it("opens the demo: a demo session cookie and a redirect to the guided tour, and no email is sent", async () => {
    const { outcome, set, asked } = await signIn(
      "http://demo.test",
      " Example@Example.com ",
    );
    expect(outcome).toMatchObject({ status: 303, location: "/en-001/demo" });
    expect(set).toHaveLength(1);
    expect(set[0]![0]).toBe("__Host-mxi_session");
    expect(set[0]![1]).toMatch(/^demo:/);
    expect(asked).toEqual([]);
  });

  it("is an ordinary sign-in where no demo is offered: a magic link is asked for, and no cookie is set", async () => {
    const { set, asked } = await signIn("", "example@example.com");
    expect(set).toEqual([]);
    expect(asked.length).toBe(1);
    expect(asked[0]).toContain("http://auth.test");
  });

  it("is an ordinary sign-in for any other address, demo or not", async () => {
    const { set, asked } = await signIn(
      "http://demo.test",
      "someone@example.org",
    );
    expect(set).toEqual([]);
    expect(asked.length).toBe(1);
  });
});

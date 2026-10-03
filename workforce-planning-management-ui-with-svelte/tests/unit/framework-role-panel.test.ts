// "My role and skills": the panel loads the selected role's skills and saves
// only what the person changed.

import { fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { afterEach, describe, expect, it, vi } from "vitest";

import FrameworkRolePanel from "../../src/lib/components/FrameworkRolePanel.svelte";

const json = (body: unknown) =>
  new Response(JSON.stringify(body), { status: 200, headers: { "content-type": "application/json" } });

afterEach(() => vi.unstubAllGlobals());

describe("FrameworkRolePanel", () => {
  it("saves only the changed skills, at the person's own level", async () => {
    const calls: Array<{ url: string; method: string; body: unknown }> = [];
    vi.stubGlobal(
      "fetch",
      vi.fn(async (url: string | URL, init?: RequestInit) => {
        const u = String(url);
        const method = init?.method ?? "GET";
        calls.push({ url: u, method, body: init?.body ? JSON.parse(String(init.body)) : undefined });
        if (u.endsWith("/framework-roles")) {
          return json([
            { framework: "esco", role_label: "software developer", role_profile_pid: null,
              occupation_uri: "http://x/o", selected_on: "2026-10-03" },
          ]);
        }
        if (u.endsWith("/framework-roles/esco/skills") && method === "GET") {
          return json({
            framework: "esco",
            role: { framework: "esco", role_label: "software developer", role_profile_pid: null,
                    occupation_uri: "http://x/o", selected_on: "2026-10-03" },
            skills: [
              { ref: "http://x/s1", label: "computer programming", relation: "essential", declared: 4 },
              { ref: "http://x/s2", label: "debug software", relation: "essential", declared: null },
            ],
          });
        }
        if (u.endsWith("/framework-roles/esco/skills") && method === "PUT") {
          return json({ declared: 1, removed: 0, skills_created: 1, skills_linked: 0 });
        }
        return json([]);
      }),
    );
    render(FrameworkRolePanel, { workerPid: "w1", framework: "esco", title: "ESCO" });

    // The current role and each skill's declared state are shown.
    expect(await screen.findByText("software developer")).toBeTruthy();
    const programming = (await screen.findByLabelText("I have computer programming")) as HTMLInputElement;
    const debug = (await screen.findByLabelText("I have debug software")) as HTMLInputElement;
    expect(programming.checked).toBe(true);
    expect(debug.checked).toBe(false);

    // Nothing changed yet, so there is nothing to save.
    const save = screen.getByTestId("save-esco") as HTMLButtonElement;
    expect(save.disabled).toBe(true);

    // Tick one skill: exactly one change, at the default level.
    await fireEvent.click(debug);
    await waitFor(() => expect(save.disabled).toBe(false));
    expect(save.textContent).toContain("1 change");
    await fireEvent.click(save);
    await waitFor(() =>
      expect(calls.some((c) => c.method === "PUT" && c.url.endsWith("/skills"))).toBe(true),
    );
    const put = calls.find((c) => c.method === "PUT" && c.url.endsWith("/skills"));
    expect(put?.body).toEqual({ selections: [{ ref: "http://x/s2", proficiency: 3 }] });
  });

  it("offers a role choice when none is selected", async () => {
    vi.stubGlobal("fetch", vi.fn(async () => json([])));
    render(FrameworkRolePanel, { workerPid: "w1", framework: "esco", title: "ESCO" });
    expect(await screen.findByText(/Choose the role you hold now/)).toBeTruthy();
    expect(screen.getByTestId("esco-role-query")).toBeTruthy();
    expect(screen.queryByTestId("save-esco")).toBeNull();
  });
});

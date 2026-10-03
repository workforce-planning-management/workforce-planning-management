// Career history, aspirations, and the HR "on their behalf" notice.

import { fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { afterEach, describe, expect, it, vi } from "vitest";

import Aspirations from "../../src/lib/components/Aspirations.svelte";
import CareerHistory from "../../src/lib/components/CareerHistory.svelte";
import FrameworkRolePanel from "../../src/lib/components/FrameworkRolePanel.svelte";

const json = (body: unknown) =>
  new Response(JSON.stringify(body), { status: 200, headers: { "content-type": "application/json" } });

afterEach(() => vi.unstubAllGlobals());

describe("FrameworkRolePanel acting for someone", () => {
  it("says the change is made on their behalf", async () => {
    vi.stubGlobal("fetch", vi.fn(async () => json([])));
    render(FrameworkRolePanel, { workerPid: "w9", framework: "esco", title: "ESCO", actingFor: "Alex Example" });
    expect(await screen.findByTestId("acting-for")).toBeTruthy();
    expect(screen.getByTestId("acting-for").textContent).toContain("Alex Example");
    expect(screen.getByTestId("acting-for").textContent).toContain("on their behalf");
  });

  it("shows no such notice for the person themselves", async () => {
    vi.stubGlobal("fetch", vi.fn(async () => json([])));
    render(FrameworkRolePanel, { workerPid: "w1", framework: "esco", title: "ESCO" });
    await screen.findByText(/Choose the role you hold now/);
    expect(screen.queryByTestId("acting-for")).toBeNull();
  });
});

describe("CareerHistory", () => {
  it("lists roles with their start and stop, marking the current one and who made it", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn(async (url: string | URL) => {
        const u = String(url);
        if (u.includes("role-history")) {
          return json([
            { pid: "r2", framework: "uk-gdad-pcf", role_label: "Tester", started_at: "2026-10-01T00:00:00Z",
              ended_at: null, current: true, recorded_by: "hr-1", on_behalf: true, role_profile_pid: "p", occupation_uri: null, selected_on: "2026-10-01" },
            { pid: "r1", framework: "uk-gdad-pcf", role_label: "Junior tester", started_at: "2024-01-01T00:00:00Z",
              ended_at: "2026-10-01T00:00:00Z", current: false, recorded_by: null, on_behalf: false, role_profile_pid: "q", occupation_uri: null, selected_on: "2024-01-01" },
          ]);
        }
        return json([]);
      }),
    );
    render(CareerHistory, { workerPid: "w1" });
    const table = await screen.findByTestId("role-history");
    await waitFor(() => expect(table.textContent).toContain("Junior tester"));
    expect(table.textContent).toContain("2024-01-01");
    expect(table.textContent).toContain("current");
    expect(table.textContent).toContain("on their behalf");
  });
});

describe("Aspirations", () => {
  it("tells a viewer other than the person that private items are hidden", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn(async (url: string | URL) =>
        String(url).includes("aspirations")
          ? json({ viewer_is_the_person: false, aspirations: [], private_hidden: 2 })
          : json([]),
      ),
    );
    render(Aspirations, { workerPid: "w1" });
    expect(await screen.findByText(/only what this person has chosen to share/)).toBeTruthy();
    expect(screen.getByText(/2 private item\(s\) are not shown/)).toBeTruthy();
  });

  it("adds a skill goal that is private unless shared", async () => {
    const posts: unknown[] = [];
    vi.stubGlobal(
      "fetch",
      vi.fn(async (url: string | URL, init?: RequestInit) => {
        const u = String(url);
        if (init?.method === "POST") {
          posts.push(JSON.parse(String(init.body)));
          return json({ pid: "a1" });
        }
        if (u.includes("aspirations")) return json({ viewer_is_the_person: true, aspirations: [], private_hidden: 0 });
        if (u.endsWith("/skills")) {
          return json([{ pid: "s1", name: "Rust", category: "technical", external_refs: [] }]);
        }
        return json([]);
      }),
    );
    render(Aspirations, { workerPid: "w1" });
    const skill = (await screen.findByLabelText("Skill")) as HTMLSelectElement;
    await waitFor(() => expect(skill.options.length).toBeGreaterThan(1));
    await fireEvent.change(skill, { target: { value: "s1" } });
    await fireEvent.click(screen.getByTestId("aspiration-add"));
    await waitFor(() => expect(posts.length).toBe(1));
    expect(posts[0]).toMatchObject({ kind: "skill", skill_pid: "s1", target_level: 4, horizon: "within_1y", shared: false });
  });
});

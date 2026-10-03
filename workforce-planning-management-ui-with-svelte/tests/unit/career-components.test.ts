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
  it("shows a viewer other than the person only what is shared, without a hidden count", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn(async (url: string | URL) =>
        String(url).includes("aspirations")
          ? json({ viewer: "manager", viewer_is_the_person: false, aspirations: [] })
          : json([]),
      ),
    );
    render(Aspirations, { workerPid: "w1" });
    expect(await screen.findByText(/only what this person has chosen to share/)).toBeTruthy();
    expect(screen.queryByText(/not shown/)).toBeNull();
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
        if (u.includes("aspirations")) return json({ viewer: "person", viewer_is_the_person: true, aspirations: [] });
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
    expect(posts[0]).toMatchObject({ kind: "skill", skill_pid: "s1", target_level: 4, horizon: "within_1y", visibility: "private" });
  });
});

describe("TeamAspirations and GroupsPanel", () => {
  it("lists direct and indirect reports with only what they shared", async () => {
    vi.stubGlobal("fetch", vi.fn(async () => json({
      manager: { pid: "m", display_name: "M" }, viewer: "manager",
      team: [
        { worker_pid: "a", display_name: "Ann", job_title: "", department: "", depth: 1, direct_report: true,
          aspirations: [{ pid: "x", kind: "skill", skill: "Rust", target_level: 4, horizon: "within_1y", status: "idea", note: null, visibility: "manager" }] },
        { worker_pid: "b", display_name: "Bo", job_title: "", department: "", depth: 2, direct_report: false, aspirations: [] },
      ],
    })));
    const { default: TeamAspirations } = await import("../../src/lib/components/TeamAspirations.svelte");
    render(TeamAspirations, { workerPid: "m" });
    const panel = await screen.findByTestId("team-aspirations");
    expect(panel.textContent).toContain("1 direct report(s), 1 indirect");
    expect(panel.textContent).toContain("Rust → level 4");
    expect(panel.textContent).toContain("indirect report");
    expect(panel.textContent).toContain("nothing shared");
  });

  it("shows several groups at once", async () => {
    vi.stubGlobal("fetch", vi.fn(async (url: string | URL) =>
      String(url).endsWith("/workers/w")
        ? json({ pid: "w", organization_ref: "organization:acme" })
        : String(url).includes("/groups?organization_ref=")
        ? json([])
        : String(url).includes("/workers/")
        ? json({ worker_pid: "w", groups: [
            { group_pid: "g1", name: "Rust guild", kind: "practice", role: "lead", joined_at: "2026-01-01T00:00:00Z", left_at: null, current: true, on_behalf: false },
            { group_pid: "g2", name: "Chess", kind: "interest", role: "member", joined_at: "2026-02-01T00:00:00Z", left_at: null, current: true, on_behalf: false },
          ] })
        : json([])));
    const { default: GroupsPanel } = await import("../../src/lib/components/GroupsPanel.svelte");
    render(GroupsPanel, { workerPid: "w" });
    const list = await screen.findByTestId("my-groups");
    await waitFor(() => expect(list.textContent).toContain("Rust guild"));
    expect(list.textContent).toContain("Chess");
    expect(list.textContent).toContain("community of practice");
    expect(list.textContent).toContain("community of interest");
  });
});

describe("DottedLinePanel", () => {
  it("lists dotted-line managers and reports separately", async () => {
    vi.stubGlobal("fetch", vi.fn(async (url: string | URL) =>
      String(url).includes("dotted-line")
        ? json({
            worker: { pid: "w", display_name: "W", job_title: "", department: "" },
            dotted_line_managers: [{ pid: "m", display_name: "Mia", job_title: "", department: "", note: "project X", started_at: "2026-01-01T00:00:00Z", ended_at: null, current: true, on_behalf: true }],
            dotted_line_reports: [{ pid: "r", display_name: "Rae", job_title: "", department: "", note: null, started_at: "2026-01-01T00:00:00Z", ended_at: null, current: true, on_behalf: false }],
          })
        : json([{ pid: "w", display_name: "W" }, { pid: "m", display_name: "Mia" }, { pid: "z", display_name: "Zed" }])));
    const { default: DottedLinePanel } = await import("../../src/lib/components/DottedLinePanel.svelte");
    render(DottedLinePanel, { workerPid: "w" });
    const managers = await screen.findByTestId("dotted-managers");
    await waitFor(() => expect(managers.textContent).toContain("Mia"));
    expect(managers.textContent).toContain("project X");
    expect(managers.textContent).toContain("on their behalf");
    expect(screen.getByTestId("dotted-reports").textContent).toContain("Rae");
    const options = [...(screen.getByLabelText(/Add a dotted-line manager/) as HTMLSelectElement).options].map((o) => o.text);
    expect(options).toContain("Zed");
    expect(options).not.toContain("W");
    expect(options).not.toContain("Mia");
  });
});

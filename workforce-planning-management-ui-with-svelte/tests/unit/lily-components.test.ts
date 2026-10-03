// The Lily kanban board and Gantt chart wrappers render their data.

import { render, screen } from "@testing-library/svelte";
import { describe, expect, it } from "vitest";

import LilyGantt from "../../src/lib/components/LilyGantt.svelte";
import LilyKanban from "../../src/lib/components/LilyKanban.svelte";

describe("LilyKanban", () => {
  it("shows each card under its column, with a count", () => {
    render(LilyKanban, {
      label: "Initiatives",
      columns: [
        { id: "draft", title: "draft" },
        { id: "active", title: "active" },
      ],
      cards: [
        { id: "1", columnId: "draft", title: "Code assistant" },
        { id: "2", columnId: "active", title: "Auto-triage" },
      ],
      onMove: () => {},
    });
    expect(screen.getByText("Code assistant")).toBeTruthy();
    expect(screen.getByText("Auto-triage")).toBeTruthy();
    expect(screen.getByText("draft")).toBeTruthy();
    // The move menu is the keyboard-accessible alternative to dragging.
    expect(screen.getByRole("button", { name: "Move Code assistant" })).toBeTruthy();
  });
});

describe("LilyGantt", () => {
  it("renders task rows and a milestone, with no editing controls", () => {
    render(LilyGantt, {
      label: "Plan timeline",
      range: { start: "2026-10-01", end: "2027-03-31" },
      today: "2026-10-03",
      tasks: [
        { id: "p", label: "FY27 plan", start: "2026-10-01", end: "2027-03-31" },
        { id: "m", label: "engineering · 5", start: "2027-01-15", end: "2027-01-15", parentId: "p" },
      ],
    });
    expect(screen.getByText("FY27 plan")).toBeTruthy();
    expect(screen.getByText("engineering · 5")).toBeTruthy();
    expect(screen.queryByText("Save")).toBeNull();
  });
});

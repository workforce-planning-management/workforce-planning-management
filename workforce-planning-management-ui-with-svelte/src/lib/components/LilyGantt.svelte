<!--
  A read-only Gantt timeline: `@lilydesignsystem/svelte-gantt-chart` with
  minimal styling (the package ships no CSS). Tasks with equal start and end
  render as milestones. No `onTaskChange` and no date-picker labels are
  passed, so the chart offers no editing — it shows, it does not reschedule.
-->
<script lang="ts">
  import GanttChart from "@lilydesignsystem/svelte-gantt-chart";
  import type { GanttTask, GanttTimeUnit } from "@lilydesignsystem/svelte-gantt-chart";

  let {
    label,
    caption,
    range,
    tasks,
    today,
    timeUnit = "month",
  }: {
    label: string;
    caption?: string;
    range: { start: string; end: string };
    tasks: GanttTask[];
    today?: string;
    timeUnit?: GanttTimeUnit;
  } = $props();
</script>

<div class="lily-gantt">
  <GanttChart
    {label}
    {caption}
    {range}
    {tasks}
    {today}
    {timeUnit}
    labels={{
      columnLabel: (start) => start.slice(0, timeUnit === "month" ? 7 : 10),
      collapseButton: (task, collapsed) =>
        collapsed ? `Expand ${task.label}` : `Collapse ${task.label}`,
      dependencySummary: (predecessors) => `Depends on: ${predecessors.join(", ")}`,
    }}
  />
</div>

<style>
  .lily-gantt {
    overflow-x: auto;
    margin-block: 0.5rem 1rem;
  }
  .lily-gantt :global(.gantt-chart) {
    border-collapse: collapse;
    width: 100%;
    font-size: 0.85rem;
  }
  .lily-gantt :global([role="gridcell"]),
  .lily-gantt :global([role="columnheader"]),
  .lily-gantt :global([role="rowheader"]) {
    padding: 0.2rem 0.4rem;
    border: 1px solid var(--line, #e3e3e3);
    white-space: nowrap;
  }
  .lily-gantt :global([data-in-range]) {
    background: color-mix(in srgb, currentColor 22%, transparent);
  }
  .lily-gantt :global([data-milestone]) {
    font-weight: 700;
  }
  .lily-gantt :global([data-today]) {
    border-inline: 2px solid #b45309;
  }
</style>

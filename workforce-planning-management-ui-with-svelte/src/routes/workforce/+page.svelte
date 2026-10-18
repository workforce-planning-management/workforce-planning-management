<script lang="ts">
  import { decideLeave, ergonomicIssues, listWorkers, listLeaveRequests, listShifts, workingTime } from "#lib/api/wpm.js";
  import { t, tf, tv } from "#lib/i18n.svelte.js";
  import CalendarPanel from "#lib/components/CalendarPanel.svelte";
  import DataTable from "#lib/components/DataTable.svelte";
  import type {
    DataGridColumn,
    DataGridRow,
  } from "@lilydesignsystem/svelte-data-grid";
  import type { Worker, LeaveRequest } from "#lib/api/types.js";

  type ShiftRow = Awaited<ReturnType<typeof listShifts>>[number];
  type WorkingTimeSignals = Awaited<ReturnType<typeof workingTime>>;

  let pending = $state<(LeaveRequest & { worker?: Worker })[] | null>(null);
  let shifts = $state<ShiftRow[]>([]);
  let signals = $state<WorkingTimeSignals | null>(null);
  let ergIssues = $state<Awaited<ReturnType<typeof ergonomicIssues>> | null>(null);
  let error = $state<string | null>(null);
  let actionError = $state<string | null>(null);

  async function load() {
    try {
      const workers = await listWorkers();
      const byPid = new Map(workers.map((e) => [e.pid, e]));
      const requestLists = await Promise.all(
        workers.map((e) => listLeaveRequests(e.pid)),
      );
      pending = requestLists
        .flat()
        .filter((r) => r.status === "requested")
        .map((r) => ({ ...r, worker: byPid.get(r.worker_pid) }));
      shifts = await listShifts();
      signals = await workingTime();
      ergIssues = await ergonomicIssues();
    } catch (cause) {
      error = cause instanceof Error ? cause.message : String(cause);
    }
  }

  $effect(() => {
    void load();
  });

  async function decide(request: LeaveRequest, decision: "approve" | "reject") {
    actionError = null;
    try {
      await decideLeave(request.pid, decision);
      await load();
    } catch (cause) {
      actionError = cause instanceof Error ? cause.message : String(cause);
    }
  }

  const leaveEvents = $derived(
    (pending ?? []).map((r) => ({
      id: r.pid,
      start: r.start_on,
      end: r.end_on,
      title: `${r.worker?.display_name ?? r.worker_pid.slice(0, 8)} · ${tv(r.kind)}`,
    })),
  );
  const shiftEvents = $derived(
    shifts.map((s) => ({
      id: s.shift.pid,
      date: s.shift.starts_at.slice(0, 10),
      title: `${s.shift.department} ${new Date(s.shift.starts_at).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })}`,
      values: { filled: `${s.assignments.length} / ${s.shift.required_headcount}` },
    })),
  );

  const pendingColumns = $derived<DataGridColumn[]>([
    {
      id: "worker",
      header: t("columns.person"),
      sortable: true,
      accessor: (r) => {
        const p = r as unknown as { worker?: { display_name: string }; worker_pid: string };
        return p.worker?.display_name ?? p.worker_pid.slice(0, 8);
      },
    },
    { id: "kind", header: t("columns.kind"), sortable: true, accessor: (r) => tv(String(r.kind)) },
    {
      id: "start_on",
      header: t("columns.dates"),
      sortable: true,
      format: (_v, r) => `${r.start_on} → ${r.end_on} (${r.days} ${t("common.days")})`,
    },
    { id: "actions", header: t("common.actions"), cell: decideCell },
  ]);
  const rotaColumns = $derived<DataGridColumn[]>([
    { id: "department", header: t("common.department"), sortable: true },
    {
      id: "starts_at",
      header: t("columns.shift"),
      sortable: true,
      format: (_v, r) =>
        `${new Date(String(r.starts_at)).toLocaleString()} → ${new Date(String(r.ends_at)).toLocaleTimeString()}`,
    },
    {
      id: "assigned",
      header: t("columns.filled"),
      sortable: true,
      format: (v, r) => `${v} / ${r.required_headcount}`,
    },
  ]);
  const ergColumns = $derived<DataGridColumn[]>([
    {
      id: "display_name",
      header: t("columns.person"),
      sortable: true,
      cell: personDept,
    },
    { id: "workstation", header: t("erg.workstation"), sortable: true },
    { id: "item", header: t("erg.issue"), sortable: true, cell: issueCell },
  ]);
  const flagColumns = $derived<DataGridColumn[]>([
    { id: "display_name", header: t("columns.person"), sortable: true, cell: personDept },
    {
      id: "average",
      header: t("columns.weeklyAverage"),
      cell: averageCell,
    },
    { id: "rest", header: t("columns.restBreaks"), cell: restCell },
  ]);
</script>

{#snippet decideCell({ row }: { row: DataGridRow })}
  {@const request = row as unknown as LeaveRequest & { worker?: Worker }}
  <button onclick={() => void decide(request, "approve")}>{t("wf.approve")}</button>
  <button onclick={() => void decide(request, "reject")}>{t("wf.reject")}</button>
{/snippet}

{#snippet personDept({ row }: { row: DataGridRow })}
  {row.display_name} <span class="muted">({row.department})</span>
{/snippet}

{#snippet issueCell({ row }: { row: DataGridRow })}
  {row.item}{#if row.note} — <span class="muted">{row.note}</span>{/if}
{/snippet}

{#snippet averageCell({ row }: { row: DataGridRow })}
  {@const flag = row as unknown as NonNullable<typeof signals>["flagged"][number]}
  {#if flag.over_48h}
    <span class="chip">{t("wf.over48")}</span>
    {tf("pages.workforce.h_wk", { value_minutes_per_week: Math.round((flag.average_weekly.value_minutes_per_week ?? 0) / 60) })}
  {/if}
{/snippet}

{#snippet restCell({ row }: { row: DataGridRow })}
  {@const flag = row as unknown as NonNullable<typeof signals>["flagged"][number]}
  {#each flag.rest_breaches as breach (breach.prev_end)}
    <span class="chip">{t("wf.restBreach")}: {Math.floor(breach.gap_minutes / 60)}h{breach.gap_minutes % 60 ? ` ${breach.gap_minutes % 60}m` : ""}</span>
  {/each}
{/snippet}

<h1>{t("nav.workforce")}</h1>

{#if error}
  <p class="error" data-testid="error">{t("common.error")}: {error}</p>
{:else if pending === null}
  <p>{t("common.loading")}</p>
{:else}
  <h2>{t("wf.leaveRequests")}</h2>
  {#if actionError}
    <p class="error" data-testid="action-error">{actionError}</p>
  {/if}
  <DataTable
    testid="pending-leave"
    label={t("wf.leaveRequests")}
    columns={pendingColumns}
    rows={pending as unknown as DataGridRow[]}
    rowId={(row) => String(row.pid)}
  />
  <CalendarPanel testid="leave-calendar" label={t("calendar.leave")} events={leaveEvents} />

  <h2>{t("wf.rota")}</h2>
  <DataTable
    testid="rota"
    label={t("wf.rota")}
    columns={rotaColumns}
    rows={shifts.map((s) => ({ ...s.shift, assigned: s.assignments.length })) as unknown as DataGridRow[]}
    rowId={(row) => String(row.pid)}
  />
  <CalendarPanel testid="shift-calendar" label={t("calendar.shifts")} events={shiftEvents}
    fields={[{ id: "filled", header: t("columns.filled") }]} />

  {#if ergIssues && ergIssues.issues.length > 0}
    <h2>{t("erg.issues")}</h2>
    <DataTable
      testid="ergonomic-issues"
      label={t("erg.issues")}
      columns={ergColumns}
      rows={ergIssues.issues.map((i, n) => ({ ...i, n })) as unknown as DataGridRow[]}
      rowId={(row) => String(row.n)}
    />
  {/if}

  {#if signals}
    <h2>{t("wf.workingTime")}</h2>
    <p class="muted">{signals.derivation}</p>
    {#if signals.flagged.length === 0}
      <p data-testid="working-time-clear">{t("wf.allClear")}</p>
    {:else}
      <DataTable
        testid="working-time"
        label={t("wf.workingTime")}
        columns={flagColumns}
        rows={signals.flagged as unknown as DataGridRow[]}
        rowId={(row) => String(row.worker_pid)}
      />
    {/if}
  {/if}
{/if}

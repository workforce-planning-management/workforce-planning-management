<script lang="ts">
  import { decideLeave, ergonomicIssues, listWorkers, listLeaveRequests, listShifts, workingTime } from "#lib/api/wpm.js";
  import { t, tf, tv } from "#lib/i18n.svelte.js";
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
</script>

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
  <table data-testid="pending-leave">
    <tbody>
      {#each pending as request (request.pid)}
        <tr>
          <td>{request.worker?.display_name ?? request.worker_pid.slice(0, 8)}</td>
          <td>{tv(request.kind)}</td>
          <td>{request.start_on} → {request.end_on} ({request.days} {t("common.days")})</td>
          <td>
            <button onclick={() => void decide(request, "approve")}>{t("wf.approve")}</button>
            <button onclick={() => void decide(request, "reject")}>{t("wf.reject")}</button>
          </td>
        </tr>
      {/each}
    </tbody>
  </table>

  <h2>{t("wf.rota")}</h2>
  <table data-testid="rota">
    <tbody>
      {#each shifts as row (row.shift.pid)}
        <tr>
          <td>{row.shift.department}</td>
          <td>{new Date(row.shift.starts_at).toLocaleString()} → {new Date(row.shift.ends_at).toLocaleTimeString()}</td>
          <td>{row.assignments.length} / {row.shift.required_headcount}</td>
        </tr>
      {/each}
    </tbody>
  </table>

  {#if ergIssues && ergIssues.issues.length > 0}
    <h2>{t("erg.issues")}</h2>
    <table data-testid="ergonomic-issues">
      <tbody>
        {#each ergIssues.issues as issue, index (index)}
          <tr>
            <td>{issue.display_name} <span class="muted">({issue.department})</span></td>
            <td>{issue.workstation}</td>
            <td>{issue.item}{#if issue.note} — <span class="muted">{issue.note}</span>{/if}</td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}

  {#if signals}
    <h2>{t("wf.workingTime")}</h2>
    <p class="muted">{signals.derivation}</p>
    {#if signals.flagged.length === 0}
      <p data-testid="working-time-clear">{t("wf.allClear")}</p>
    {:else}
      <table data-testid="working-time">
        <tbody>
          {#each signals.flagged as flag (flag.worker_pid)}
            <tr>
              <td>{flag.display_name} <span class="muted">({flag.department})</span></td>
              <td>
                {#if flag.over_48h}
                  <span class="chip">{t("wf.over48")}</span>
                  {tf("pages.workforce.h_wk", { value_minutes_per_week: Math.round((flag.average_weekly.value_minutes_per_week ?? 0) / 60) })}
                {/if}
              </td>
              <td>
                {#each flag.rest_breaches as breach (breach.prev_end)}
                  <span class="chip">{t("wf.restBreach")}: {Math.floor(breach.gap_minutes / 60)}h{breach.gap_minutes % 60 ? ` ${breach.gap_minutes % 60}m` : ""}</span>
                {/each}
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    {/if}
  {/if}
{/if}

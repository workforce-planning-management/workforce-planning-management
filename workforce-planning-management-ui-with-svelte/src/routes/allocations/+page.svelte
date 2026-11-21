<!--
  Allocations (`/allocations`, WPM-R275, WPM-R276): the people of an organization, in name order, with what each is planned
  to work on and each month's standing (over-allocated, a clash with leave, free). For HR and resource managers; anyone else
  is told it is not theirs to see. It never ranks people and shows no score; a month the organization does not track is "—".
  A resource manager confirms, ends or removes an allocation here; the service refuses one for themselves.
-->
<script lang="ts">
  import { formatCentiFte, formatMonth } from "#lib/locale-format.js";
  import { guardedLoad } from "#lib/loader.svelte.js";
  import { errorText } from "#lib/write.js";
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import {
    createAllocation,
    deleteAllocation,
    getWorkerAssignments,
    listAllocations,
    setAllocationStatus,
    type AssignmentSummary,
  } from "#lib/api/wpm.js";
  import { ApiError } from "#lib/api/client.js";
  import type { AllocationMonth, AllocationSignal, WorkerAllocations } from "#lib/api/types.js";
  import { page } from "$app/state";
  import { t, tp, i18n } from "#lib/i18n.svelte.js";

  const organizations = $derived((page.data.scope ?? []) as string[]);
  let organization = $state<string>("");
  $effect(() => {
    if (!organization && organizations.length > 0) organization = organizations[0] ?? "";
  });

  let people = $state<WorkerAllocations[]>([]);
  let allowed = $state(true);
  let error = $state<string | null>(null);
  const months = $derived(people[0]?.months.map((m) => m.month) ?? []);

  const load = guardedLoad(
    async () => (organization ? (await listAllocations(organization)).items : []),
    (found) => {
      people = found;
      allowed = true;
      error = null;
    },
    (cause) => {
      if (cause instanceof ApiError && (cause.status === 403 || cause.status === 401)) {
        allowed = false;
      } else {
        error = errorText(cause);
      }
    },
  );

  $effect(() => {
    void organization;
    void load();
  });

  async function run(action: () => Promise<unknown>) {
    error = null;
    try {
      await action();
      await load();
    } catch (cause) {
      error = errorText(cause);
    }
  }

  const fte = (centi: number) => formatCentiFte(centi, i18n.locale);

  // The form to add an allocation: a person, one of their current jobs, a source, months, a share and a status.
  let who = $state("");
  let jobs = $state<AssignmentSummary[]>([]);
  let job = $state("");
  let source = $state("");
  let fromMonth = $state("");
  let toMonth = $state("");
  let share = $state(50);
  let confirmNow = $state(false);
  $effect(() => {
    const person = who;
    jobs = [];
    job = "";
    if (!person) return;
    let stale = false;
    getWorkerAssignments(person)
      .then((found) => {
        if (stale) return;
        jobs = found.assignments.filter((a) => a.current);
        job = jobs[0]?.pid ?? "";
      })
      .catch((cause) => {
        if (!stale) error = errorText(cause);
      });
    return () => {
      stale = true;
    };
  });

  function add(event: SubmitEvent) {
    event.preventDefault();
    if (!who || !job || !source.trim() || !fromMonth || !toMonth) return;
    const person = who;
    void run(() =>
      createAllocation(person, {
        assignment_pid: job,
        source_ref: source.trim(),
        from_month: `${fromMonth}-01`,
        to_month: `${toMonth}-01`,
        fte_centi: Math.round(share),
        status: confirmNow ? "confirmed" : "tentative",
      }),
    );
  }

  function mark(signal: AllocationSignal): string {
    switch (signal.kind) {
      case "over_allocated":
        return `▲ ${tp("allocations.signal.overAllocated", { fte: fte(signal.by_centi) })}`;
      case "leave_clash":
        return `▲ ${tp("allocations.signal.leaveClash", { fte: fte(signal.short_centi) })}`;
      case "available":
        return `● ${tp("allocations.signal.available", { fte: fte(signal.free_centi) })}`;
      case "assignment_over_allocated":
        return `▲ ${tp("allocations.signal.assignmentOver", { fte: fte(signal.by_centi) })}`;
    }
  }

  function cell(m: AllocationMonth | undefined): string {
    if (!m || m.standing === "unknown") return "—";
    return (
      tp("allocations.allocated", { allocated: fte(m.allocated_centi), available: fte(m.available_centi) }) ?? "—"
    );
  }
</script>

<h1>{t("nav.allocations")}</h1>
<p class="muted">{t("pages.allocations.hint")}</p>

{#if organizations.length > 1}
  <label>
    {t("pages.allocations.organization")}
    <select bind:value={organization} data-testid="allocations-organization">
      {#each organizations as o (o)}<option value={o}>{o}</option>{/each}
    </select>
  </label>
{/if}

{#if error}<ErrorNote testid="allocations-page-error" message={error} />{/if}

{#if allowed && people.length > 0}
  <form class="panel al-form" data-testid="allocation-form" onsubmit={add}>
    <h2>{t("pages.allocations.add")}</h2>
    <label>
      {t("pages.allocations.person")}
      <select bind:value={who} data-testid="allocation-person">
        <option value="">—</option>
        {#each people as p (p.worker_pid)}<option value={p.worker_pid}>{p.display_name}</option>{/each}
      </select>
    </label>
    <label>
      {t("pages.allocations.job")}
      <select bind:value={job} data-testid="allocation-job" disabled={jobs.length === 0}>
        {#each jobs as a (a.pid)}<option value={a.pid}>{a.job_title} ({fte(a.fte_percent)})</option>{/each}
      </select>
    </label>
    <label>
      {t("allocations.source")}
      <input bind:value={source} data-testid="allocation-source" />
    </label>
    <label>
      {t("pages.allocations.from")}
      <input type="month" bind:value={fromMonth} data-testid="allocation-from" />
    </label>
    <label>
      {t("pages.allocations.to")}
      <input type="month" bind:value={toMonth} data-testid="allocation-to" />
    </label>
    <label>
      {t("pages.allocations.sharePercent")}
      <input type="number" min="1" max="100" bind:value={share} data-testid="allocation-share" />
    </label>
    <label>
      <input type="checkbox" bind:checked={confirmNow} data-testid="allocation-confirmed" />
      {t("pages.allocations.confirmNow")}
    </label>
    <button type="submit" data-testid="allocation-add">{t("pages.allocations.add")}</button>
  </form>
{/if}

{#if !allowed}
  <p data-testid="allocations-not-allowed">● {t("pages.allocations.notAllowed")}</p>
{:else if people.length === 0}
  <p data-testid="allocations-empty">{t("pages.allocations.empty")}</p>
{:else}
  <div class="al-scroll">
    <table data-testid="allocations-grid">
      <thead>
        <tr>
          <th scope="col">{t("pages.allocations.person")}</th>
          {#each months as m (m)}<th scope="col">{formatMonth(m)}</th>{/each}
        </tr>
      </thead>
      <tbody>
        {#each people as person (person.worker_pid)}
          <tr data-testid="allocations-row">
            <th scope="row">{person.display_name}</th>
            {#each months as month (month)}
              {@const m = person.months.find((x) => x.month === month)}
              <td>
                {cell(m)}
                {#if m && m.standing === "known"}
                  {#each m.signals as s, i (i)}<span class="al-signal">{mark(s)}</span>{/each}
                {/if}
              </td>
            {/each}
          </tr>
          {#each person.allocations.filter((a) => a.status !== "ended") as a (a.pid)}
            <tr class="al-detail">
              <td colspan={months.length + 1}><div class="al-detail-body">
                {a.source_ref} · {tp("allocations.monthRange", {
                  from: formatMonth(a.from_month),
                  to: formatMonth(a.to_month),
                })} · {fte(a.fte_centi)} · {t(`allocations.state.${a.status}`)}
                <span class="al-actions">
                  {#if a.status === "tentative"}
                    <button type="button" data-testid="allocation-confirm" onclick={() => run(() => setAllocationStatus(a.pid, "confirmed"))}>
                      {t("pages.allocations.confirm")}
                    </button>
                  {/if}
                  <button type="button" data-testid="allocation-end" onclick={() => run(() => setAllocationStatus(a.pid, "ended"))}>
                    {t("pages.allocations.end")}
                  </button>
                  <button type="button" data-testid="allocation-remove" onclick={() => run(() => deleteAllocation(a.pid))}>
                    {t("pages.allocations.remove")}
                  </button>
                </span>
              </div></td>
            </tr>
          {/each}
        {/each}
      </tbody>
    </table>
  </div>
{/if}

<style>
  .al-scroll {
    overflow-x: auto;
  }
  .al-scroll table {
    min-inline-size: 40rem;
  }
  .al-signal {
    display: block;
  }
  .al-actions {
    margin-inline-start: 0.5em;
  }
  .al-form label {
    display: block;
    margin-block: 0.4em;
  }
  /* A person's name keeps its own case (the global header style upper-cases). */
  .al-scroll th[scope="row"] {
    text-transform: none;
  }
  /* The detail line stays in view and wraps while the months scroll beside it. */
  .al-detail-body {
    position: sticky;
    inset-inline-start: 0;
    max-inline-size: calc(100vw - 4rem);
    white-space: normal;
  }
  .al-detail td {
    font-size: 0.9em;
  }
</style>

<!--
  A person's allocations (WPM-R275): what they are planned to work on, month by month, and what each month says
  (over-allocated, a clash with leave, free). It is a plan, never a measure of how much someone did, and it has no score.
  The service shows it to the person, their line managers, HR and resource managers; anyone else gets a 403 and the panel
  renders nothing. Where the organization does not track allocation, a month is "—", never zero. The kind of leave is never shown.
-->
<script lang="ts">
  import { formatCentiFte, formatDate, formatMonth } from "#lib/locale-format.js";
  import { guardedLoad } from "#lib/loader.svelte.js";
  import { errorText } from "#lib/write.js";
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import { getWorkerAllocations } from "#lib/api/wpm.js";
  import { ApiError } from "#lib/api/client.js";
  import type { AllocationSignal, WorkerAllocations } from "#lib/api/types.js";
  import { t, tp, i18n } from "#lib/i18n.svelte.js";

  let { workerPid }: { workerPid: string } = $props();

  let held = $state<WorkerAllocations | null>(null);
  let allowed = $state(true);
  let error = $state<string | null>(null);

  const load = guardedLoad(
    () => getWorkerAllocations(workerPid),
    (found) => {
      held = found;
      allowed = true;
    },
    (cause) => {
      // Not someone who may read these allocations: not an error, just not theirs to see.
      if (cause instanceof ApiError && (cause.status === 403 || cause.status === 401)) {
        allowed = false;
      } else {
        error = errorText(cause);
      }
    },
  );

  $effect(() => {
    void workerPid;
    void load();
  });

  const fte = (centi: number) => formatCentiFte(centi, i18n.locale);

  function signalText(signal: AllocationSignal): string {
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
</script>

{#if allowed}
  <section class="panel" data-testid="allocations">
    <h2>{t("allocations.title")}</h2>
    <p class="muted">{t("allocations.hint")}</p>
    {#if error}<ErrorNote testid="allocations-error" message={error} />{/if}
    {#if held}
      {#if held.tracked === false}
        <p class="muted" data-testid="allocations-untracked">● {t("allocations.untracked")}</p>
      {/if}
      {#if held.allocations.length === 0}
        <p data-testid="allocations-none">{t("allocations.none")}</p>
      {:else}
        <div class="ma-scroll">
          <table data-testid="allocations-list">
            <thead>
              <tr>
                <th scope="col">{t("allocations.source")}</th>
                <th scope="col">{t("allocations.months")}</th>
                <th scope="col">{t("allocations.share")}</th>
                <th scope="col">{t("allocations.status")}</th>
              </tr>
            </thead>
            <tbody>
              {#each held.allocations as a (a.pid)}
                <tr>
                  <td>{a.source_ref}</td>
                  <td>{tp("allocations.monthRange", { from: formatMonth(a.from_month), to: formatMonth(a.to_month) })}</td>
                  <td>{fte(a.fte_centi)}</td>
                  <td>
                    {t(`allocations.state.${a.status}`)}
                    {#if a.runs_past_end}
                      <span class="ma-past-end" data-testid="allocations-past-end">
                        ▲ {tp("allocations.runsPastEnd", { date: formatDate(a.runs_past_end) })}
                      </span>
                    {/if}
                  </td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      {/if}
      <ul class="ma-months" data-testid="allocations-months">
        {#each held.months as m (m.month)}
          <li>
            <strong>{formatMonth(m.month)}</strong>:
            {#if m.standing === "unknown"}
              —
            {:else}
              {tp("allocations.allocated", {
                allocated: fte(m.allocated_centi),
                available: fte(m.available_centi),
              })}
              {#each m.signals as s, i (i)}
                <span class="ma-signal">{signalText(s)}</span>
              {/each}
            {/if}
          </li>
        {/each}
      </ul>
    {/if}
  </section>
{/if}

<style>
  .ma-scroll {
    overflow-x: auto;
  }
  /* Scroll a narrow screen rather than break a word across lines. */
  .ma-scroll table {
    min-inline-size: 34rem;
  }
  .ma-scroll th,
  .ma-scroll td {
    overflow-wrap: normal;
    word-break: normal;
  }
  .ma-past-end {
    display: block;
  }
  .ma-months {
    padding-inline-start: 1.2em;
  }
  .ma-signal {
    display: block;
  }
</style>

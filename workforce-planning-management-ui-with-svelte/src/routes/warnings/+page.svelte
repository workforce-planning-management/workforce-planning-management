<!--
  Early warnings (`/warnings`, WPM-R67 to R69): the conditions the service has seen for an
  organization (a pool over-committed in the next six months, a measure in breach, a cutover at risk),
  each dated by the first sweep that saw it, with days open and the time to report. A person
  acknowledges each warning; the time to report is not known while any is unacknowledged. "Look now"
  runs a sweep (an administrator), which opens, closes and notifies.
-->
<script lang="ts">
  import {
    acknowledgeEarlyWarning,
    listEarlyWarnings,
    sweepEarlyWarnings,
    type EarlyWarnings,
  } from "#lib/api/wpm.js";
  import OrgPicker from "#lib/components/OrgPicker.svelte";
  import WriteResult from "#lib/components/WriteResult.svelte";
  import { t, tf, tv } from "#lib/i18n.svelte.js";
  import { failure, type Failure } from "#lib/write.js";

  let org = $state("");
  let view = $state<EarlyWarnings | null>(null);
  let error = $state<string | null>(null);
  let failed = $state<Failure | null>(null);
  let swept = $state<string | null>(null);
  let reload = $state(0);

  $effect(() => {
    const chosen = org;
    void reload;
    if (!chosen) return;
    let current = true;
    void listEarlyWarnings(chosen)
      .then((v) => {
        if (!current) return;
        view = v;
        error = null;
      })
      .catch((cause) => {
        if (!current) return;
        view = null;
        error = cause instanceof Error ? cause.message : String(cause);
      });
    return () => {
      current = false;
    };
  });

  async function run(action: () => Promise<unknown>) {
    failed = null;
    try {
      await action();
      reload += 1;
    } catch (cause) {
      failed = failure(cause);
    }
  }

  async function sweep() {
    swept = null;
    await run(async () => {
      const r = await sweepEarlyWarnings(org);
      swept = tf("ew.swept", r);
    });
  }
</script>

<svelte:head><title>{t("titles.warnings")}</title></svelte:head>

<h1>{t("ew.title")}</h1>
<p class="muted">{t("ew.hint")}</p>

<div class="form"><OrgPicker bind:value={org} /></div>

{#if error}
  <p class="error" data-testid="error">{t("common.error")}: {error}</p>
{:else if !org}
  <p class="muted">{t("common.noOrganization")}</p>
{:else if view === null}
  <p>{t("common.loading")}</p>
{:else}
  <section class="panel">
    <p>
      <button type="button" data-testid="sweep" onclick={sweep}>{t("ew.sweep")}</button>
      {#if swept}<span class="muted" role="status" data-testid="swept">{swept}</span>{/if}
    </p>
    <WriteResult {failed} />
    <p data-testid="time-to-report">
      <strong>
        {view.time_to_report_days === null
          ? t("ew.timeToReportNone")
          : tf("ew.timeToReport", { days: view.time_to_report_days })}
      </strong>
    </p>
    {#if view.warnings.length === 0}
      <p class="muted" data-testid="no-warnings">{t("ew.none")}</p>
    {:else}
      <ul class="warnings">
        {#each view.warnings as w (w.pid)}
          <li data-testid="warning" data-open={w.cleared_on === null}>
            <span class="chip" class:danger={w.cleared_on === null}>{tv(w.kind)}</span>
            {#if w.subject_name}<strong>{w.subject_name}</strong>{/if}
            <span class="muted">
              · {tf("ew.firstSeen", { date: w.first_true_on })} · {tf("ew.daysOpen", { days: w.days_open })}
              {#if w.cleared_on}· {tf("ew.cleared", { date: w.cleared_on })}{/if}
              ·
              {w.acknowledged_on
                ? tf("ew.acknowledged", { date: w.acknowledged_on })
                : t("ew.notAcknowledged")}
            </span>
            {#if !w.acknowledged_on}
              <button
                type="button"
                data-testid="acknowledge"
                onclick={() => run(() => acknowledgeEarlyWarning(w.pid))}
              >
                {t("ew.acknowledge")}
              </button>
            {/if}
          </li>
        {/each}
      </ul>
    {/if}
    <details>
      <summary>{t("cap.basis")}</summary>
      <p class="muted">{t("ew.basisText")}</p>
      <p class="muted small">{t("common.fromService")}: {view.basis}</p>
    </details>
  </section>
{/if}

<style>
  .warnings {
    list-style: none;
    padding: 0;
  }
  .warnings li {
    margin: 0.5rem 0;
  }
  .small {
    font-size: 0.82rem;
  }
</style>

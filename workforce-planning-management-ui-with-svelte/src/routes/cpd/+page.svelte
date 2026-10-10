<!--
  Continuing professional development (`/cpd`): what is required in a period,
  what a worker has recorded (with optional evidence and verification), and
  professional registrations with an expiry. All server-derived; amounts are
  units (hours or points).
-->
<script lang="ts">
  import {
    CPD_CATEGORIES,
    CPD_UNITS,
    cpdOverview,
    cpdProgress,
    createCpdEntry,
    createCpdRequirement,
    listCpdEntries,
    listCpdRequirements,
    listWorkers,
    verifyCpdEntry,
  } from "#lib/api/wpm.js";
  import { percentWithWorkings } from "#lib/format.js";
  import { t, tf, tv } from "#lib/i18n.svelte.js";
  import type { Worker } from "#lib/api/types.js";

  type Overview = Awaited<ReturnType<typeof cpdOverview>>;
  type Progress = Awaited<ReturnType<typeof cpdProgress>>;
  type Entries = Awaited<ReturnType<typeof listCpdEntries>>;
  type Requirements = Awaited<ReturnType<typeof listCpdRequirements>>;

  let workers = $state<Worker[]>([]);
  let workerPid = $state("");
  let overview = $state<Overview | null>(null);
  let requirements = $state<Requirements>([]);
  let progress = $state<Progress | null>(null);
  let entries = $state<Entries>([]);
  let error = $state<string | null>(null);

  // new requirement
  let reqName = $state("");
  let reqUnit = $state<string>("hours");
  let reqAmount = $state(30);
  let reqStart = $state("");
  let reqEnd = $state("");
  // new entry
  let entryDate = $state(new Date().toISOString().slice(0, 10));
  let entryActivity = $state("");
  let entryCategory = $state<string>("course");
  let entryUnit = $state<string>("hours");
  let entryAmount = $state(1);
  let entryUrl = $state("");

  const message = (cause: unknown) =>
    cause instanceof Error ? cause.message : String(cause);

  async function loadShared() {
    try {
      [overview, requirements] = await Promise.all([cpdOverview(), listCpdRequirements()]);
    } catch (cause) {
      error = message(cause);
    }
  }

  async function loadWorker() {
    progress = null;
    entries = [];
    if (!workerPid) return;
    try {
      [progress, entries] = await Promise.all([cpdProgress(workerPid), listCpdEntries(workerPid)]);
    } catch (cause) {
      error = message(cause);
    }
  }

  $effect(() => {
    void (async () => {
      try {
        workers = await listWorkers();
      } catch (cause) {
        error = message(cause);
      }
      await loadShared();
    })();
  });

  async function addRequirement() {
    error = null;
    try {
      await createCpdRequirement({
        name: reqName,
        unit: reqUnit,
        required: reqAmount,
        period_start: reqStart,
        period_end: reqEnd,
      });
      reqName = "";
      await loadShared();
      await loadWorker();
    } catch (cause) {
      error = message(cause);
    }
  }

  async function addEntry() {
    if (!workerPid) return;
    error = null;
    try {
      await createCpdEntry(workerPid, {
        entry_date: entryDate,
        activity: entryActivity,
        category: entryCategory,
        unit: entryUnit,
        amount: entryAmount,
        ...(entryUrl ? { evidence_url: entryUrl } : {}),
      });
      entryActivity = "";
      entryUrl = "";
      await loadWorker();
      await loadShared();
    } catch (cause) {
      error = message(cause);
    }
  }

  async function verify(pid: string) {
    error = null;
    try {
      await verifyCpdEntry(pid);
      await loadWorker();
      await loadShared();
    } catch (cause) {
      error = message(cause);
    }
  }
</script>

<svelte:head><title>{tf("pages.cpd.page_title", { cpd: t("nav.cpd") })}</title></svelte:head>

<h1>{t("nav.cpd")}</h1>
{#if error}<p class="error" data-testid="error">{error}</p>{/if}

{#if overview}
  <h2>{t("pages.cpd.across_the_workforce")}</h2>
  <p class="muted">{overview.derivation}</p>
  <p data-testid="cpd-registrations">
    {tf("pages.cpd.registrations_expiring_expired", { expiring: overview.registrations.expiring, expired: overview.registrations.expired })}
  </p>
  <table data-testid="cpd-overview">
    <thead>
      <tr><th>{t("pages.cpd.requirement")}</th><th>{t("pages.cpd.period")}</th><th>{t("pages.cpd.applies_to")}</th><th>{t("pages.cpd.met_recorded")}</th><th>{t("pages.cpd.met_verified")}</th></tr>
    </thead>
    <tbody>
      {#each overview.requirements as row (row.requirement_pid)}
        <tr>
          <td>{row.name} <span class="muted">({tv(row.unit)})</span></td>
          <td>{row.period_start} → {row.period_end}</td>
          <td>{row.applicable_workers}</td>
          <td>{percentWithWorkings(row.met_recorded)}</td>
          <td>{percentWithWorkings(row.met_verified)}</td>
        </tr>
      {:else}
        <tr><td colspan="5" class="muted">{t("pages.cpd.no_cpd_requirements_defined_yet")}</td></tr>
      {/each}
    </tbody>
  </table>
{/if}

<h2>{t("pages.cpd.define_a_requirement")}</h2>
<form
  onsubmit={(event) => {
    event.preventDefault();
    void addRequirement();
  }}
>
  <label>{t("pages.cpd.name")} <input data-testid="cpd-req-name" bind:value={reqName} required /></label>
  <label>
    {t("pages.cpd.unit")}
    <select bind:value={reqUnit}>{#each CPD_UNITS as u (u)}<option value={u}>{tv(u)}</option>{/each}</select>
  </label>
  <label>{t("pages.cpd.required")} <input type="number" min="1" step="any" bind:value={reqAmount} required /></label>
  <label>{t("pages.cpd.from")} <input type="date" bind:value={reqStart} required /></label>
  <label>{t("pages.cpd.to")} <input type="date" bind:value={reqEnd} required /></label>
  <button type="submit" data-testid="cpd-req-add">{t("pages.cpd.add")}</button>
</form>
<p class="muted">{tf("pages.cpd.requirement_s_defined", { requirements: requirements.length })}</p>

<h2>{t("pages.cpd.a_worker_s_cpd")}</h2>
<p>
  <label>
    {t("pages.cpd.worker")}
    <select
      data-testid="cpd-worker"
      bind:value={workerPid}
      onchange={() => void loadWorker()}
    >
      <option value="">{t("pages.cpd.choose")}</option>
      {#each workers as w (w.pid)}<option value={w.pid}>{w.display_name}</option>{/each}
    </select>
  </label>
</p>

{#if progress}
  <p class="muted">{progress.derivation}</p>
  <table data-testid="cpd-progress">
    <thead>
      <tr><th>{t("pages.cpd.requirement")}</th><th>{t("pages.cpd.required")}</th><th>{t("pages.cpd.recorded")}</th><th>{t("pages.cpd.verified")}</th><th>{t("pages.cpd.remaining")}</th></tr>
    </thead>
    <tbody>
      {#each progress.requirements as row (row.requirement_pid)}
        <tr>
          <td>{row.name} <span class="muted">({row.period_start} → {row.period_end})</span></td>
          <td>{row.required} {tv(row.unit)}</td>
          <td>{row.recorded}</td>
          <td>{row.verified}</td>
          <td class:warn={!row.met}>{row.remaining}</td>
        </tr>
      {:else}
        <tr><td colspan="5" class="muted">{t("pages.cpd.no_requirement_applies_to_this")}</td></tr>
      {/each}
    </tbody>
  </table>
  {#if progress.registrations.length > 0}
    <h3>{t("pages.cpd.registrations")}</h3>
    <ul>
      {#each progress.registrations as r, index (index)}
        <li>{r.body} — {r.expires_on ?? t("pages.cpd.no_expiry")} <span class="chip" class:warn={r.status === "expired" || r.status === "expiring"}>{tv(r.status)}</span></li>
      {/each}
    </ul>
  {/if}

  <h3>{t("pages.cpd.ledger")}</h3>
  <table data-testid="cpd-entries">
    <thead>
      <tr><th>{t("pages.cpd.date")}</th><th>{t("pages.cpd.activity")}</th><th>{t("pages.cpd.category")}</th><th>{t("pages.cpd.amount")}</th><th>{t("pages.cpd.evidence")}</th><th></th></tr>
    </thead>
    <tbody>
      {#each entries as e (e.pid)}
        <tr>
          <td>{e.entry_date}</td>
          <td>{e.activity}{#if e.source === "lms"} <span class="chip">{t("pages.cpd.lms")}</span>{/if}</td>
          <td>{tv(e.category)}</td>
          <td>{e.amount} {tv(e.unit)}</td>
          <td>
            {#if e.evidence_url}<a href={e.evidence_url} rel="noopener noreferrer">{t("pages.cpd.link")}</a>{:else}—{/if}
          </td>
          <td>
            {#if e.verified_on}✓ {e.verified_on}{:else}
              <button type="button" onclick={() => void verify(e.pid)}>{t("pages.cpd.verify")}</button>
            {/if}
          </td>
        </tr>
      {:else}
        <tr><td colspan="6" class="muted">{t("pages.cpd.nothing_recorded_yet")}</td></tr>
      {/each}
    </tbody>
  </table>

  <h3>{t("pages.cpd.record_an_activity")}</h3>
  <form
    onsubmit={(event) => {
      event.preventDefault();
      void addEntry();
    }}
  >
    <label>{t("pages.cpd.date")} <input type="date" bind:value={entryDate} required /></label>
    <label>{t("pages.cpd.activity")} <input data-testid="cpd-entry-activity" bind:value={entryActivity} required /></label>
    <label>
      {t("pages.cpd.category")}
      <select bind:value={entryCategory}>{#each CPD_CATEGORIES as c (c)}<option value={c}>{tv(c)}</option>{/each}</select>
    </label>
    <label>
      {t("pages.cpd.unit")}
      <select bind:value={entryUnit}>{#each CPD_UNITS as u (u)}<option value={u}>{tv(u)}</option>{/each}</select>
    </label>
    <label>{t("pages.cpd.amount")} <input type="number" min="0.01" step="any" bind:value={entryAmount} required /></label>
    <label>{t("pages.cpd.evidence_link")} <input type="url" bind:value={entryUrl} placeholder={t("pages.cpd.https")} /></label>
    <button type="submit" data-testid="cpd-entry-add">{t("pages.cpd.add")}</button>
  </form>
{/if}

<style>
  td.warn, .chip.warn { color: #b45309; font-weight: 600; }
</style>

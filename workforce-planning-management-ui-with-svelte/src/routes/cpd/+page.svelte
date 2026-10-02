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
  import { t } from "#lib/i18n.svelte.js";
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

<svelte:head><title>{t("nav.cpd")} — WPM</title></svelte:head>

<h1>{t("nav.cpd")}</h1>
{#if error}<p class="error" data-testid="error">{error}</p>{/if}

{#if overview}
  <h2>Across the workforce</h2>
  <p class="muted">{overview.derivation}</p>
  <p data-testid="cpd-registrations">
    Registrations: {overview.registrations.expiring} expiring, {overview.registrations.expired} expired
  </p>
  <table data-testid="cpd-overview">
    <thead>
      <tr><th>Requirement</th><th>Period</th><th>Applies to</th><th>Met (recorded)</th><th>Met (verified)</th></tr>
    </thead>
    <tbody>
      {#each overview.requirements as row (row.requirement_pid)}
        <tr>
          <td>{row.name} <span class="muted">({row.unit})</span></td>
          <td>{row.period_start} → {row.period_end}</td>
          <td>{row.applicable_workers}</td>
          <td>{percentWithWorkings(row.met_recorded)}</td>
          <td>{percentWithWorkings(row.met_verified)}</td>
        </tr>
      {:else}
        <tr><td colspan="5" class="muted">No CPD requirements defined yet.</td></tr>
      {/each}
    </tbody>
  </table>
{/if}

<h2>Define a requirement</h2>
<form
  onsubmit={(event) => {
    event.preventDefault();
    void addRequirement();
  }}
>
  <label>Name <input data-testid="cpd-req-name" bind:value={reqName} required /></label>
  <label>
    Unit
    <select bind:value={reqUnit}>{#each CPD_UNITS as u (u)}<option value={u}>{u}</option>{/each}</select>
  </label>
  <label>Required <input type="number" min="1" step="any" bind:value={reqAmount} required /></label>
  <label>From <input type="date" bind:value={reqStart} required /></label>
  <label>To <input type="date" bind:value={reqEnd} required /></label>
  <button type="submit" data-testid="cpd-req-add">Add</button>
</form>
<p class="muted">{requirements.length} requirement(s) defined.</p>

<h2>A worker's CPD</h2>
<p>
  <label>
    Worker
    <select
      data-testid="cpd-worker"
      bind:value={workerPid}
      onchange={() => void loadWorker()}
    >
      <option value="">Choose…</option>
      {#each workers as w (w.pid)}<option value={w.pid}>{w.display_name}</option>{/each}
    </select>
  </label>
</p>

{#if progress}
  <p class="muted">{progress.derivation}</p>
  <table data-testid="cpd-progress">
    <thead>
      <tr><th>Requirement</th><th>Required</th><th>Recorded</th><th>Verified</th><th>Remaining</th></tr>
    </thead>
    <tbody>
      {#each progress.requirements as row (row.requirement_pid)}
        <tr>
          <td>{row.name} <span class="muted">({row.period_start} → {row.period_end})</span></td>
          <td>{row.required} {row.unit}</td>
          <td>{row.recorded}</td>
          <td>{row.verified}</td>
          <td class:warn={!row.met}>{row.remaining}</td>
        </tr>
      {:else}
        <tr><td colspan="5" class="muted">No requirement applies to this worker.</td></tr>
      {/each}
    </tbody>
  </table>
  {#if progress.registrations.length > 0}
    <h3>Registrations</h3>
    <ul>
      {#each progress.registrations as r, index (index)}
        <li>{r.body} — {r.expires_on ?? "no expiry"} <span class="chip" class:warn={r.status === "expired" || r.status === "expiring"}>{r.status}</span></li>
      {/each}
    </ul>
  {/if}

  <h3>Ledger</h3>
  <table data-testid="cpd-entries">
    <thead>
      <tr><th>Date</th><th>Activity</th><th>Category</th><th>Amount</th><th>Evidence</th><th></th></tr>
    </thead>
    <tbody>
      {#each entries as e (e.pid)}
        <tr>
          <td>{e.entry_date}</td>
          <td>{e.activity}</td>
          <td>{e.category}</td>
          <td>{e.amount} {e.unit}</td>
          <td>
            {#if e.evidence_url}<a href={e.evidence_url} rel="noopener noreferrer">link</a>{:else}—{/if}
          </td>
          <td>
            {#if e.verified_on}✓ {e.verified_on}{:else}
              <button type="button" onclick={() => void verify(e.pid)}>Verify</button>
            {/if}
          </td>
        </tr>
      {:else}
        <tr><td colspan="6" class="muted">Nothing recorded yet.</td></tr>
      {/each}
    </tbody>
  </table>

  <h3>Record an activity</h3>
  <form
    onsubmit={(event) => {
      event.preventDefault();
      void addEntry();
    }}
  >
    <label>Date <input type="date" bind:value={entryDate} required /></label>
    <label>Activity <input data-testid="cpd-entry-activity" bind:value={entryActivity} required /></label>
    <label>
      Category
      <select bind:value={entryCategory}>{#each CPD_CATEGORIES as c (c)}<option value={c}>{c}</option>{/each}</select>
    </label>
    <label>
      Unit
      <select bind:value={entryUnit}>{#each CPD_UNITS as u (u)}<option value={u}>{u}</option>{/each}</select>
    </label>
    <label>Amount <input type="number" min="0.01" step="any" bind:value={entryAmount} required /></label>
    <label>Evidence link <input type="url" bind:value={entryUrl} placeholder="https://…" /></label>
    <button type="submit" data-testid="cpd-entry-add">Add</button>
  </form>
{/if}

<style>
  td.warn, .chip.warn { color: #b45309; font-weight: 600; }
</style>

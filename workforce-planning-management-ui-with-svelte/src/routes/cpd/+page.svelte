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
  import CalendarPanel from "#lib/components/CalendarPanel.svelte";
  import DataTable from "#lib/components/DataTable.svelte";
  import type {
    DataGridColumn,
    DataGridRow,
  } from "@lilydesignsystem/svelte-data-grid";

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

  const overviewColumns = $derived<DataGridColumn[]>([
    { id: "name", header: t("pages.cpd.requirement"), sortable: true, rowHeader: true, cell: requirementCell },
    { id: "period_start", header: t("pages.cpd.period"), sortable: true, cell: periodCell },
    { id: "applicable_workers", header: t("pages.cpd.applies_to"), sortable: true, align: "end" },
    {
      id: "met_recorded",
      header: t("pages.cpd.met_recorded"),
      accessor: (r) => percentWithWorkings(r.met_recorded as never),
    },
    {
      id: "met_verified",
      header: t("pages.cpd.met_verified"),
      accessor: (r) => percentWithWorkings(r.met_verified as never),
    },
  ]);
  const progressColumns = $derived<DataGridColumn[]>([
    { id: "name", header: t("pages.cpd.requirement"), sortable: true, rowHeader: true, cell: progressRequirementCell },
    { id: "required", header: t("pages.cpd.required"), sortable: true, align: "end", cell: requiredCell },
    { id: "recorded", header: t("pages.cpd.recorded"), sortable: true, align: "end" },
    { id: "verified", header: t("pages.cpd.verified"), sortable: true, align: "end" },
    { id: "remaining", header: t("pages.cpd.remaining"), sortable: true, align: "end", cell: remainingCell },
  ]);
  const entryColumns = $derived<DataGridColumn[]>([
    { id: "entry_date", type: "date", header: t("pages.cpd.date"), sortable: true },
    { id: "activity", header: t("pages.cpd.activity"), sortable: true, cell: activityCell },
    { id: "category", header: t("pages.cpd.category"), sortable: true, accessor: (r) => tv(String(r.category)) },
    { id: "amount", header: t("pages.cpd.amount"), sortable: true, align: "end", cell: entryAmountCell },
    { id: "evidence_url", header: t("pages.cpd.evidence"), cell: evidenceCell },
    { id: "verified_on", header: t("pages.cpd.verified"), cell: verifyCell },
  ]);

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

{#snippet requirementCell({ row }: { row: DataGridRow })}
  {@const r = row as unknown as Overview["requirements"][number]}
  {r.name} <span class="muted">({tv(r.unit)})</span>
{/snippet}

{#snippet periodCell({ row }: { row: DataGridRow })}
  {row.period_start} → {row.period_end}
{/snippet}

{#snippet progressRequirementCell({ row }: { row: DataGridRow })}
  {row.name} <span class="muted">({row.period_start} → {row.period_end})</span>
{/snippet}

{#snippet requiredCell({ row }: { row: DataGridRow })}
  {row.required} {tv(String(row.unit))}
{/snippet}

{#snippet remainingCell({ row }: { row: DataGridRow })}
  <span class:warn={!row.met}>{row.remaining}</span>
{/snippet}

{#snippet activityCell({ row }: { row: DataGridRow })}
  {row.activity}{#if row.source === "lms"}
    {#if row.lms_url}
      <a class="chip" href={String(row.lms_url)} target="_blank" rel="noopener noreferrer" title={t("pages.cpd.lms_open")}
        >{t("pages.cpd.lms")} ↗</a
      >
    {:else}
      <span class="chip">{t("pages.cpd.lms")}</span>
    {/if}
  {/if}
{/snippet}

{#snippet entryAmountCell({ row }: { row: DataGridRow })}
  {row.amount} {tv(String(row.unit))}
{/snippet}

{#snippet evidenceCell({ row }: { row: DataGridRow })}
  {#if row.evidence_url}<a href={String(row.evidence_url)} rel="noopener noreferrer">{t("pages.cpd.link")}</a>{:else}—{/if}
{/snippet}

{#snippet verifyCell({ row }: { row: DataGridRow })}
  {#if row.verified_on}✓ {row.verified_on}{:else}
    <button type="button" onclick={() => void verify(String(row.pid))}>{t("pages.cpd.verify")}</button>
  {/if}
{/snippet}

<h1>{t("nav.cpd")}</h1>
{#if error}<p class="error" data-testid="error">{error}</p>{/if}

{#if overview}
  <h2>{t("pages.cpd.across_the_workforce")}</h2>
  <p class="muted">{overview.derivation}</p>
  <p data-testid="cpd-registrations">
    {tf("pages.cpd.registrations_expiring_expired", { expiring: overview.registrations.expiring, expired: overview.registrations.expired })}
  </p>
  {#if overview.requirements.length === 0}
    <p class="muted" data-testid="cpd-overview">{t("pages.cpd.no_cpd_requirements_defined_yet")}</p>
  {:else}
    <DataTable
      testid="cpd-overview"
      exportName="cpd-overview"
      label={t("pages.cpd.across_the_workforce")}
      columns={overviewColumns}
      rows={overview.requirements as unknown as DataGridRow[]}
      rowId={(row) => String(row.requirement_pid)}
    />
  {/if}
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
  {#if progress.requirements.length === 0}
    <p class="muted" data-testid="cpd-progress">{t("pages.cpd.no_requirement_applies_to_this")}</p>
  {:else}
    <DataTable
      testid="cpd-progress"
      label={t("pages.cpd.a_worker_s_cpd")}
      columns={progressColumns}
      rows={progress.requirements as unknown as DataGridRow[]}
      rowId={(row) => String(row.requirement_pid)}
    />
  {/if}
  {#if progress.registrations.length > 0}
    <h3>{t("pages.cpd.registrations")}</h3>
    <ul>
      {#each progress.registrations as r, index (index)}
        <li>{r.body} — {r.expires_on ?? t("pages.cpd.no_expiry")} <span class="chip" class:warn={r.status === "expired" || r.status === "expiring"}>{tv(r.status)}</span></li>
      {/each}
    </ul>
  {/if}

  <CalendarPanel
    testid="cpd-calendar"
    label={t("calendar.cpd")}
    events={[
      ...entries.map((e) => ({ id: e.pid, date: e.entry_date, title: `${e.activity} · ${e.amount} ${tv(e.unit)}` })),
      ...progress.registrations.flatMap((r, i) =>
        r.expires_on ? [{ id: `registration-${i}`, date: r.expires_on, title: `${r.body} · ${tv(r.status)}` }] : [],
      ),
    ]}
    ranges={progress.requirements.map((r) => ({ start: r.period_start, end: r.period_end, kind: "requirement", label: r.name }))}
  />

  <h3>{t("pages.cpd.ledger")}</h3>
  {#if entries.length === 0}
    <p class="muted" data-testid="cpd-entries">{t("pages.cpd.nothing_recorded_yet")}</p>
  {:else}
    <DataTable
      testid="cpd-entries"
      label={t("pages.cpd.ledger")}
      columns={entryColumns}
      rows={entries as unknown as DataGridRow[]}
      rowId={(row) => String(row.pid)}
      pageSize={25}
    />
  {/if}

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
  .warn, .chip.warn { color: #b45309; font-weight: 600; }
</style>

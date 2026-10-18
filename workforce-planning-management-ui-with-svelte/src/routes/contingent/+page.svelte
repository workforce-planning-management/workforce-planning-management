<!--
  Contingent workforce (`/contingent`, WPM-R62, WPM-R86): contractors and fixed-term staff in
  aggregate — the share of FTE (each apart), engagements ending in the next 30, 60 and 90 calendar
  days, past their end with no decision, extended many times, and long-running contractors; the
  months in which funding ends; the trend; and the programme roll-up. HR only. Nobody is named in a
  figure: a count of one to nine is withheld and shown as such (never as 0), no total is given, and
  "Show who" opens the list behind a figure by worker number — a read the service audits.
-->
<script lang="ts">
  import { ApiError } from "#lib/api/client.js";
  import {
    contingentList,
    contingentWorkforce,
    openConversionPlans,
    type OpenConversionRow,
    programmeRollup,
    type ContingentList,
    type ContingentWorkforce,
    type ProgrammeRollup,
    type Withheld,
  } from "#lib/api/wpm.js";
  import { l, t, tf, tv } from "#lib/i18n.svelte.js";
  import DataTable from "#lib/components/DataTable.svelte";
  import type { DataGridColumn, DataGridRow } from "@lilydesignsystem/svelte-data-grid";

  let view = $state<ContingentWorkforce | null>(null);
  let rollup = $state<ProgrammeRollup | null>(null);
  let forbidden = $state(false);
  let error = $state<string | null>(null);
  let open = $state<ContingentList | null>(null);
  let people = $state<Awaited<ReturnType<typeof contingentList>>["people"] | null>(null);
  let listError = $state<string | null>(null);
  let conversions = $state<OpenConversionRow[] | null>(null);
  let conversionsOpen = $state(false);
  let conversionsError = $state<string | null>(null);

  /** Open plans and engagements past their end with none: names workers by number, and the service audits the read. */
  async function showConversions() {
    conversionsOpen = !conversionsOpen;
    if (!conversionsOpen || conversions !== null) return;
    conversionsError = null;
    try {
      conversions = await openConversionPlans();
    } catch (cause) {
      conversionsError = cause instanceof Error ? cause.message : String(cause);
    }
  }

  $effect(() => {
    let current = true;
    void (async () => {
      try {
        const [loaded, programmes] = await Promise.all([contingentWorkforce(), programmeRollup()]);
        if (!current) return;
        view = loaded;
        rollup = programmes;
        error = null;
      } catch (cause) {
        if (!current) return;
        if (cause instanceof ApiError && (cause.status === 403 || cause.status === 401)) {
          forbidden = true;
        } else {
          error = cause instanceof Error ? cause.message : String(cause);
        }
      }
    })();
    return () => {
      current = false;
    };
  });

  async function show(list: ContingentList) {
    if (open === list) {
      open = null;
      people = null;
      return;
    }
    open = list;
    people = null;
    listError = null;
    try {
      const result = await contingentList(list);
      if (open === list) people = result.people;
    } catch (cause) {
      listError = cause instanceof Error ? cause.message : String(cause);
    }
  }

  /** A count: a number, or "withheld" for one to nine people. Never 0 for a withheld one. */
  const count = (n: Withheld) => (n === null ? null : String(n));
  const pct = (n: number | null) => (n === null ? "—" : `${n.toFixed(1)}%`);
  const fte = (n: number | null) => (n === null ? "—" : n.toFixed(2));

  const pctFmt = (v: unknown) => pct(v as number | null);
  const fteFmt = (v: unknown) => fte(v as number | null);

  const departmentColumns = $derived<DataGridColumn[]>([
    { id: "department", header: t("common.department"), sortable: true },
    { id: "headcount", header: t("contingent.people"), sortable: true, align: "end" },
    { id: "contractor_percent_of_fte", header: t("contingent.contractors"), sortable: true, align: "end", format: pctFmt },
    { id: "fixed_term_percent_of_fte", header: t("contingent.fixedTerm"), sortable: true, align: "end", format: pctFmt },
  ]);

  const whoColumns = $derived<DataGridColumn[]>([
    { id: "worker_number", pinned: "start", header: t("contingent.workerNumber"), sortable: true },
    { id: "department", header: t("common.department"), sortable: true },
    { id: "employment_type", header: t("engagement.basis"), sortable: true, accessor: (r) => tv(String(r.employment_type)) },
    { id: "engagement_ends_on", type: "date", header: t("engagement.ends"), sortable: true },
  ]);

  const conversionColumns = $derived<DataGridColumn[]>([
    { id: "worker_number", pinned: "start", header: t("contingent.workerNumber"), sortable: true, cell: conversionWorkerCell },
    { id: "department", header: t("common.department"), sortable: true },
    { id: "employment_type", header: t("engagement.basis"), sortable: true, accessor: (r) => tv(String(r.employment_type)) },
    { id: "engagement_ends_on", type: "date", header: t("engagement.ends"), sortable: true },
    {
      id: "plan",
      header: t("conversion.plan"),
      sortable: true,
      accessor: (r) => {
        const plan = r.plan as OpenConversionRow["plan"];
        return plan ? `${tv(plan.intent)} · ${tv(plan.status)}` : null;
      },
    },
    {
      id: "flags",
      header: t("conversion.flags"),
      accessor: (r) => (r.flags as string[]).map((f) => tv(f)).join(", ") || null,
    },
  ]);

  const fundingColumns = $derived<DataGridColumn[]>([
    {
      id: "month",
      header: t("contingent.month"),
      accessor: (r) => (r.month === "already_ended" ? t("contingent.alreadyEnded") : r.month),
    },
    { id: "people", header: t("contingent.people"), align: "end", cell: withheldCell },
    { id: "fte", header: t("contingent.fte"), align: "end", format: fteFmt },
  ]);

  const trendColumns = $derived<DataGridColumn[]>([
    { id: "as_of", header: t("contingent.date"), sortable: true },
    { id: "contractor_percent_of_fte", header: t("contingent.contractors"), sortable: true, align: "end", format: pctFmt },
    { id: "fixed_term_percent_of_fte", header: t("contingent.fixedTerm"), sortable: true, align: "end", format: pctFmt },
  ]);

  const programmeColumns = $derived<DataGridColumn[]>([
    { id: "month", header: t("contingent.month"), sortable: true },
    { id: "funded_fte", header: t("contingent.fundedFte"), sortable: true, align: "end", format: fteFmt },
    { id: "contractor_percent_of_fte", header: t("contingent.contractors"), sortable: true, align: "end", format: pctFmt },
    { id: "fixed_term_percent_of_fte", header: t("contingent.fixedTerm"), sortable: true, align: "end", format: pctFmt },
    { id: "engagements_ending", header: t("contingent.engagementsEnding"), sortable: true, align: "end", cell: withheldCell },
    { id: "funding_ending", header: t("contingent.fundingEnding"), sortable: true, align: "end", cell: withheldCell },
  ]);

  const figures = $derived(
    view === null
      ? []
      : ([
          ["ending_within_30", t("contingent.ending30"), view.engagements.ending_within_30_calendar_days],
          ["ending_31_to_60", t("contingent.ending60"), view.engagements.ending_31_to_60_calendar_days],
          ["ending_61_to_90", t("contingent.ending90"), view.engagements.ending_61_to_90_calendar_days],
          ["past_end_undecided", t("contingent.pastEnd"), view.engagements.past_end_with_no_decision],
          [
            "much_extended",
            tf("contingent.extended", { times: view.engagements.extended_more_than.times }),
            view.engagements.extended_more_than.people,
          ],
          [
            "long_running_contractors",
            tf("contingent.longRunning", {
              months: view.engagements.contractors_engaged_longer_than.calendar_months,
            }),
            view.engagements.contractors_engaged_longer_than.people,
          ],
        ] as Array<[ContingentList, string, Withheld]>),
  );
</script>

<svelte:head><title>{t("titles.contingent")}</title></svelte:head>

{#snippet conversionWorkerCell({ row }: { row: DataGridRow })}
  <a href={l(`/workers/${row.worker_pid}`)}>{row.worker_number}</a>
{/snippet}

{#snippet withheldCell({ value }: { value: unknown })}
  {#if value === null || value === undefined}<span class="muted">{t("contingent.withheld")}</span>{:else}{value}{/if}
{/snippet}

<h1>{t("contingent.title")}</h1>
<p class="muted">{t("contingent.hint")}</p>

{#if forbidden}
  <p class="muted" data-testid="contingent-forbidden">{t("contingent.forbidden")}</p>
{:else if error}
  <p class="error" data-testid="error">{t("common.error")}: {error}</p>
{:else if view === null}
  <p>{t("common.loading")}</p>
{:else}
  <section class="panel" data-testid="contingent-share">
    <h2>{t("contingent.share")}</h2>
    <p>
      <strong data-testid="share-contractor">{pct(view.contractor_percent_of_fte)}</strong> {t("contingent.contractors")} ·
      <strong data-testid="share-fixed-term">{pct(view.fixed_term_percent_of_fte)}</strong> {t("contingent.fixedTerm")}
      <span class="muted">· {tf("contingent.employed", { headcount: view.headcount, fte: fte(view.fte) })}</span>
    </p>
    {#if view.departments_shown}
      <DataTable
        testid="contingent-departments"
        exportName="contingent-departments"
        label={t("contingent.share")}
        columns={departmentColumns}
        rows={view.by_department as unknown as DataGridRow[]}
        rowId={(row) => String(row.department)}
      />
    {:else}
      <p class="muted" data-testid="departments-withheld">{t("contingent.departmentsWithheld")}</p>
    {/if}
  </section>

  <section class="panel" data-testid="contingent-engagements">
    <h2>{t("contingent.engagements")}</h2>
    <div class="scroll">
      <!-- Plain table: a headerless label / count / button list, and the grid needs column headers. -->
      <table>
        <tbody>
          {#each figures as [list, label, n] (list)}
            <tr data-testid={`figure-${list}`}>
              <td>{label}</td>
              <td data-testid={`figure-${list}-count`}>
                {#if count(n) === null}<span class="muted">{t("contingent.withheld")}</span>{:else}<strong>{count(n)}</strong>{/if}
              </td>
              <td>
                <button type="button" data-testid={`show-${list}`} aria-expanded={open === list} onclick={() => void show(list)}>
                  {open === list ? t("contingent.hideWho") : t("contingent.showWho")}
                </button>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
    {#if open}
      <div data-testid="contingent-who">
        <h3>{t("contingent.whoTitle")} — {tv(open)}</h3>
        {#if listError}
          <p class="error">{t("common.error")}: {listError}</p>
        {:else if people === null}
          <p>{t("common.loading")}</p>
        {:else if people.length === 0}
          <p class="muted">{t("contingent.nobody")}</p>
        {:else}
          <DataTable
            testid="contingent-who-table"
            label={t("contingent.whoTitle")}
            columns={whoColumns}
            rows={people as unknown as DataGridRow[]}
            rowId={(row) => String(row.worker_pid)}
            pageSize={25}
          />
        {/if}
      </div>
    {/if}
    <p class="muted">{t("contingent.noTotal")}</p>
  </section>

  <section class="panel" data-testid="contingent-conversions">
    <h2>{t("conversion.listTitle")}</h2>
    <p class="muted">{t("conversion.listHint")}</p>
    <button type="button" data-testid="show-conversions" aria-expanded={conversionsOpen} onclick={() => void showConversions()}>
      {conversionsOpen ? t("conversion.hidePlans") : t("conversion.showPlans")}
    </button>
    {#if conversionsOpen}
      {#if conversionsError}
        <p class="error">{t("common.error")}: {conversionsError}</p>
      {:else if conversions === null}
        <p>{t("common.loading")}</p>
      {:else if conversions.length === 0}
        <p class="muted" data-testid="conversions-none">{t("conversion.nobody")}</p>
      {:else}
        <DataTable
          testid="conversion-table"
          label={t("conversion.listTitle")}
          columns={conversionColumns}
          rows={conversions as unknown as DataGridRow[]}
          rowId={(row) => String(row.worker_pid)}
          pageSize={25}
        />
      {/if}
    {/if}
  </section>

  <section class="panel" data-testid="contingent-funding">
    <h2>{t("contingent.fundingByMonth")}</h2>
    <DataTable
      label={t("contingent.fundingByMonth")}
      columns={fundingColumns}
      rows={view.funding_ends as unknown as DataGridRow[]}
      rowId={(row) => String(row.month)}
    />
  </section>

  <section class="panel" data-testid="contingent-trend">
    <h2>{t("contingent.trend")}</h2>
    {#if view.trend.status === "insufficient_history"}
      <p class="muted" data-testid="trend-insufficient">{t("contingent.trendInsufficient")}</p>
    {:else}
      <DataTable
        testid="trend-table"
        label={t("contingent.trend")}
        columns={trendColumns}
        rows={view.trend.points as unknown as DataGridRow[]}
        rowId={(row) => String(row.as_of)}
      />
    {/if}
  </section>

  {#if rollup}
    <section class="panel" data-testid="contingent-programmes">
      <h2>{t("contingent.programmes")}</h2>
      <p class="muted">{t("contingent.programmesHint")}</p>
      {#if rollup.withheld_programmes > 0}
        <p class="muted" data-testid="programmes-withheld">
          {tf("contingent.programmesWithheld", { count: rollup.withheld_programmes })}
        </p>
      {/if}
      {#if rollup.programmes.length === 0}
        <p class="muted" data-testid="programmes-none">{t("contingent.noProgrammes")}</p>
      {/if}
      {#each rollup.programmes as p (p.programme)}
        <h3>{p.programme}</h3>
        <DataTable
          testid={`programme-${p.programme}`}
          label={p.programme}
          columns={programmeColumns}
          rows={p.months as unknown as DataGridRow[]}
          rowId={(row) => String(row.month)}
        />
      {/each}
    </section>
  {/if}
{/if}

<style>
  .scroll {
    overflow-x: auto;
  }
  .scroll :global(td),
  .scroll :global(th) {
    overflow-wrap: normal;
  }
  .scroll :global(td:nth-child(2)) {
    white-space: nowrap;
  }
  .scroll :global(button) {
    white-space: nowrap;
  }
</style>

<script lang="ts">
  import { formatDateRange } from "#lib/locale-format.js";
  import { errorText } from "#lib/write.js";
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import { page } from "$app/state";
  import {
    getRun,
    minimumWageCheck,
    money,
    runAction,
    runPayslips,
  } from "#lib/api/wpm.js";
  import { i18n, t, l, tv } from "#lib/i18n.svelte.js";
  import { createLoader } from "#lib/loader.svelte.js";
  import type {
    MinimumWageCheck,
    MinimumWageRow,
    Payslip,
    PayrollRun,
  } from "#lib/api/types.js";
  import DataTable from "#lib/components/DataTable.svelte";
  import OvertimeCheck from "#lib/components/OvertimeCheck.svelte";
  import type {
    DataGridColumn,
    DataGridRow,
  } from "@lilydesignsystem/svelte-data-grid";

  /** The action(s) each run status offers (WPM-D5: derive, approve, pay). */
  const ACTIONS: Record<string, ("calculate" | "approve" | "pay" | "reopen")[]> = {
    draft: ["calculate"],
    calculated: ["approve", "reopen"],
    approved: ["pay"],
    paid: [],
  };

  let actionError = $state<string | null>(null);
  // The check is an audited read, so it runs only when asked, and is dropped when the run changes.
  let wage = $state<MinimumWageCheck | null>(null);
  let wageError = $state<string | null>(null);
  let wageBusy = $state(false);

  const pid = $derived(page.params.pid ?? "");

  // Going from one run to another drops a slow answer for the first.
  const loader = createLoader(async () => {
    const found = await getRun(pid);
    const slips: Payslip[] = found.status === "draft" ? [] : await runPayslips(pid);
    return { run: found as PayrollRun, payslips: slips };
  });
  const run = $derived(loader.data?.run ?? null);
  const payslips = $derived(loader.data?.payslips ?? []);
  const error = $derived(loader.error);
  const load = () => loader.load();

  $effect(() => {
    if (pid) void load();
  });

  $effect(() => {
    void pid;
    wage = null;
    wageError = null;
  });

  async function checkWage() {
    const asked = pid;
    wageBusy = true;
    wageError = null;
    try {
      const answer = await minimumWageCheck(asked);
      if (asked === pid) wage = answer;
    } catch (cause) {
      if (asked === pid) wageError = errorText(cause);
    } finally {
      wageBusy = false;
    }
  }

  /** A reason or band the locale has no words for falls back to its own name, never to nothing. */
  const words = (value: string | null) => (value === null ? "—" : tv(value));

  async function act(action: "calculate" | "approve" | "pay" | "reopen") {
    actionError = null;
    try {
      await runAction(pid, action);
      await load();
    } catch (cause) {
      actionError = errorText(cause);
    }
  }

  const wageColumns = $derived<DataGridColumn[]>([
    {
      id: "worker_pid",
      header: t("nav.workers"),
      sortable: true,
      accessor: (r) => String((r as unknown as MinimumWageRow).worker_pid).slice(0, 8),
      cell: wageWorkerCell,
    },
    {
      id: "outcome",
      header: t("minWage.result"),
      sortable: true,
      accessor: (r) => t(`minWage.${(r as unknown as MinimumWageRow).outcome}`),
    },
    {
      id: "reason",
      header: t("minWage.why"),
      accessor: (r) => words((r as unknown as MinimumWageRow).reason),
    },
    {
      id: "band",
      header: t("payScales.band"),
      accessor: (r) => words((r as unknown as MinimumWageRow).band),
    },
  ]);

  const columns = $derived<DataGridColumn[]>([
    {
      id: "worker_pid",
      header: t("nav.workers"),
      sortable: true,
      accessor: (r) => String((r as unknown as Payslip).worker_pid).slice(0, 8),
      cell: workerCell,
    },
    {
      id: "gross_minor",
      header: t("pay.gross"),
      sortable: true,
      align: "end",
      format: (v, r) => money(Number(v), (r as unknown as Payslip).currency, i18n.locale),
    },
    {
      id: "statutory_overtime_minor",
      header: t("pay.statutoryOvertime"),
      align: "end",
      format: (v, r) => money(Number(v ?? 0), (r as unknown as Payslip).currency, i18n.locale),
    },
    { id: "deductions", header: t("pay.deductions"), cell: deductionsCell },
    { id: "employer_costs", header: t("pay.employerCost"), cell: employerCostCell },
    {
      id: "net_minor",
      header: t("pay.net"),
      sortable: true,
      align: "end",
      format: (v, r) => money(Number(v), (r as unknown as Payslip).currency, i18n.locale),
    },
  ]);
</script>

<svelte:head><title>{page.data.title}</title></svelte:head>

{#snippet workerCell({ row, formatted }: { row: DataGridRow; formatted: string })}
  {@const slip = row as unknown as Payslip}
  <a href={l(`/workers/${slip.worker_pid}`)}>{formatted}</a>
{/snippet}

{#snippet wageWorkerCell({ row, formatted }: { row: DataGridRow; formatted: string })}
  {@const entry = row as unknown as MinimumWageRow}
  <a href={l(`/workers/${entry.worker_pid}`)}>{formatted}</a>
{/snippet}

{#snippet employerCostCell({ row }: { row: DataGridRow })}
  {@const slip = row as unknown as Payslip}
  {#each slip.employer_costs ?? [] as line (line.name)}
    <span class="chip"
      >{line.name}: {line.unresolved
        ? t("pay.employerUnresolved")
        : money(line.amount_minor ?? 0, slip.currency, i18n.locale)}</span
    >
  {/each}
{/snippet}

{#snippet deductionsCell({ row }: { row: DataGridRow })}
  {@const slip = row as unknown as Payslip}
  {#each slip.deductions as deduction (deduction.label)}
    <span class="chip">{deduction.label}: {money(deduction.amount_minor, slip.currency, i18n.locale)}</span>
  {/each}
{/snippet}

{#if error}
  <ErrorNote testid="error" prefixed message={error} />
{:else if run === null}
  <p>{t("common.loading")}</p>
{:else}
  <h1>
    {formatDateRange(run.period_start, run.period_end)}
    <span class="chip" data-testid="run-status">{tv(run.status)}</span>
  </h1>
  <div class="panel">
    {#each ACTIONS[run.status] ?? [] as action (action)}
      <button data-testid={`action-${action}`} onclick={() => void act(action)}>
        {action === "calculate" ? t("pay.calculate") : action === "pay" ? t("pay.pay") : action}
      </button>
    {/each}
    {#if actionError}
      <ErrorNote testid="action-error" message={actionError} />
    {/if}
  </div>

  <h2>{t("emp.payslips")}</h2>
  <DataTable
    testid="payslips"
    label={t("emp.payslips")}
    columns={columns}
    rows={payslips as unknown as DataGridRow[]}
    rowId={(row) => String(row.pid)}
    pageSize={25}
  />

  {#if run.status !== "draft"}
    <h2>{t("minWage.title")}</h2>
    <div class="panel" data-testid="min-wage">
      <button data-testid="min-wage-run" disabled={wageBusy} onclick={() => void checkWage()}>
        {t("minWage.run")}
      </button>
      {#if wageError}
        <ErrorNote testid="min-wage-error" message={wageError} />
      {/if}
      {#if wage}
        <p data-testid="min-wage-note">{wage.note}</p>
        <p>
          {#each Object.entries(wage.counts) as [outcome, count] (outcome)}
            <span class="chip" data-testid={`min-wage-count-${outcome}`}
              >{t(`minWage.${outcome}` as "minWage.ok")}: {count}</span
            >
          {/each}
        </p>
        <DataTable
          testid="min-wage-rows"
          label={t("minWage.title")}
          columns={wageColumns}
          rows={wage.workers as unknown as DataGridRow[]}
          rowId={(row) => String(row.worker_pid)}
          pageSize={25}
        />
      {/if}
    </div>
  {/if}

  {#if run.status !== "draft"}
    <OvertimeCheck {pid} />
  {/if}
{/if}

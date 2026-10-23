<script lang="ts">
  import { page } from "$app/state";
  import { getRun, money, runAction, runPayslips } from "#lib/api/wpm.js";
  import { i18n, t, l, tv } from "#lib/i18n.svelte.js";
  import { createLoader } from "#lib/loader.svelte.js";
  import type { Payslip, PayrollRun } from "#lib/api/types.js";
  import DataTable from "#lib/components/DataTable.svelte";
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

  async function act(action: "calculate" | "approve" | "pay" | "reopen") {
    actionError = null;
    try {
      await runAction(pid, action);
      await load();
    } catch (cause) {
      actionError = cause instanceof Error ? cause.message : String(cause);
    }
  }

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
    { id: "deductions", header: t("pay.deductions"), cell: deductionsCell },
    {
      id: "net_minor",
      header: t("pay.net"),
      sortable: true,
      align: "end",
      format: (v, r) => money(Number(v), (r as unknown as Payslip).currency, i18n.locale),
    },
  ]);
</script>

{#snippet workerCell({ row, formatted }: { row: DataGridRow; formatted: string })}
  {@const slip = row as unknown as Payslip}
  <a href={l(`/workers/${slip.worker_pid}`)}>{formatted}</a>
{/snippet}

{#snippet deductionsCell({ row }: { row: DataGridRow })}
  {@const slip = row as unknown as Payslip}
  {#each slip.deductions as deduction (deduction.label)}
    <span class="chip">{deduction.label}: {money(deduction.amount_minor, slip.currency, i18n.locale)}</span>
  {/each}
{/snippet}

{#if error}
  <p class="error" data-testid="error">{t("common.error")}: {error}</p>
{:else if run === null}
  <p>{t("common.loading")}</p>
{:else}
  <h1>
    {run.period_start} → {run.period_end}
    <span class="chip" data-testid="run-status">{tv(run.status)}</span>
  </h1>
  <div class="panel">
    {#each ACTIONS[run.status] ?? [] as action (action)}
      <button data-testid={`action-${action}`} onclick={() => void act(action)}>
        {action === "calculate" ? t("pay.calculate") : action === "pay" ? t("pay.pay") : action}
      </button>
    {/each}
    {#if actionError}
      <p class="error" data-testid="action-error">{actionError}</p>
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
{/if}

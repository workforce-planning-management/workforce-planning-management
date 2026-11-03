<!--
  The advisory statutory-overtime check of a payroll run (WPM-R219, WPM-D157, WPM-T418, WPM-T419). The check is an audited read, so it
  runs only when asked. A person with no recorded status, no seat or no place of work is "not enough to compare", never "no overtime";
  an exempt post says whether its salary is below the configured weekly floor, or that none is configured. It changes no payslip.
-->
<script lang="ts">
  import { errorText } from "#lib/write.js";
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import DataTable from "#lib/components/DataTable.svelte";
  import { formatNumber } from "#lib/locale-format.js";
  import { i18n, l, t } from "#lib/i18n.svelte.js";
  import { money, overtimeCheck } from "#lib/api/wpm.js";
  import type { OvertimeCheck, OvertimeRow } from "#lib/api/types.js";
  import type {
    DataGridColumn,
    DataGridRow,
  } from "@lilydesignsystem/svelte-data-grid";

  let { pid }: { pid: string } = $props();

  let answer = $state<OvertimeCheck | null>(null);
  let error = $state<string | null>(null);
  let busy = $state(false);

  // Going from one run to another drops the answer for the first.
  $effect(() => {
    void pid;
    answer = null;
    error = null;
  });

  async function run() {
    const asked = pid;
    busy = true;
    error = null;
    try {
      const found = await overtimeCheck(asked);
      if (asked === pid) answer = found;
    } catch (cause) {
      if (asked === pid) error = errorText(cause);
    } finally {
      busy = false;
    }
  }

  const reason = (value: string | null) =>
    value === null ? "—" : t(`overtimeCheck.reason_${value}` as never);

  const floorWords = (row: OvertimeRow) =>
    row.outcome !== "exempt"
      ? "—"
      : row.below_exempt_floor === null || row.below_exempt_floor === undefined
        ? t("overtimeCheck.floor_not_configured")
        : row.below_exempt_floor
          ? t("overtimeCheck.floor_below")
          : t("overtimeCheck.floor_ok");

  const columns = $derived<DataGridColumn[]>([
    {
      id: "worker_pid",
      header: t("nav.workers"),
      sortable: true,
      accessor: (r) => String((r as unknown as OvertimeRow).worker_pid).slice(0, 8),
      cell: workerCell,
    },
    {
      id: "outcome",
      header: t("overtimeCheck.result"),
      sortable: true,
      accessor: (r) => t(`overtimeCheck.${(r as unknown as OvertimeRow).outcome}` as never),
    },
    {
      id: "reason",
      header: t("overtimeCheck.why"),
      accessor: (r) => reason((r as unknown as OvertimeRow).reason),
    },
    {
      id: "overtime_minutes",
      header: t("overtimeCheck.minutes"),
      align: "end",
      accessor: (r) => {
        const m = (r as unknown as OvertimeRow).overtime_minutes;
        return m === undefined ? "—" : formatNumber(m);
      },
    },
    {
      id: "premium_minor",
      header: t("overtimeCheck.premium"),
      align: "end",
      accessor: (r) => {
        const row = r as unknown as OvertimeRow;
        return row.premium_minor === undefined || row.premium_minor === null || !row.currency
          ? "—"
          : money(row.premium_minor, row.currency, i18n.locale);
      },
    },
    {
      id: "floor",
      header: t("overtimeCheck.floor"),
      accessor: (r) => floorWords(r as unknown as OvertimeRow),
    },
  ]);
</script>

{#snippet workerCell({ row, formatted }: { row: DataGridRow; formatted: string })}
  {@const entry = row as unknown as OvertimeRow}
  <a href={l(`/workers/${entry.worker_pid}`)}>{formatted}</a>
{/snippet}

<h2>{t("overtimeCheck.title")}</h2>
<div class="panel" data-testid="overtime-check">
  <button data-testid="overtime-run" disabled={busy} onclick={() => void run()}>
    {t("overtimeCheck.run")}
  </button>
  {#if error}<ErrorNote testid="overtime-error" message={error} />{/if}
  {#if answer}
    <p data-testid="overtime-note">{answer.note}</p>
    <p>
      {#each Object.entries(answer.counts) as [outcome, count] (outcome)}
        <span class="chip" data-testid={`overtime-count-${outcome}`}
          >{t(`overtimeCheck.${outcome}` as never)}: {count}</span
        >
      {/each}
    </p>
    <DataTable
      testid="overtime-rows"
      label={t("overtimeCheck.title")}
      columns={columns}
      rows={answer.workers as unknown as DataGridRow[]}
      rowId={(row) => String(row.worker_pid)}
      pageSize={25}
    />
  {/if}
</div>

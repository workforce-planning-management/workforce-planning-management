<!--
  Payslip queries (WPM-R178): what people have raised about their payslips, oldest first, for HR and payroll to close. A
  caller who may not read them sees nothing here. Closing changes no amount; a correction is made in the payroll run.
-->
<script lang="ts">
  import { formatDate, formatDateRange } from "#lib/locale-format.js";
  import { guardedLoad } from "#lib/loader.svelte.js";
  import { errorText } from "#lib/write.js";
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import { closePayslipQuery, listPayslipQueries } from "#lib/api/wpm.js";
  import type { PayslipQueryRow } from "#lib/api/wpm.js";
  import { t } from "#lib/i18n.svelte.js";

  let rows = $state<PayslipQueryRow[] | null>(null);
  let error = $state<string | null>(null);

  const load = guardedLoad(
    () => listPayslipQueries(),
    (found) => {
      rows = found;
    },
    () => {
      // Refused: this caller does not deal with payslip queries, so the panel stays away.
      rows = null;
    },
  );

  $effect(() => {
    void load();
  });

  async function close(pid: string) {
    error = null;
    try {
      await closePayslipQuery(pid);
      await load();
    } catch (cause) {
      error = errorText(cause);
    }
  }
</script>

{#if rows}
  <section class="panel" data-testid="payslip-queries">
    <h2>{t("payslips.queueTitle")}</h2>
    <p class="muted">{t("payslips.queueHint")}</p>
    {#if error}<ErrorNote testid="error" message={error} />{/if}
    {#if rows.length === 0}
      <p class="muted" data-testid="payslip-queries-none">{t("payslips.queueNone")}</p>
    {:else}
      <table>
        <thead>
          <tr>
            <th scope="col">{t("payslips.queuePerson")}</th>
            <th scope="col">{t("payslips.period")}</th>
            <th scope="col">{t("payslips.queueComment")}</th>
            <th scope="col">{t("payslips.queueRaised")}</th>
            <th scope="col"><span class="visually-hidden">{t("payslips.close")}</span></th>
          </tr>
        </thead>
        <tbody>
          {#each rows as row (row.pid)}
            <tr data-testid="payslip-query-row">
              <td>{row.display_name}</td>
              <td>{row.period_start && row.period_end ? formatDateRange(row.period_start, row.period_end) : "—"}</td>
              <td>{row.comment}</td>
              <td>{formatDate(row.raised_on)}</td>
              <td>
                <button type="button" onclick={() => close(row.pid)} data-testid="payslip-query-close">
                  {t("payslips.close")}
                </button>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    {/if}
  </section>
{/if}

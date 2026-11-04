<!--
  My payslips (WPM-R178): the person's own payslips from approved and paid runs, the year to date per tax year and
  currency, a printable payslip (the browser's print, no file made by the server), and a query payroll sees and closes.
  A query changes no amount. With no tax year start configured the page says so, and shows no total.
-->
<script lang="ts">
  import { tick } from "svelte";
  import { formatDate, formatDateRange } from "#lib/locale-format.js";
  import { guardedLoad } from "#lib/loader.svelte.js";
  import { errorText } from "#lib/write.js";
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import { money, myPayslips, raisePayslipQuery } from "#lib/api/wpm.js";
  import type { MyPayslips, PayslipYearToDate } from "#lib/api/wpm.js";
  import { i18n, t, tf } from "#lib/i18n.svelte.js";

  let data = $state<MyPayslips | null>(null);
  let error = $state<string | null>(null);
  let asking = $state<string | null>(null);
  let comment = $state("");
  let printing = $state<string | null>(null);

  const load = guardedLoad(
    () => myPayslips(),
    (found) => {
      data = found;
    },
    (cause) => {
      error = errorText(cause);
    },
  );

  $effect(() => {
    void load();
  });

  const yearKey = (year: PayslipYearToDate) => [year.starts_on, year.currency].join(" ");

  async function send(pid: string) {
    error = null;
    try {
      await raisePayslipQuery(pid, comment);
      asking = null;
      comment = "";
      await load();
    } catch (cause) {
      error = errorText(cause);
    }
  }

  async function print(pid: string) {
    printing = pid;
    await tick();
    window.print();
    printing = null;
  }
</script>

<svelte:head><title>{t("titles.myPayslips")}</title></svelte:head>

<h1>{t("payslips.title")}</h1>
<p class="muted no-print">{t("payslips.hint")}</p>
{#if error}<ErrorNote testid="error" message={error} />{/if}

{#if data}
  <section class="panel no-print" data-testid="payslips-ytd">
    <h2>{t("payslips.ytd")}</h2>
    {#if data.year_to_date === null}
      <p class="muted" data-testid="payslips-ytd-none">{t("payslips.ytdNone")}</p>
    {:else}
      {#each data.year_to_date as year (yearKey(year))}
        <h3>{tf("payslips.ytdTaxYear", { range: formatDateRange(year.starts_on, year.ends_on) })}</h3>
        <p class="muted">{tf("payslips.ytdPayslips", { currency: year.currency, count: year.payslips })}</p>
        <dl data-testid="payslips-ytd-year">
          <dt>{t("pay.gross")}</dt>
          <dd>{money(year.gross_minor, year.currency, i18n.locale)}</dd>
          {#each year.deductions as line (line.label)}
            <dt>{line.label}</dt>
            <dd>{money(line.amount_minor, year.currency, i18n.locale)}</dd>
          {/each}
          <dt>{t("pay.net")}</dt>
          <dd>{money(year.net_minor, year.currency, i18n.locale)}</dd>
        </dl>
      {/each}
    {/if}
  </section>

  {#if data.payslips.length === 0}
    <p class="muted" data-testid="payslips-none">{t("payslips.none")}</p>
  {/if}
  {#each data.payslips as slip (slip.pid)}
    <section
      class="panel slip"
      class:elsewhere={printing !== null && printing !== slip.pid}
      data-testid="payslip"
    >
      <h2>{formatDateRange(slip.period_start, slip.period_end)}</h2>
      <dl>
        <dt>{t("pay.gross")}</dt>
        <dd>{money(slip.gross_minor, slip.currency, i18n.locale)}</dd>
        {#if slip.statutory_overtime_minor > 0}
          <dt>{t("pay.statutoryOvertime")}</dt>
          <dd>{money(slip.statutory_overtime_minor, slip.currency, i18n.locale)}</dd>
        {/if}
        {#each slip.deductions as line (line.label)}
          <dt>{line.label}</dt>
          <dd>{money(line.amount_minor, slip.currency, i18n.locale)}</dd>
        {/each}
        <dt>{t("pay.net")}</dt>
        <dd data-testid="payslip-net">{money(slip.net_minor, slip.currency, i18n.locale)}</dd>
      </dl>
      <div class="no-print">
        {#if slip.query?.open}
          <p data-testid="payslip-query">{tf("payslips.queryOpen", { day: formatDate(slip.query.raised_on) })}</p>
        {:else if slip.query?.closed_on}
          <p data-testid="payslip-query">{tf("payslips.queryClosed", { day: formatDate(slip.query.closed_on) })}</p>
        {/if}
        {#if asking === slip.pid}
          <form
            onsubmit={(event) => {
              event.preventDefault();
              void send(slip.pid);
            }}
          >
            <label>
              {t("payslips.queryComment")}
              <textarea bind:value={comment} maxlength="500" rows="3" required data-testid="payslip-query-comment"
              ></textarea>
            </label>
            <p class="muted">{t("payslips.queryHint")}</p>
            <button type="submit" data-testid="payslip-query-send">{t("payslips.querySend")}</button>
            <button type="button" onclick={() => (asking = null)}>{t("payslips.queryCancel")}</button>
          </form>
        {:else}
          <button type="button" onclick={() => print(slip.pid)} data-testid="payslip-print">{t("payslips.print")}</button>
          {#if !slip.query?.open}
            <button type="button" onclick={() => (asking = slip.pid)} data-testid="payslip-query-open">
              {t("payslips.query")}
            </button>
          {/if}
        {/if}
      </div>
    </section>
  {/each}
{/if}

<style>
  dl {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    gap: 0.25rem 1.5rem;
    max-width: 32rem;
  }
  dd {
    margin: 0;
    text-align: end;
    font-variant-numeric: tabular-nums;
  }
  textarea {
    display: block;
    width: 100%;
    max-width: 32rem;
  }
  /* The browser prints the one payslip chosen, with none of the page around it. */
  @media print {
    :global(nav.top),
    :global(.drawer),
    :global(.demo-banner),
    :global(.overlay-container),
    .no-print,
    .elsewhere {
      display: none;
    }
    .slip {
      border: 0;
      box-shadow: none;
    }
  }
</style>

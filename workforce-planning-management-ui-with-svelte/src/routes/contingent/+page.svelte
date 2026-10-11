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
    programmeRollup,
    type ContingentList,
    type ContingentWorkforce,
    type ProgrammeRollup,
    type Withheld,
  } from "#lib/api/wpm.js";
  import { t, tf, tv } from "#lib/i18n.svelte.js";

  let view = $state<ContingentWorkforce | null>(null);
  let rollup = $state<ProgrammeRollup | null>(null);
  let forbidden = $state(false);
  let error = $state<string | null>(null);
  let open = $state<ContingentList | null>(null);
  let people = $state<Awaited<ReturnType<typeof contingentList>>["people"] | null>(null);
  let listError = $state<string | null>(null);

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
      <div class="scroll wide">
        <table data-testid="contingent-departments">
          <thead>
            <tr>
              <th>{t("common.department")}</th><th>{t("contingent.people")}</th>
              <th>{t("contingent.contractors")}</th><th>{t("contingent.fixedTerm")}</th>
            </tr>
          </thead>
          <tbody>
            {#each view.by_department as d (d.department)}
              <tr>
                <td>{d.department}</td><td>{d.headcount ?? "—"}</td>
                <td>{pct(d.contractor_percent_of_fte)}</td><td>{pct(d.fixed_term_percent_of_fte)}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {:else}
      <p class="muted" data-testid="departments-withheld">{t("contingent.departmentsWithheld")}</p>
    {/if}
  </section>

  <section class="panel" data-testid="contingent-engagements">
    <h2>{t("contingent.engagements")}</h2>
    <div class="scroll">
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
          <div class="scroll wide">
            <table data-testid="contingent-who-table">
              <thead>
                <tr>
                  <th>{t("contingent.workerNumber")}</th><th>{t("common.department")}</th>
                  <th>{t("engagement.basis")}</th><th>{t("engagement.ends")}</th>
                </tr>
              </thead>
              <tbody>
                {#each people as p (p.worker_pid)}
                  <tr>
                    <td>{p.worker_number}</td><td>{p.department}</td>
                    <td>{tv(p.employment_type)}</td><td>{p.engagement_ends_on ?? "—"}</td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        {/if}
      </div>
    {/if}
    <p class="muted">{t("contingent.noTotal")}</p>
  </section>

  <section class="panel" data-testid="contingent-funding">
    <h2>{t("contingent.fundingByMonth")}</h2>
    <div class="scroll">
      <table>
        <thead>
          <tr><th>{t("contingent.month")}</th><th>{t("contingent.people")}</th><th>{t("contingent.fte")}</th></tr>
        </thead>
        <tbody>
          {#each view.funding_ends as m (m.month)}
            <tr>
              <td>{m.month === "already_ended" ? t("contingent.alreadyEnded") : m.month}</td>
              <td>{#if count(m.people) === null}<span class="muted">{t("contingent.withheld")}</span>{:else}{m.people}{/if}</td>
              <td>{fte(m.fte)}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  </section>

  <section class="panel" data-testid="contingent-trend">
    <h2>{t("contingent.trend")}</h2>
    {#if view.trend.status === "insufficient_history"}
      <p class="muted" data-testid="trend-insufficient">{t("contingent.trendInsufficient")}</p>
    {:else}
      <div class="scroll">
        <table data-testid="trend-table">
          <thead>
            <tr><th>{t("contingent.date")}</th><th>{t("contingent.contractors")}</th><th>{t("contingent.fixedTerm")}</th></tr>
          </thead>
          <tbody>
            {#each view.trend.points as p (p.as_of)}
              <tr><td>{p.as_of}</td><td>{pct(p.contractor_percent_of_fte)}</td><td>{pct(p.fixed_term_percent_of_fte)}</td></tr>
            {/each}
          </tbody>
        </table>
      </div>
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
        <div class="scroll wide">
          <table data-testid={`programme-${p.programme}`}>
            <thead>
              <tr>
                <th>{t("contingent.month")}</th><th>{t("contingent.fundedFte")}</th>
                <th>{t("contingent.contractors")}</th><th>{t("contingent.fixedTerm")}</th>
                <th>{t("contingent.engagementsEnding")}</th><th>{t("contingent.fundingEnding")}</th>
              </tr>
            </thead>
            <tbody>
              {#each p.months as m (m.month)}
                <tr>
                  <td>{m.month}</td><td>{fte(m.funded_fte)}</td>
                  <td>{pct(m.contractor_percent_of_fte)}</td><td>{pct(m.fixed_term_percent_of_fte)}</td>
                  <td>{#if m.engagements_ending === null}<span class="muted">{t("contingent.withheld")}</span>{:else}{m.engagements_ending}{/if}</td>
                  <td>{#if m.funding_ending === null}<span class="muted">{t("contingent.withheld")}</span>{:else}{m.funding_ending}{/if}</td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      {/each}
    </section>
  {/if}
{/if}

<style>
  .scroll {
    overflow-x: auto;
  }
  /* A table scrolls inside its wrapper on a phone instead of squeezing words apart. */
  .wide :global(table) {
    min-width: 40rem;
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

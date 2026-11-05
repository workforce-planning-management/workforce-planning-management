<!--
  Train or hire (WPM-R229): one internal candidate for one seat against an external hire, with both totals and every input. A
  missing input answers unknown and names it. An estimate for a person to weigh; WPM decides nothing.
-->
<script lang="ts">
  import { guardedLoad } from "#lib/loader.svelte.js";
  import { errorText } from "#lib/write.js";
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import { listWorkers, money, trainOrHire, type Position, type TrainOrHireView } from "#lib/api/wpm.js";
  import type { Worker } from "#lib/api/types.js";
  import { inputNames } from "#lib/economics.js";
  import { t, tf } from "#lib/i18n.svelte.js";

  let { positions, organization }: { positions: Position[]; organization: string } = $props();

  let workers = $state<Worker[]>([]);
  let seat = $state("");
  let candidate = $state("");
  let view = $state<TrainOrHireView | null>(null);
  let error = $state<string | null>(null);

  const candidates = $derived(workers.filter((w) => w.organization_ref === organization));

  const load = guardedLoad(
    () => listWorkers(),
    (found) => {
      workers = found;
    },
    (cause) => {
      error = errorText(cause);
    },
  );

  $effect(() => {
    void load();
  });

  async function compare(event: SubmitEvent) {
    event.preventDefault();
    error = null;
    view = null;
    try {
      view = await trainOrHire(seat, candidate);
    } catch (cause) {
      error = errorText(cause);
    }
  }
</script>

<section class="panel" data-testid="train-or-hire">
  <h2>{t("economics.trainOrHire")}</h2>
  <p class="muted">{t("economics.trainOrHireHint")}</p>
  {#if error}<ErrorNote testid="error" message={error} />{/if}
  <form onsubmit={compare}>
    <label>
      {t("economics.seat")}
      <select bind:value={seat} required data-testid="toh-seat">
        <option value="" disabled></option>
        {#each positions as position (position.pid)}<option value={position.pid}>{position.title}</option>{/each}
      </select>
    </label>
    <label>
      {t("economics.candidate")}
      <select bind:value={candidate} required data-testid="toh-candidate">
        <option value="" disabled></option>
        {#each candidates as worker (worker.pid)}<option value={worker.pid}>{worker.display_name}</option>{/each}
      </select>
    </label>
    <button type="submit" data-testid="toh-compare">{t("economics.compare")}</button>
  </form>

  {#if view}
    <p data-testid="toh-answer"><strong>{t(`economics.answer.${view.answer}`)}</strong></p>
    {#if view.missing.length > 0}
      <p class="muted" data-testid="toh-missing">{tf("economics.missing", { inputs: inputNames(view.missing) })}</p>
    {/if}
    {#if view.train && view.hire}
      <h3>{t("economics.train")}</h3>
      <dl data-testid="toh-train">
        <dt>{t("economics.total")}</dt>
        <dd>{money(view.train.total_minor, view.currency)}</dd>
        <dt>{t("economics.fees")}</dt>
        <dd>{money(view.train.fees_minor, view.currency)}</dd>
        <dt>{t("economics.hoursCost")}</dt>
        <dd>{money(view.train.hours_cost_minor, view.currency)}</dd>
        <dt>{t("economics.rampUp")}</dt>
        <dd>{money(view.train.ramp_up_minor, view.currency)}</dd>
      </dl>
      <h3>{t("economics.hire")}</h3>
      <dl data-testid="toh-hire">
        <dt>{t("economics.total")}</dt>
        <dd>{money(view.hire.total_minor, view.currency)}</dd>
        <dt>{t("economics.recruitment")}</dt>
        <dd>{money(view.hire.recruitment_minor, view.currency)}</dd>
        <dt>{t("economics.vacancy")}</dt>
        <dd>{money(view.hire.vacancy_minor, view.currency)}</dd>
        <dt>{t("economics.rampUp")}</dt>
        <dd>{money(view.hire.ramp_up_minor, view.currency)}</dd>
        <dt>{t("economics.payDifference")}</dt>
        <dd>{money(view.hire.pay_difference_minor, view.currency)}</dd>
      </dl>
    {/if}
  {/if}
</section>

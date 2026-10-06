<!--
  A worker's band and step on a pay scale (NHS Agenda for Change, Wales today).
  A band and step is a salary, so only the worker and HR can see or change it: the
  panel renders nothing for anyone else (the service refuses the read with 403).
  "Eligible" means the years on the step set by the circular — not that a move
  will happen.
-->
<script lang="ts">
  import {
    clearWorkerPayPosition,
    getPayScale,
    getWorkerPayPosition,
    listPayScales,
    moneyWhole,
    setWorkerPayPosition,
  } from "#lib/api/wpm.js";
  import { ApiError } from "#lib/api/client.js";
  import type { PayScale, WorkerPayPosition } from "#lib/api/types.js";
  import { t, tp, i18n } from "#lib/i18n.svelte.js";

  let { workerPid }: { workerPid: string } = $props();

  let held = $state<WorkerPayPosition | null>(null);
  let scale = $state<PayScale | null>(null);
  let allowed = $state(true);
  let error = $state<string | null>(null);
  let band = $state("");
  let step = $state("1");
  let since = $state("");

  const message = (cause: unknown) => (cause instanceof Error ? cause.message : String(cause));
  const steps = $derived(scale?.bands.find((b) => b.code === band)?.steps.length ?? 0);

  async function load() {
    try {
      held = (await getWorkerPayPosition(workerPid)).pay_position;
      allowed = true;
      const [first] = await listPayScales();
      scale = first ? await getPayScale(first.id) : null;
    } catch (cause) {
      // Not the worker or HR: not an error, just not theirs to see.
      if (cause instanceof ApiError && (cause.status === 403 || cause.status === 401)) {
        allowed = false;
      } else {
        error = message(cause);
      }
    }
  }

  $effect(() => {
    void workerPid;
    void load();
  });

  async function run(action: () => Promise<unknown>) {
    error = null;
    try {
      await action();
      await load();
    } catch (cause) {
      error = message(cause);
    }
  }

  function save(event: SubmitEvent) {
    event.preventDefault();
    if (!scale || !band) return;
    const scaleId = scale.id;
    void run(() =>
      setWorkerPayPosition(workerPid, {
        scale: scaleId,
        band,
        step: Number(step),
        ...(since && { step_since: since }),
      }),
    );
  }

  const amount = (minor: number | null | undefined, currency: string | null | undefined) =>
    moneyWhole(minor, currency, i18n.locale);
</script>

{#if allowed}
  <section class="panel" data-testid="pay-position">
    <h2>{t("payPosition.title")}</h2>
    <p class="muted">{t("payPosition.hint")}</p>
    {#if error}<p class="error" data-testid="error">{error}</p>{/if}
    {#if held}
      <p data-testid="pay-position-held">
        {tp("payPosition.current", {
          band: held.band,
          step: held.step,
          amount: amount(held.annual_minor, held.currency),
          since: held.step_since,
        })}
      </p>
      {#if held.progression}
        <p data-testid="pay-position-progression">
          {#if held.progression.kind === "at_top"}
            ● {t("payScales.progression.at_top")}
          {:else if held.progression.kind === "due"}
            ✓ {tp("payPosition.due", {
              date: held.progression.eligible_on,
              amount: amount(held.progression.next_annual_minor, held.currency),
            })}
          {:else}
            ● {tp("payPosition.notYet", {
              date: held.progression.eligible_on,
              days: held.progression.days_remaining,
              amount: amount(held.progression.next_annual_minor, held.currency),
            })}
          {/if}
        </p>
      {/if}
      <button data-testid="pay-position-clear" onclick={() => run(() => clearWorkerPayPosition(workerPid))}>
        {t("grades.clear")}
      </button>
    {:else}
      <p class="muted" data-testid="pay-position-none">{t("payPosition.none")}</p>
    {/if}
    {#if scale}
      <form class="cx-form" onsubmit={save} data-testid="pay-position-form">
        <label>
          {t("payScales.band")}
          <select data-testid="pay-position-band" bind:value={band}>
            <option value="">—</option>
            {#each scale.bands as b (b.code)}<option value={b.code}>{b.code}</option>{/each}
          </select>
        </label>
        <label>
          {t("payScales.step")}
          <select data-testid="pay-position-step" bind:value={step}>
            {#each Array.from({ length: Math.max(steps, 1) }, (_, i) => i + 1) as n (n)}
              <option value={String(n)}>{n}</option>
            {/each}
          </select>
        </label>
        <label>
          {t("grades.since")}
          <input type="date" data-testid="pay-position-since" bind:value={since} />
        </label>
        <button type="submit" data-testid="pay-position-save">{t("grades.save")}</button>
      </form>
    {/if}
  </section>
{/if}

<style>
  .cx-form {
    display: flex;
    flex-wrap: wrap;
    gap: 0.75rem 1rem;
    align-items: flex-end;
    margin-top: 0.75rem;
  }
  .cx-form label {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    min-width: 0;
    max-width: 100%;
  }
</style>

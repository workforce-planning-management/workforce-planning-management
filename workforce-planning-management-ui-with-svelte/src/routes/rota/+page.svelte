<!--
  On-call rota (`/rota`): a rotation of workers where the duty passes to
  the next member every few calendar days. Shows who is on call now, the next four
  weeks as stretches (with why: their turn, covering for someone away, or a
  swap), and how many days each member carries. A member on approved leave
  is skipped to the next available person; a day nobody can take is shown
  as unavailable, never guessed. HR creates rotas and records swaps.
-->
<script lang="ts">
  import { page } from "$app/state";
  import {
    addRotaSwap,
    createRota,
    getRota,
    listRotas,
    listWorkers,
    decideSwap,
    listSwapRequests,
    removeRotaSwap,
    requestSwap,
    retireRota,
  } from "#lib/api/wpm.js";
  import type { RotaSummary, RotaView, SwapRequest } from "#lib/api/types.js";
  import { t } from "#lib/i18n.svelte.js";

  const organizationRefs = $derived((page.data.scope ?? []) as string[]);

  let rotas = $state<RotaSummary[] | null>(null);
  let chosen = $state("");
  let view = $state<RotaView | null>(null);
  let everyone = $state<Array<{ pid: string; display_name: string; organization_ref: string }>>([]);
  let error = $state<string | null>(null);

  // New rota form.
  let name = $state("");
  let period = $state(7);
  let startsOn = $state(new Date().toISOString().slice(0, 10));
  let organization = $state("");
  let picked = $state<string[]>([]);
  let requests = $state<SwapRequest[]>([]);
  // New swap-request form.
  let askRequester = $state("");
  let askTaker = $state("");
  let askFrom = $state("");
  let askUntil = $state("");
  let askNote = $state("");
  // New swap form.
  let swapWorker = $state("");
  let swapFrom = $state("");
  let swapUntil = $state("");
  let swapNote = $state("");

  const message = (cause: unknown) => (cause instanceof Error ? cause.message : String(cause));
  const org = $derived(organization || organizationRefs[0] || "");
  const orgWorkers = $derived(everyone.filter((w) => w.organization_ref === org));
  // Colleagues of the rota being viewed (its own organization, not the create form's).
  const viewWorkers = $derived(everyone.filter((w) => w.organization_ref === view?.organization_ref));
  const nameOf = (pid: string) => everyone.find((w) => w.pid === pid)?.display_name ?? pid;

  async function loadList() {
    try {
      const [list, workers] = await Promise.all([listRotas(), listWorkers()]);
      rotas = list;
      everyone = workers;
      if (!chosen || !list.some((r) => r.pid === chosen)) chosen = list[0]?.pid ?? "";
    } catch (cause) {
      error = message(cause);
    }
  }

  async function loadView() {
    view = null;
    if (!chosen) return;
    try {
      [view, requests] = await Promise.all([getRota(chosen), listSwapRequests(chosen)]);
    } catch (cause) {
      error = message(cause);
    }
  }

  $effect(() => {
    void loadList();
  });
  $effect(() => {
    void chosen;
    void loadView();
  });

  async function run(action: () => Promise<unknown>, reloadList = false) {
    error = null;
    try {
      await action();
      if (reloadList) await loadList();
      await loadView();
    } catch (cause) {
      error = message(cause);
    }
  }

  const sourceLabel = (source: string | null) =>
    source === "rotation"
      ? t("rota.sourceRotation")
      : source === "skipped"
        ? t("rota.sourceSkipped")
        : source === "override"
          ? t("rota.sourceOverride")
          : "—";
</script>

<svelte:head><title>{t("nav.rota")} — WPM</title></svelte:head>

<h1>{t("nav.rota")}</h1>
{#if error}<p class="error" data-testid="error">{t("common.error")}: {error}</p>{/if}

{#if rotas === null}
  <p>{t("common.loading")}</p>
{:else}
  {#if rotas.length === 0}
    <p class="muted" data-testid="rota-none">{t("rota.none")}</p>
  {:else}
    <p>
      <select bind:value={chosen} data-testid="rota-choice" aria-label={t("nav.rota")}>
        {#each rotas as r (r.pid)}<option value={r.pid}>{r.name}</option>{/each}
      </select>
    </p>
  {/if}

  {#if view}
    <p data-testid="on-call-now">
      {t("rota.onCallNow")}:
      {#if view.on_call_today}
        <strong>{view.on_call_today.name ?? view.on_call_today.worker_pid}</strong>
      {:else}
        <span class="muted">{t("rota.nobody")}</span>
      {/if}
    </p>

    <h2>{view.name}</h2>
    <table data-testid="rota-runs">
      <thead>
        <tr><th>{t("rota.from")}</th><th>{t("rota.to")}</th><th>{t("rota.who")}</th><th>{t("rota.why")}</th></tr>
      </thead>
      <tbody>
        {#each view.runs as run (run.from)}
          <tr>
            <td>{run.from}</td>
            <td>{run.to}</td>
            <td>{run.worker_name ?? (run.worker_pid ? run.worker_pid : t("rota.nobody"))}</td>
            <td class="muted">{sourceLabel(run.source)}</td>
          </tr>
        {/each}
      </tbody>
    </table>

    <h3>{t("rota.load")}</h3>
    <ul data-testid="rota-load">
      {#each view.load as l (l.worker_pid)}<li>{l.name ?? l.worker_pid}: {l.days}</li>{/each}
    </ul>

    <h3>{t("rota.members")}</h3>
    <ol data-testid="rota-members">
      {#each view.members as m (m.worker_pid)}
        <li>{m.name ?? m.worker_pid}{#if m.job_title} <span class="muted">— {m.job_title}</span>{/if}</li>
      {/each}
    </ol>

    <h3>{t("rota.swaps")}</h3>
    <ul data-testid="rota-swaps">
      {#each view.overrides as o (o.pid)}
        <li>
          {o.worker_name ?? o.worker_pid} · {o.starts_on} → {o.ends_on}
          {#if o.note}<span class="muted">— {o.note}</span>{/if}
          <button type="button" onclick={() => void run(() => removeRotaSwap(o.pid))}>
            {t("backups.remove")}
          </button>
        </li>
      {/each}
    </ul>
    <form
      data-testid="swap-form"
      onsubmit={(event) => {
        event.preventDefault();
        const id = view?.pid;
        if (!id) return;
        void run(async () => {
          await addRotaSwap(id, {
            worker_pid: swapWorker,
            starts_on: swapFrom,
            ends_on: swapUntil,
            ...(swapNote.trim() ? { note: swapNote.trim() } : {}),
          });
          swapWorker = swapFrom = swapUntil = swapNote = "";
        });
      }}
    >
      <label>
        {t("rota.addSwap")}
        <select bind:value={swapWorker} required data-testid="swap-worker">
          <option value="" disabled>…</option>
          {#each view.members as m (m.worker_pid)}<option value={m.worker_pid}>{m.name ?? m.worker_pid}</option>{/each}
        </select>
      </label>
      <label>{t("rota.from")} <input type="date" bind:value={swapFrom} required /></label>
      <label>{t("rota.to")} <input type="date" bind:value={swapUntil} required /></label>
      <label>{t("backups.note")} <input bind:value={swapNote} maxlength="500" /></label>
      <button type="submit" data-testid="swap-add">{t("rota.addSwap")}</button>
    </form>
    <h3>{t("rota.swapTitle")}</h3>
    <ul data-testid="rota-requests">
      {#each requests as r (r.pid)}
        <li>
          {r.requester_name ?? r.requester_pid} → {r.taker_name ?? r.taker_pid}
          · {r.starts_on} → {r.ends_on} · <span class="chip">{r.status}</span>
          {#if r.status === "requested"}
            <button type="button" onclick={() => void run(() => decideSwap(r.pid, "accept"))}>{t("rota.accept")}</button>
            <button type="button" onclick={() => void run(() => decideSwap(r.pid, "decline"))}>{t("rota.decline")}</button>
            <button type="button" onclick={() => void run(() => decideSwap(r.pid, "cancel"))}>{t("rota.cancel")}</button>
          {/if}
        </li>
      {:else}
        <li class="muted">{t("rota.swapNone")}</li>
      {/each}
    </ul>
    <form
      data-testid="swap-request-form"
      onsubmit={(event) => {
        event.preventDefault();
        const id = view?.pid;
        if (!id) return;
        void run(async () => {
          await requestSwap(id, {
            requester_pid: askRequester,
            taker_pid: askTaker,
            starts_on: askFrom,
            ends_on: askUntil,
            ...(askNote.trim() ? { note: askNote.trim() } : {}),
          });
          askRequester = askTaker = askFrom = askUntil = askNote = "";
        });
      }}
    >
      <label>
        {t("rota.who")}
        <select bind:value={askRequester} required data-testid="ask-requester">
          <option value="" disabled>…</option>
          {#each view.members as m (m.worker_pid)}<option value={m.worker_pid}>{m.name ?? m.worker_pid}</option>{/each}
        </select>
      </label>
      <label>
        {t("rota.swapTaker")}
        <select bind:value={askTaker} required data-testid="ask-taker">
          <option value="" disabled>…</option>
          {#each viewWorkers.filter((w) => w.pid !== askRequester) as w (w.pid)}<option value={w.pid}>{w.display_name}</option>{/each}
        </select>
      </label>
      <label>{t("rota.from")} <input type="date" bind:value={askFrom} required /></label>
      <label>{t("rota.to")} <input type="date" bind:value={askUntil} required /></label>
      <label>{t("backups.note")} <input bind:value={askNote} maxlength="500" /></label>
      <button type="submit" data-testid="swap-request-add">{t("rota.swapRequest")}</button>
    </form>
    <p>
      <button
        type="button"
        onclick={() => {
          const id = view?.pid;
          if (id) void run(() => retireRota(id), true);
        }}
      >
        {t("rota.retire")}
      </button>
    </p>
  {/if}

  <details>
    <summary>{t("rota.create")}</summary>
    <form
      data-testid="rota-create"
      onsubmit={(event) => {
        event.preventDefault();
        void run(async () => {
          const made = await createRota({
            organization_ref: org,
            name: name.trim(),
            period_days: period,
            starts_on: startsOn,
            members: picked,
          });
          name = "";
          picked = [];
          chosen = made.pid;
        }, true);
      }}
    >
      <label>{t("common.name")} <input bind:value={name} required maxlength="200" data-testid="rota-name" /></label>
      <label>
        {t("rota.organization")}
        <select bind:value={organization}>
          {#each organizationRefs as ref (ref)}<option value={ref}>{ref}</option>{/each}
        </select>
      </label>
      <label>{t("rota.period")} <input type="number" min="1" max="31" bind:value={period} required /></label>
      <label>{t("rota.startsOn")} <input type="date" bind:value={startsOn} required /></label>
      <fieldset>
        <legend>{t("rota.members")}</legend>
        {#each orgWorkers as w (w.pid)}
          <label>
            <input type="checkbox" value={w.pid} bind:group={picked} />
            {w.display_name}
          </label>
        {/each}
        {#if picked.length > 0}
          <p class="muted">{picked.map(nameOf).join(" → ")}</p>
        {/if}
      </fieldset>
      <button type="submit" data-testid="rota-save">{t("rota.save")}</button>
    </form>
  </details>
{/if}

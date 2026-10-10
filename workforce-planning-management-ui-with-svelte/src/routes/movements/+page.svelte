<!--
  Joiners and leavers (`/movements`): who is joining and who is leaving, each
  with a dated checklist and how far along it is — late items are counted.
  Open a record for someone (a leaver needs a last day and a reason); open a
  record to work its checklist and, for a leaver, hand over what they hold.
-->
<script lang="ts">
  import { goto } from "$app/navigation";
  import { listMovements, listWorkers, openMovement } from "#lib/api/wpm.js";
  import type { Movement } from "#lib/api/types.js";
  import { t, l, tf } from "#lib/i18n.svelte.js";

  const REASONS = [
    ["resignation", "movements.reasonResignation"],
    ["redundancy", "movements.reasonRedundancy"],
    ["retirement", "movements.reasonRetirement"],
    ["end_of_contract", "movements.reasonEndOfContract"],
    ["dismissal", "movements.reasonDismissal"],
    ["other", "movements.reasonOther"],
  ] as const;

  let rows = $state<Movement[] | null>(null);
  let people = $state<Array<{ pid: string; display_name: string }>>([]);
  let error = $state<string | null>(null);
  let kind = $state<"joiner" | "leaver">("leaver");
  let person = $state("");
  let date = $state("");
  let reason = $state("resignation");

  const message = (cause: unknown) => (cause instanceof Error ? cause.message : String(cause));

  async function load() {
    try {
      [rows, people] = await Promise.all([listMovements(), listWorkers()]);
    } catch (cause) {
      error = message(cause);
    }
  }
  $effect(() => {
    void load();
  });

  const joiners = $derived((rows ?? []).filter((m) => m.kind === "joiner"));
  const leavers = $derived((rows ?? []).filter((m) => m.kind === "leaver"));

  async function start() {
    error = null;
    try {
      const made = await openMovement(person, {
        kind,
        ...(date ? { effective_on: date } : {}),
        ...(kind === "leaver" ? { reason } : {}),
      });
      await goto(l(`/movements/${made.pid}`));
    } catch (cause) {
      error = message(cause);
    }
  }
</script>

<svelte:head><title>{tf("pages.movements.page_title", { movements: t("nav.movements") })}</title></svelte:head>

<h1>{t("nav.movements")}</h1>
{#if error}<p class="error" data-testid="error">{t("common.error")}: {error}</p>{/if}

{#snippet table(list: Movement[], testid: string, dateLabel: string)}
  {#if list.length === 0}
    <p class="muted">{t("movements.none")}</p>
  {:else}
    <table data-testid={testid}>
      <thead>
        <tr>
          <th>{t("movements.person")}</th>
          <th>{dateLabel}</th>
          <th>{t("movements.progress")}</th>
        </tr>
      </thead>
      <tbody>
        {#each list as m (m.pid)}
          <tr>
            <td>
              <a href={l(`/movements/${m.pid}`)}>{m.worker_name}</a>
              <span class="muted">· {m.job_title}, {m.department}</span>
            </td>
            <td>{m.effective_on}</td>
            <td>
              {m.progress.closed}/{m.progress.total}
              {#if m.progress.overdue > 0}
                <span class="chip" data-testid="overdue-chip">▲ {m.progress.overdue} {t("movements.overdue")}</span>
              {/if}
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}
{/snippet}

{#if rows === null}
  {#if !error}<p>{t("common.loading")}</p>{/if}
{:else}
  <h2>{t("movements.leavers")}</h2>
  {@render table(leavers, "leavers-table", t("movements.lastDay"))}
  <h2>{t("movements.joiners")}</h2>
  {@render table(joiners, "joiners-table", t("movements.startDay"))}
{/if}

<details>
  <summary>{t("movements.start")}</summary>
  <form
    data-testid="movement-form"
    onsubmit={(event) => {
      event.preventDefault();
      void start();
    }}
  >
    <label>
      {t("movements.person")}
      <select bind:value={person} required data-testid="movement-person">
        <option value="" disabled>…</option>
        {#each people as p (p.pid)}<option value={p.pid}>{p.display_name}</option>{/each}
      </select>
    </label>
    <label>
      <select bind:value={kind} data-testid="movement-kind" aria-label={t("movements.start")}>
        <option value="leaver">{t("movements.kindLeaver")}</option>
        <option value="joiner">{t("movements.kindJoiner")}</option>
      </select>
    </label>
    <label>
      {kind === "leaver" ? t("movements.lastDay") : t("movements.startDay")}
      <input type="date" bind:value={date} required={kind === "leaver"} data-testid="movement-date" />
    </label>
    {#if kind === "leaver"}
      <label>
        {t("movements.reason")}
        <select bind:value={reason} data-testid="movement-reason">
          {#each REASONS as [value, key] (value)}<option {value}>{t(key)}</option>{/each}
        </select>
      </label>
    {/if}
    <button type="submit" data-testid="movement-start">{t("movements.start")}</button>
  </form>
</details>

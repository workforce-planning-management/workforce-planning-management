<!--
  My personal development reviews (WPM-R172): when the reader's next is due, their own, and everyone below them by name with their
  due date, where a manager opens one. No content here. An unknown due date stays unknown.
-->
<script lang="ts">
  import { goto } from "$app/navigation";
  import { page } from "$app/state";
  import { guardedLoad } from "#lib/loader.svelte.js";
  import { errorText } from "#lib/write.js";
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import { myPadrs, openPadr, type MyPadrs, type PadrDue, type PadrLine } from "#lib/api/wpm.js";
  import type { MyOrganization } from "#lib/api/types.js";
  import { formatDate, formatDateRange } from "#lib/locale-format.js";
  import { standingWord } from "#lib/padr.js";
  import { l, t, tv } from "#lib/i18n.svelte.js";

  const me = $derived(
    ((page.data.organizations ?? []) as MyOrganization[]).find((m) => m.worker_pid !== null)?.worker_pid ?? null,
  );

  let view = $state<MyPadrs | null>(null);
  let error = $state<string | null>(null);
  let periods = $state<Record<string, { start: string; end: string }>>({});

  const load = guardedLoad(
    () => myPadrs(),
    (found) => {
      view = found;
      periods = Object.fromEntries(
        found.team.map((person) => [person.worker_pid, periods[person.worker_pid] ?? { start: "", end: "" }]),
      );
    },
    (cause) => {
      error = errorText(cause);
    },
  );

  $effect(() => {
    if (me) void load();
  });

  const isOpen = (line: PadrLine) => line.status === "draft";

  async function open(event: SubmitEvent, worker: string) {
    event.preventDefault();
    error = null;
    const period = periods[worker];
    if (!period) return;
    try {
      const made = await openPadr(worker, { period_start: period.start, period_end: period.end });
      await goto(l(`/me/padr/${made.pid}`));
    } catch (cause) {
      error = errorText(cause);
    }
  }
</script>

{#snippet due(row: PadrDue)}
  <span data-testid="padr-standing">{standingWord(row.standing)}</span>
  {#if row.due_on}<span class="muted">· {formatDate(row.due_on)}</span>{/if}
{/snippet}

{#snippet lines(rows: PadrLine[])}
  <ul>
    {#each rows as row (row.pid)}
      <li data-testid="padr-line">
        <a href={l(`/me/padr/${row.pid}`)}>
          {row.period_start && row.period_end ? formatDateRange(row.period_start, row.period_end) : t("padr.title")}
        </a>
        <span class="muted">· {row.completed_on ? t("padr.complete") : tv(row.status)}</span>
      </li>
    {/each}
  </ul>
{/snippet}

<svelte:head><title>{t("titles.myPadrs")}</title></svelte:head>

<h1>{t("padr.title")}</h1>
<p class="muted">{t("padr.hint")}</p>

{#if error}<ErrorNote testid="error" prefixed message={error} />{/if}

{#if !me}
  <p class="muted" data-testid="padr-no-worker">{t("kanban.noWorker")}</p>
{:else if view === null}
  <p>{t("common.loading")}</p>
{:else}
  <section class="panel" data-testid="padr-mine">
    <h2>{t("padr.mine")}</h2>
    <p>{t("padr.nextDue")}: {@render due(view.due)}</p>
    {#if view.padrs.length > 0}
      {@render lines(view.padrs)}
    {:else}
      <p class="muted">{t("padr.none")}</p>
    {/if}
  </section>

  {#if view.team.length > 0}
    <h2>{t("goals.team")}</h2>
    {#each view.team as person (person.worker_pid)}
      <section class="panel" data-testid="padr-person">
        <h3>{person.display_name}</h3>
        <p>{t("padr.nextDue")}: {@render due(person.due)}</p>
        {@render lines(person.padrs)}
        {#if !person.padrs.some(isOpen) && periods[person.worker_pid]}
          <form onsubmit={(event) => void open(event, person.worker_pid)}>
            <label>
              {t("padr.periodStart")}
              <input type="date" bind:value={periods[person.worker_pid]!.start} required data-testid="padr-open-start" />
            </label>
            <label>
              {t("padr.periodEnd")}
              <input type="date" bind:value={periods[person.worker_pid]!.end} required data-testid="padr-open-end" />
            </label>
            <button type="submit" data-testid="padr-open">{t("padr.open")}</button>
          </form>
        {/if}
      </section>
    {/each}
  {/if}
{/if}

<style>
  form label {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    margin-bottom: 0.75rem;
    min-width: 0;
    max-width: 100%;
  }
  form input {
    max-width: 100%;
    box-sizing: border-box;
  }
</style>

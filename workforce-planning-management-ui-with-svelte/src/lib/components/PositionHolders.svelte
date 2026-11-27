<!--
  One seat's holder history (WPM-R290): each tenure with its start and end, and the seat's continuity over 36 months. The
  service shows it to HR and other privileged readers only; anyone else gets a 403 and the panel renders nothing. A tenure
  open when the history began has an unknown start; an erased holder keeps the dates and is named nowhere.
-->
<script lang="ts">
  import { formatDate, formatInteger } from "#lib/locale-format.js";
  import { guardedLoad } from "#lib/loader.svelte.js";
  import { errorText } from "#lib/write.js";
  import { ApiError } from "#lib/api/client.js";
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import { positionHolders } from "#lib/api/wpm.js";
  import { t, i18n } from "#lib/i18n.svelte.js";

  let { pid, title }: { pid: string; title: string } = $props();

  type History = Awaited<ReturnType<typeof positionHolders>>;
  let history = $state<History | null>(null);
  let allowed = $state(true);
  let error = $state<string | null>(null);

  const load = guardedLoad(
    () => positionHolders(pid),
    (found) => {
      history = found;
      allowed = true;
      error = null;
    },
    (cause) => {
      // Not someone who may read who held a post: not an error, just not theirs to see.
      if (cause instanceof ApiError && (cause.status === 403 || cause.status === 401)) {
        allowed = false;
      } else {
        error = errorText(cause);
      }
    },
  );
  $effect(() => {
    void pid;
    void load();
  });

  const days = (n: number | null) => (n === null ? t("pages.positions.unknown") : formatInteger(n, i18n.locale));
</script>

{#if allowed}
  <section data-testid="position-holders">
    <h3>{t("pages.positions.holders")}: {title}</h3>
    <p class="muted">{t("pages.positions.holders_note")}</p>
    {#if error}<ErrorNote testid="position-holders-error" message={error} />{/if}
    {#if history}
      {#if history.holders.length > 0}
        <table data-testid="position-holders-table">
          <thead>
            <tr>
              <th scope="col">{t("pages.positions.holder")}</th>
              <th scope="col">{t("pages.positions.started")}</th>
              <th scope="col">{t("pages.positions.ended")}</th>
            </tr>
          </thead>
          <tbody>
            {#each history.holders as tenure, i (i)}
              <tr>
                <td>{tenure.erased ? t("pages.positions.erased") : tenure.worker_pid}</td>
                <td>{tenure.start_known ? formatDate(tenure.started_on, "medium", i18n.locale) : t("pages.positions.start_unknown")}</td>
                <td>{tenure.ended_on ? formatDate(tenure.ended_on, "medium", i18n.locale) : t("pages.positions.still_holds")}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      {/if}
      <dl class="cx-continuity" data-testid="position-continuity">
        <dt>{t("pages.positions.tenures")}</dt>
        <dd>{formatInteger(history.continuity.tenures, i18n.locale)}</dd>
        <dt>{t("pages.positions.days_held")}</dt>
        <dd>{days(history.continuity.days_held)}</dd>
        <dt>{t("pages.positions.days_empty")}</dt>
        <dd>{days(history.continuity.days_empty)}</dd>
        <dt>{t("pages.positions.counted_from")}</dt>
        <dd>{formatDate(history.continuity.counted_from, "medium", i18n.locale)}</dd>
      </dl>
      {#if !history.continuity.complete}<p class="muted" data-testid="position-continuity-partial">● {t("pages.positions.window_partial")}</p>{/if}
    {/if}
  </section>
{/if}

<style>
  .cx-continuity {
    display: grid;
    grid-template-columns: max-content auto;
    gap: 0.25em 1em;
  }
  .cx-continuity dd {
    margin: 0;
  }
</style>

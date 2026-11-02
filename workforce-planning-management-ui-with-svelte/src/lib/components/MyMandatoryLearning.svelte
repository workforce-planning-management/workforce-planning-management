<!--
  My statutory and mandatory learning: the person's own status against each requirement in force, with the last
  completion and the day it expires. Unknown says so in words (a record that does not say which group you are in);
  it is never shown as in date or as missing (WPM-D117).
-->
<script lang="ts">
  import { formatDate } from "#lib/locale-format.js";
  import { guardedLoad } from "#lib/loader.svelte.js";
  import { errorText } from "#lib/write.js";
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import { workerMandatoryLearning } from "#lib/api/wpm.js";
  import type { MyLearningRow } from "#lib/api/types.js";
  import { t } from "#lib/i18n.svelte.js";

  let { workerPid }: { workerPid: string } = $props();

  let rows = $state<MyLearningRow[] | null>(null);
  let error = $state<string | null>(null);

  const load = guardedLoad(
    () => workerMandatoryLearning(workerPid),
    (answer) => {
      rows = answer.requirements;
    },
    (cause) => {
      error = errorText(cause);
    },
  );

  $effect(() => {
    void workerPid;
    void load();
  });

  // A requirement that does not apply to the person is not listed.
  const shown = $derived((rows ?? []).filter((r) => r.status !== "not_applicable"));
</script>

<section class="panel" data-testid="my-learning">
  <h2>{t("mandatoryLearning.mine")}</h2>
  {#if error}
    <ErrorNote testid="my-learning-error" message={error} />
  {:else if rows === null}
    <p>{t("common.loading")}</p>
  {:else if shown.length === 0}
    <p>{t("mandatoryLearning.none")}</p>
  {:else}
    <ul>
      {#each shown as row (row.requirement_pid)}
        <li data-testid={`learning-${row.status}`}>
          <strong>{row.subject}</strong>
          <span class="chip">{t(`mandatoryLearning.${row.status}`)}</span>
          {#if row.last_completed_on}
            <span class="muted"
              >{t("mandatoryLearning.lastDone")}: {formatDate(row.last_completed_on)}</span
            >
          {/if}
          {#if row.expires_on}
            <span class="muted">{t("mandatoryLearning.expires")}: {formatDate(row.expires_on)}</span>
          {/if}
          {#if row.status === "unknown"}
            <span class="muted">{t("mandatoryLearning.unknownNote")}</span>
          {/if}
        </li>
      {/each}
    </ul>
  {/if}
</section>

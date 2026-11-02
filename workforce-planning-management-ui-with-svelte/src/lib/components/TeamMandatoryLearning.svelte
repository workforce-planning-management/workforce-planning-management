<!--
  My team's statutory and mandatory learning: for a manager, each person below them by name (a work record, not a
  sensitive one) who is due soon, expired, never completed or unknown on a requirement. Unknown is listed as unknown,
  never as in date. Renders nothing for anyone with nobody below them.
-->
<script lang="ts">
  import { formatDate } from "#lib/locale-format.js";
  import { guardedLoad } from "#lib/loader.svelte.js";
  import { errorText } from "#lib/write.js";
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import { teamMandatoryLearning } from "#lib/api/wpm.js";
  import type { TeamLearningPerson } from "#lib/api/types.js";
  import { l, t } from "#lib/i18n.svelte.js";

  let { workerPid }: { workerPid: string } = $props();

  let people = $state<TeamLearningPerson[] | null>(null);
  let error = $state<string | null>(null);

  const load = guardedLoad(
    () => teamMandatoryLearning(workerPid),
    (answer) => {
      people = answer.people;
    },
    (cause) => {
      error = errorText(cause);
    },
  );

  $effect(() => {
    void workerPid;
    void load();
  });

  const NEEDS = new Set(["due_soon", "expired", "never_completed", "unknown"]);
  const attention = $derived(
    (people ?? [])
      .map((p) => ({ ...p, requirements: p.requirements.filter((r) => NEEDS.has(r.status)) }))
      .filter((p) => p.requirements.length > 0),
  );
</script>

{#if error}
  <ErrorNote testid="team-learning-error" message={error} />
{:else if people && people.length > 0}
  <section class="panel" data-testid="team-learning">
    <h2>{t("mandatoryLearning.team")}</h2>
    {#if attention.length === 0}
      <p>{t("mandatoryLearning.teamNone")}</p>
    {:else}
      <ul>
        {#each attention as person (person.worker_pid)}
          <li data-testid="team-learning-person">
            <a href={l(`/workers/${person.worker_pid}`)}>{person.display_name}</a>
            {#each person.requirements as r (r.requirement_pid)}
              <span class="chip" data-testid={`team-learning-${r.status}`}
                >{r.subject}: {t(`mandatoryLearning.${r.status}`)}{r.expires_on
                  ? ` (${formatDate(r.expires_on)})`
                  : ""}</span
              >
            {/each}
          </li>
        {/each}
      </ul>
    {/if}
  </section>
{/if}

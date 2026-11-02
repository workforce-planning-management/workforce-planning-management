<!--
  My team's qualifications: for a manager, each person below them by name (a work record) whose role asks for a
  qualification they do not hold in date and verified. Declared-only is listed as declared, never as met. A person with
  no role recorded is not listed as clean; they are simply not compared. Renders nothing for anyone with nobody below them.
-->
<script lang="ts">
  import { guardedLoad } from "#lib/loader.svelte.js";
  import { errorText } from "#lib/write.js";
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import { teamQualifications } from "#lib/api/wpm.js";
  import type { TeamQualificationPerson } from "#lib/api/types.js";
  import { l, t } from "#lib/i18n.svelte.js";

  let { workerPid }: { workerPid: string } = $props();

  let people = $state<TeamQualificationPerson[] | null>(null);
  let error = $state<string | null>(null);

  const load = guardedLoad(
    () => teamQualifications(workerPid),
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

  const LABEL = {
    declared_only: "qualifications.standingDeclared",
    expired: "qualifications.standingExpired",
    missing: "qualifications.standingMissing",
  } as const;

  const attention = $derived(
    (people ?? [])
      .map((p) => ({ ...p, gap: p.gap.filter((g) => g.standing !== "met") }))
      .filter((p) => p.gap.length > 0),
  );
</script>

{#if error}
  <ErrorNote testid="team-qualifications-error" message={error} />
{:else if people && people.length > 0}
  <section class="panel" data-testid="team-qualifications">
    <h2>{t("qualifications.team")}</h2>
    {#if attention.length === 0}
      <p>{t("qualifications.teamNone")}</p>
    {:else}
      <ul>
        {#each attention as person (person.worker_pid)}
          <li data-testid="team-qualifications-person">
            <a href={l(`/workers/${person.worker_pid}`)}>{person.display_name}</a>
            {#each person.gap as g (g.title)}
              <span class="chip" data-testid={`team-qualifications-${g.standing}`}
                >{g.title}: {t(LABEL[g.standing as keyof typeof LABEL])}</span
              >
            {/each}
          </li>
        {/each}
      </ul>
    {/if}
  </section>
{/if}

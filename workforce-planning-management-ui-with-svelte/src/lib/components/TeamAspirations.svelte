<!--
  For a manager: everyone below them — direct reports and indirect reports —
  with the aspirations each has chosen to share with their managers. A person's
  private aspirations never appear here, and neither does how many there are.
-->
<script lang="ts">
  import { downlineAspirations } from "#lib/api/wpm.js";

  let { workerPid }: { workerPid: string } = $props();

  type Team = Awaited<ReturnType<typeof downlineAspirations>>["team"];
  let team = $state<Team>([]);
  let error = $state<string | null>(null);

  $effect(() => {
    void workerPid;
    void downlineAspirations(workerPid)
      .then((r) => (team = r.team))
      .catch((cause) => (error = cause instanceof Error ? cause.message : String(cause)));
  });

  const direct = $derived(team.filter((p) => p.direct_report).length);
  const goal = (a: Team[number]["aspirations"][number]) =>
    a.kind === "skill" ? `${a.skill} → level ${a.target_level}` : `Role: ${a.role_label}`;
</script>

{#if error}
  <p class="error">{error}</p>
{:else if team.length > 0}
  <section class="panel" data-testid="team-aspirations">
    <h2>My team's aspirations</h2>
    <p class="muted">
      {direct} direct report(s), {team.length - direct} indirect. Only what each person chose to share with their managers.
    </p>
    <ul>
      {#each team as person (person.worker_pid)}
        <li>
          <strong>{person.display_name}</strong>
          <span class="chip">{person.direct_report ? "direct report" : "indirect report"}</span>
          {#if person.aspirations.length === 0}
            <span class="muted">nothing shared</span>
          {:else}
            <ul>
              {#each person.aspirations as a (a.pid)}
                <li>{goal(a)} · {a.horizon.replaceAll("_", " ")} · {a.status.replace("_", " ")}{#if a.note} — <span class="muted">{a.note}</span>{/if}</li>
              {/each}
            </ul>
          {/if}
        </li>
      {/each}
    </ul>
  </section>
{/if}

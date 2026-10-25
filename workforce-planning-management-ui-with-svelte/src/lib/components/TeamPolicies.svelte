<!--
  Team policies: for a manager, how many of their direct reports have
  acknowledged each policy aimed at the team, and how many have not. The
  service gives both numbers only from the floor; under it, nothing, since two
  numbers would name them. Renders nothing for anyone with no direct reports
  in the audience. Names are never listed here.
-->
<script lang="ts">
  import { guardedLoad } from "#lib/loader.svelte.js";
  import { errorText } from "#lib/write.js";
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import { getMyPolicies, getTeamPolicyCount } from "#lib/api/wpm.js";
  import type { TeamPolicyCount } from "#lib/api/wpm.js";
  import { t, tf } from "#lib/i18n.svelte.js";

  let { workerPid }: { workerPid: string } = $props();

  type Line = { policy: string; version: number; floor: number; count: TeamPolicyCount };
  let lines = $state<Line[]>([]);
  let error = $state<string | null>(null);

  const load = guardedLoad(
    async () => {
      const mine = await getMyPolicies();
      const out: Line[] = [];
      for (const p of mine.policies) {
        const answer = await getTeamPolicyCount(workerPid, p.version.pid);
        out.push({ policy: p.policy, version: p.version.version, floor: answer.floor, count: answer.count });
      }
      return out;
    },
    (found) => {
      lines = found.filter((l) => l.count.kind !== "no_one");
    },
    (cause) => {
      error = errorText(cause);
    },
  );

  $effect(() => {
    void workerPid;
    void load();
  });
</script>

{#if error}
  <ErrorNote testid="team-policies-error" message={error} />
{:else if lines.length > 0}
  <section class="panel" data-testid="team-policies">
    <h2>{t("teamPolicies.title")}</h2>
    <p class="muted">{t("teamPolicies.hint")}</p>
    <ul>
      {#each lines as l (l.policy + l.version)}
        <li data-testid="team-policy-row">
          <strong>{l.policy}</strong> <span class="muted">— {tf("myPolicies.version", { version: l.version })}</span>:
          {#if l.count.kind === "shown"}
            {tf("teamPolicies.shown", { acknowledged: l.count.acknowledged, waiting: l.count.waiting })}
          {:else}
            <span class="muted">{tf("teamPolicies.withheld", { floor: l.floor })}</span>
          {/if}
        </li>
      {/each}
    </ul>
  </section>
{/if}

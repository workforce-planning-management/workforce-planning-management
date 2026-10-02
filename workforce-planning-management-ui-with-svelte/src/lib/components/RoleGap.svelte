<!--
  "Compare to a role": one worker's declared proficiency against a role
  profile's requirements. Compares declarations; `undeclared` is unknown, not
  below. A development conversation starter, never a selection decision.
-->
<script lang="ts">
  import { listRoleProfiles, workerRoleGap } from "#lib/api/wpm.js";
  import { percentWithWorkings } from "#lib/format.js";

  let { workerPid }: { workerPid: string } = $props();

  type Profiles = Awaited<ReturnType<typeof listRoleProfiles>>;
  type Gap = Awaited<ReturnType<typeof workerRoleGap>>;

  let profiles = $state<Profiles>([]);
  let roleId = $state("");
  let gap = $state<Gap | null>(null);
  let error = $state<string | null>(null);

  $effect(() => {
    void (async () => {
      try {
        profiles = await listRoleProfiles();
      } catch (cause) {
        error = cause instanceof Error ? cause.message : String(cause);
      }
    })();
  });

  async function compare(id: string) {
    roleId = id;
    gap = null;
    error = null;
    if (!id) return;
    try {
      gap = await workerRoleGap(workerPid, id);
    } catch (cause) {
      error = cause instanceof Error ? cause.message : String(cause);
    }
  }
</script>

{#if profiles.length > 0}
  <div class="panel" data-testid="role-gap">
    <h3>Compare to a role</h3>
    <label>
      Role
      <select
        data-testid="role-gap-select"
        value={roleId}
        onchange={(event) => void compare(event.currentTarget.value)}
      >
        <option value="">Choose…</option>
        {#each profiles as p (p.pid)}<option value={p.pid}>{p.job_title}</option>{/each}
      </select>
    </label>
    {#if error}<p class="error">{error}</p>{/if}
    {#if gap}
      <p class="muted">{gap.derivation}</p>
      <p>
        Critical requirements met: {percentWithWorkings(gap.critical_met)} · All: {percentWithWorkings(
          gap.all_met,
        )}
      </p>
      <table>
        <thead>
          <tr><th>Skill</th><th>Importance</th><th>Needs</th><th>Declared</th><th>Status</th></tr>
        </thead>
        <tbody>
          {#each gap.requirements as row (row.skill_pid)}
            <tr>
              <td>{row.skill}</td>
              <td>{row.importance}</td>
              <td>{row.min_proficiency}</td>
              <td>{row.declared ?? "—"}</td>
              <td class:warn={row.grade !== "met"}>
                {row.grade}{#if row.shortfall !== null} (−{row.shortfall}){/if}
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    {/if}
  </div>
{/if}

<style>
  td.warn { color: #b45309; font-weight: 600; }
</style>

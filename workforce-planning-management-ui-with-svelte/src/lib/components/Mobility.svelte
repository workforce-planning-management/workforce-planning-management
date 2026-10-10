<!--
  Internal mobility (self-service): roles that fit this worker's OWN declared
  skills, open requisitions, and the interest they have chosen to express.
  Compares declarations; never ranks people. Others see only aggregate counts.
-->
<script lang="ts">
  import { t, tv } from "#lib/i18n.svelte.js";
  import {
    expressMobilityInterest,
    listMobilityInterests,
    opportunities,
    roleMatches,
    withdrawMobilityInterest,
  } from "#lib/api/wpm.js";
  import { percentWithWorkings } from "#lib/format.js";

  let { workerPid }: { workerPid: string } = $props();

  type Matches = Awaited<ReturnType<typeof roleMatches>>;
  type Opportunities = Awaited<ReturnType<typeof opportunities>>;
  type Interests = Awaited<ReturnType<typeof listMobilityInterests>>;

  let matches = $state<Matches | null>(null);
  let open = $state<Opportunities | null>(null);
  let interests = $state<Interests>([]);
  let error = $state<string | null>(null);

  const message = (cause: unknown) =>
    cause instanceof Error ? cause.message : String(cause);

  async function load() {
    try {
      [matches, open, interests] = await Promise.all([
        roleMatches(workerPid),
        opportunities(workerPid),
        listMobilityInterests(workerPid),
      ]);
    } catch (cause) {
      error = message(cause);
    }
  }

  $effect(() => {
    void load();
  });

  const interested = (pid: string) => interests.some((i) => i.target_pid === pid);

  async function express(target: { role_profile_pid?: string; requisition_pid?: string }) {
    error = null;
    try {
      await expressMobilityInterest(workerPid, target);
      await load();
    } catch (cause) {
      error = message(cause);
    }
  }

  async function withdraw(pid: string) {
    error = null;
    try {
      await withdrawMobilityInterest(pid);
      await load();
    } catch (cause) {
      error = message(cause);
    }
  }
</script>

{#if matches && (matches.roles.length > 0 || (open?.opportunities.length ?? 0) > 0)}
  <div class="panel" data-testid="mobility">
    <h3>{t("comp.mobility.roles_that_fit_my_skills")}</h3>
    <p class="muted">{matches.derivation}</p>
    {#if error}<p class="error">{error}</p>{/if}
    <table>
      <thead><tr><th>{t("comp.mobility.role")}</th><th>{t("comp.mobility.critical_met")}</th><th>{t("comp.mobility.all_met")}</th><th></th></tr></thead>
      <tbody>
        {#each matches.roles as role (role.role_profile_pid)}
          <tr>
            <td>{role.job_title}</td>
            <td>{percentWithWorkings(role.fit.critical_met)}</td>
            <td>{percentWithWorkings(role.fit.all_met)}</td>
            <td>
              {#if interested(role.role_profile_pid)}
                <span class="muted">{t("comp.mobility.interested")}</span>
              {:else}
                <button type="button" onclick={() => void express({ role_profile_pid: role.role_profile_pid })}>
                  {t("comp.mobility.i_m_interested")}
                </button>
              {/if}
            </td>
          </tr>
        {/each}
      </tbody>
    </table>

    {#if open && open.opportunities.length > 0}
      <h3>{t("comp.mobility.open_roles")}</h3>
      <table>
        <thead><tr><th>{t("comp.mobility.role")}</th><th>{t("comp.mobility.department")}</th><th>{t("comp.mobility.my_fit_critical")}</th><th></th></tr></thead>
        <tbody>
          {#each open.opportunities as opp (opp.requisition_pid)}
            <tr>
              <td>{opp.job_title}</td>
              <td>{opp.department}</td>
              <td>{opp.fit ? percentWithWorkings(opp.fit.critical_met) : "—"}</td>
              <td>
                {#if interested(opp.requisition_pid)}
                  <span class="muted">{t("comp.mobility.interested")}</span>
                {:else}
                  <button type="button" onclick={() => void express({ requisition_pid: opp.requisition_pid })}>
                    {t("comp.mobility.i_m_interested")}
                  </button>
                {/if}
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    {/if}

    {#if interests.length > 0}
      <h3>{t("comp.mobility.my_expressed_interest")}</h3>
      <ul data-testid="mobility-interests">
        {#each interests as i (i.pid)}
          <li>
            {i.title ?? i.target_pid} <span class="muted">({tv(i.kind)})</span>
            <button type="button" onclick={() => void withdraw(i.pid)}>{t("comp.mobility.withdraw")}</button>
          </li>
        {/each}
      </ul>
    {/if}
  </div>
{/if}

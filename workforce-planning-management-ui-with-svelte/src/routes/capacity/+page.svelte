<!--
  Capacity (`/capacity`, WPM-R56 to R60): the skill pools of an organization by month, supply against
  the demand of active programmes, with firm partner capacity, the constraint pool and how many
  programmes draw on it. Pools and months, never people. An unknown month is shown as unknown.
-->
<script lang="ts">
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import {
    addSkillPoolMember,
    capacityView,
    createSkillPool,
    listRoleProfiles,
    listSkills,
    setProgrammeDemand,
    type CapacityView,
  } from "#lib/api/wpm.js";
  import CapacityHeatmap from "#lib/components/CapacityHeatmap.svelte";
  import OrgPicker from "#lib/components/OrgPicker.svelte";
  import WriteResult from "#lib/components/WriteResult.svelte";
  import { t, tf } from "#lib/i18n.svelte.js";
  import { attempt, errorText, type Failure } from "#lib/write.js";

  let org = $state("");
  let view = $state<CapacityView | null>(null);
  let error = $state<string | null>(null);
  let loading = $state(false);
  let reload = $state(0);
  let failed = $state<Failure | null>(null);
  let roleProfiles = $state<Array<{ pid: string; job_title: string }>>([]);
  let skills = $state<Array<{ pid: string; name: string }>>([]);

  // The pool forms.
  let poolName = $state("");
  let reservation = $state(0);
  let memberPool = $state("");
  let memberKind = $state<"role" | "skill">("role");
  let memberRole = $state("");
  let memberSkill = $state("");
  let minProficiency = $state(3);
  // The claim form.
  let programme = $state("");
  let claimPool = $state("");
  let claimMonth = $state("");
  let claimFte = $state(1);

  async function write(action: () => Promise<unknown>) {
    failed = await attempt(action);
    if (!failed) reload += 1;
  }

  $effect(() => {
    let current = true;
    void listRoleProfiles()
      .then((p) => current && (roleProfiles = p))
      .catch(() => current && (roleProfiles = []));
    void listSkills()
      .then((p) => current && (skills = p))
      .catch(() => current && (skills = []));
    return () => {
      current = false;
    };
  });

  $effect(() => {
    const chosen = org;
    void reload;
    if (!chosen) return;
    let current = true;
    loading = true;
    void (async () => {
      try {
        const loaded = await capacityView(chosen);
        if (!current) return;
        view = loaded;
        error = null;
      } catch (cause) {
        if (!current) return;
        view = null;
        error = errorText(cause);
      } finally {
        if (current) loading = false;
      }
    })();
    return () => {
      current = false;
    };
  });

  const constraintName = $derived(
    view?.pools.find((p) => p.pid === view?.constraint_pool)?.name ?? null,
  );
</script>

<svelte:head><title>{t("titles.capacity")}</title></svelte:head>

<h1>{t("cap.title")}</h1>
<p class="muted">{t("cap.hint")}</p>

<div class="form"><OrgPicker bind:value={org} /></div>

{#if error}
  <ErrorNote testid="error" prefixed message={error} />
{:else if !org}
  <p class="muted">{t("common.noOrganization")}</p>
{:else if view === null || loading}
  <p>{t("common.loading")}</p>
{:else if view.pools.length === 0}
  <p class="muted" data-testid="capacity-none">{t("cap.none")}</p>
{:else}
  <section class="panel">
    {#if constraintName}
      <p data-testid="constraint">
        <strong>{tf("cap.constraint", { name: constraintName })}</strong>
        {#if view.wip}
          <span class="muted">
            ·
            {#if view.wip.limit === null}
              {tf("cap.wipNoLimit", { active: view.wip.active })}
            {:else}
              {tf("cap.wip", { active: view.wip.active, limit: view.wip.limit })}
            {/if}
          </span>
        {/if}
      </p>
    {/if}
    <CapacityHeatmap months={view.months} pools={view.pools} />
    <p class="muted small">{t("cap.cellHint")}</p>
    <details>
      <summary>{t("cap.basis")}</summary>
      <p class="muted">{t("cap.basisText")}</p>
      <p class="muted small">{t("common.fromService")}: {view.derivation}</p>
    </details>
  </section>
{/if}

{#if org}
  <section class="panel" data-testid="manage-capacity">
    <h2>{t("cap.manage")}</h2>
    <form
      class="form"
      onsubmit={(e) => {
        e.preventDefault();
        void write(() =>
          createSkillPool({
            organization_ref: org,
            name: poolName,
            operations_reservation_bp: Math.round(reservation * 100),
          }),
        );
      }}
    >
      <label>{t("cap.poolName")} <input bind:value={poolName} required data-testid="pool-name" /></label>
      <label>{t("cap.reservation")} <input type="number" min="0" max="100" step="0.5" bind:value={reservation} /></label>
      <button type="submit" data-testid="create-pool">{t("cap.createPool")}</button>
    </form>

    <h3>{t("cap.addMember")}</h3>
    <p class="muted small">{t("cap.memberHint")}</p>
    <form
      class="form"
      onsubmit={(e) => {
        e.preventDefault();
        void write(() =>
          addSkillPoolMember(
            memberPool,
            memberKind === "role"
              ? { role_profile_pid: memberRole }
              : { skill_pid: memberSkill, min_proficiency: minProficiency },
          ),
        );
      }}
    >
      <label>
        {t("cap.pool")}
        <select bind:value={memberPool} required data-testid="member-pool">
          <option value="" disabled></option>
          {#each view?.pools ?? [] as p (p.pid)}<option value={p.pid}>{p.name}</option>{/each}
        </select>
      </label>
      <label>
        <input type="radio" bind:group={memberKind} value="role" /> {t("cut.roleProfile")}
      </label>
      <label>
        <input type="radio" bind:group={memberKind} value="skill" /> {t("cap.skill")}
      </label>
      {#if memberKind === "role"}
        <label>
          {t("cut.roleProfile")}
          <select bind:value={memberRole} required data-testid="member-role">
            <option value="" disabled></option>
            {#each roleProfiles as p (p.pid)}<option value={p.pid}>{p.job_title}</option>{/each}
          </select>
        </label>
      {:else}
        <label>
          {t("cap.skill")}
          <select bind:value={memberSkill} required>
            <option value="" disabled></option>
            {#each skills as s (s.pid)}<option value={s.pid}>{s.name}</option>{/each}
          </select>
        </label>
        <label>
          {t("cap.minProficiency")}
          <input type="number" min="1" max="5" bind:value={minProficiency} required />
        </label>
      {/if}
      <button type="submit" data-testid="add-member">{t("common.add")}</button>
    </form>

    <h3>{t("cap.demandTitle")}</h3>
    <p class="muted small">{t("cap.claimHint")}</p>
    <form
      class="form"
      onsubmit={(e) => {
        e.preventDefault();
        void write(() =>
          setProgrammeDemand({
            programme_ref: programme,
            pool_pid: claimPool,
            month: claimMonth,
            fte_centi: Math.round(claimFte * 100),
          }),
        );
      }}
    >
      <label>{t("common.programme")} <input bind:value={programme} required data-testid="claim-programme" /></label>
      <label>
        {t("cap.pool")}
        <select bind:value={claimPool} required data-testid="claim-pool">
          <option value="" disabled></option>
          {#each view?.pools ?? [] as p (p.pid)}<option value={p.pid}>{p.name}</option>{/each}
        </select>
      </label>
      <label>{t("common.month")} <input type="date" bind:value={claimMonth} required data-testid="claim-month" /></label>
      <label>{t("cap.fte")} <input type="number" min="0" step="0.01" bind:value={claimFte} required data-testid="claim-fte" /></label>
      <button type="submit" data-testid="save-claim">{t("cap.saveClaim")}</button>
    </form>
    <WriteResult {failed} />
  </section>
{/if}

<style>
  .small {
    font-size: 0.82rem;
  }
</style>

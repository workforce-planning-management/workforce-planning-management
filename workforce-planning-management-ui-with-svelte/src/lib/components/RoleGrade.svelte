<!--
  A role profile's grade: its job level and its pay band. Setting both links a
  level to a pay band *for this role* — a statement by whoever edits the role, not
  an equivalence WPM derives (no source says which level matches which band).
-->
<script lang="ts">
  import { guardedLoad } from "#lib/loader.svelte.js";
  import { errorText } from "#lib/write.js";
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import {
    clearRoleGrade,
    getJobLevelFramework,
    getPayScale,
    getRoleGrade,
    listJobLevelFrameworks,
    listPayScales,
    moneyWhole,
    setRoleGrade,
  } from "#lib/api/wpm.js";
  import type { JobLevelFramework, JobLevelFrameworkSummary, PayScale, RoleGrade } from "#lib/api/types.js";
  import { t, i18n } from "#lib/i18n.svelte.js";

  let { profilePid }: { profilePid: string } = $props();

  let grade = $state<RoleGrade | null>(null);
  let ladder = $state<JobLevelFramework | null>(null);
  let ladders = $state<JobLevelFrameworkSummary[]>([]);
  let scale = $state<PayScale | null>(null);
  let error = $state<string | null>(null);
  let level = $state("");
  let band = $state("");

  const message = (cause: unknown) => (errorText(cause));

  const load = guardedLoad(
    async () => {
      const found = await getRoleGrade(profilePid);
      const list = await listJobLevelFrameworks();
      // The ladder the role's level is on, else the one being looked at, else the first.
      const wanted = found.job_level?.framework ?? ladder?.id ?? list[0]?.id;
      const ladderFound = wanted ? await getJobLevelFramework(wanted) : null;
      const [firstScale] = await listPayScales();
      const scaleFound = firstScale ? await getPayScale(firstScale.id) : null;
      return { found, ladderFound, scaleFound, list };
    },
    ({ found, ladderFound, scaleFound, list }) => {
      grade = found;
      level = found.job_level?.level?.code ?? "";
      band = found.pay_band?.band ?? "";
      ladder = ladderFound;
      ladders = list;
      scale = scaleFound;
    },
    (cause) => {
      error = message(cause);
    },
  );

  $effect(() => {
    void profilePid;
    void load();
  });

  async function chooseLadder(event: Event) {
    const id = (event.currentTarget as HTMLSelectElement).value;
    level = "";
    try {
      ladder = await getJobLevelFramework(id);
    } catch (cause) {
      error = message(cause);
    }
  }

  async function save(event: SubmitEvent) {
    event.preventDefault();
    error = null;
    if (!level && !band) {
      error = t("grades.pickOne");
      return;
    }
    try {
      await setRoleGrade(profilePid, {
        ...(level && ladder && { job_level: { framework: ladder.id, level } }),
        ...(band && scale && { pay_band: { scale: scale.id, band } }),
      });
      await load();
    } catch (cause) {
      error = message(cause);
    }
  }

  async function clear() {
    error = null;
    try {
      await clearRoleGrade(profilePid);
      await load();
    } catch (cause) {
      error = message(cause);
    }
  }
</script>

<section class="panel" data-testid="role-grade">
  <h3>{t("grades.roleTitle")}</h3>
  <p class="muted">{t("grades.roleHint")}</p>
  {#if error}<ErrorNote testid="error" message={error} />{/if}
  {#if grade?.job_level?.level || grade?.pay_band}
    <p data-testid="role-grade-held">
      {#if grade.job_level?.level}<strong>{grade.job_level.level.code}</strong> {grade.job_level.level.title}{/if}
      {#if grade.job_level?.level && grade.pay_band} · {/if}
      {#if grade.pay_band}
        {t("payScales.band")} <strong>{grade.pay_band.band}</strong>
        {#if grade.pay_band.entry_minor !== undefined && grade.pay_band.top_minor !== undefined}
          <span class="muted">
            ({moneyWhole(grade.pay_band.entry_minor, grade.pay_band.currency, i18n.locale)}–{moneyWhole(
              grade.pay_band.top_minor,
              grade.pay_band.currency,
              i18n.locale,
            )})
          </span>
        {/if}
      {/if}
    </p>
    <button data-testid="role-grade-clear" onclick={clear}>{t("grades.clear")}</button>
  {:else}
    <p class="muted" data-testid="role-grade-none">{t("grades.noRoleGrade")}</p>
  {/if}
  <form class="cx-form" onsubmit={save} data-testid="role-grade-form">
    {#if ladders.length > 1}
      <label>
        {t("jobLevels.framework")}
        <select data-testid="role-framework-choice" value={ladder?.id} onchange={chooseLadder}>
          {#each ladders as f (f.id)}<option value={f.id}>{f.name}</option>{/each}
        </select>
      </label>
    {/if}
    <label>
      {t("jobLevels.level")}
      <select data-testid="role-level-choice" bind:value={level}>
        <option value="">—</option>
        {#each ladder?.levels ?? [] as l (l.code)}<option value={l.code}>{l.code} — {l.title}</option>{/each}
      </select>
    </label>
    <label>
      {t("payScales.band")}
      <select data-testid="role-band-choice" bind:value={band}>
        <option value="">—</option>
        {#each scale?.bands ?? [] as b (b.code)}<option value={b.code}>{b.code}</option>{/each}
      </select>
    </label>
    <button type="submit" data-testid="role-grade-save">{t("grades.save")}</button>
  </form>
</section>

<style>
  .cx-form {
    display: flex;
    flex-wrap: wrap;
    gap: 0.75rem 1rem;
    align-items: flex-end;
    margin-top: 0.75rem;
  }
  .cx-form label {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    min-width: 0;
    max-width: 100%;
  }
  .cx-form select {
    max-width: 100%;
  }
</style>

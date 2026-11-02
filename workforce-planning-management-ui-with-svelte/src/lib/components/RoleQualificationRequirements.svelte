<!--
  What each role asks for (WPM-R171, WPM-T435): choose a role profile, read the qualifications it asks for, add one.
  A requirement is policy, not a person's record; a person's own page compares it with what they hold. A framework and
  its minimum level go together.
-->
<script lang="ts">
  import { attempt, errorText } from "#lib/write.js";
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import {
    addRoleQualificationRequirement,
    listRoleProfiles,
    roleQualificationRequirements,
  } from "#lib/api/wpm.js";
  import type { QualificationRequirement } from "#lib/api/types.js";
  import { t } from "#lib/i18n.svelte.js";

  let profiles = $state<{ pid: string; job_title: string }[]>([]);
  let role = $state("");
  let rows = $state<QualificationRequirement[] | null>(null);
  let error = $state<string | null>(null);
  let reads = $state(0);
  let title = $state("");
  let kind = $state("");
  let framework = $state("");
  let level = $state("");

  $effect(() => {
    void (async () => {
      try {
        profiles = await listRoleProfiles();
      } catch (cause) {
        error = errorText(cause);
      }
    })();
  });

  $effect(() => {
    const chosen = role;
    void reads;
    if (!chosen) {
      rows = null;
      return;
    }
    let stale = false;
    void (async () => {
      try {
        const found = await roleQualificationRequirements(chosen);
        if (!stale) rows = found.requirements;
      } catch (cause) {
        if (!stale) error = errorText(cause);
      }
    })();
    return () => {
      stale = true;
    };
  });

  async function add(event: SubmitEvent) {
    event.preventDefault();
    const n = Number.parseInt(level, 10);
    const withLevel = framework.trim() !== "" && Number.isFinite(n);
    const failed = await attempt(() =>
      addRoleQualificationRequirement(role, {
        title,
        kind: kind || undefined,
        framework: withLevel ? framework.trim() : undefined,
        min_level: withLevel ? n : undefined,
      }),
    );
    error = failed?.message ?? null;
    if (failed === null) {
      title = kind = framework = level = "";
      reads += 1;
    }
  }
</script>

<section class="panel" data-testid="role-requirements">
  <h2>{t("qualifications.reqTitle")}</h2>
  <p class="muted">{t("qualifications.reqHint")}</p>
  <label>
    {t("qualifications.reqRole")}
    <select bind:value={role} data-testid="role-requirements-role">
      <option value=""></option>
      {#each profiles as p (p.pid)}<option value={p.pid}>{p.job_title}</option>{/each}
    </select>
  </label>
  {#if error}<ErrorNote testid="role-requirements-error" message={error} />{/if}
  {#if rows}
    {#if rows.length === 0}
      <p data-testid="role-requirements-none">{t("qualifications.reqNone")}</p>
    {:else}
      <ul data-testid="role-requirements-list">
        {#each rows as r (r.pid)}
          <li>
            {r.title}
            {#if r.kind}<span class="chip">{t(`values.${r.kind}`)}</span>{/if}
            {#if r.framework}<span class="chip">{r.framework} ≥ {r.min_level}</span>{/if}
          </li>
        {/each}
      </ul>
    {/if}
    <form onsubmit={add}>
      <label
        >{t("qualifications.name")}
        <input bind:value={title} required maxlength="200" data-testid="role-requirements-title" />
      </label>
      <label
        >{t("qualifications.kind")}
        <select bind:value={kind}>
          <option value=""></option>
          <option value="academic">{t("values.academic")}</option>
          <option value="professional">{t("values.professional")}</option>
          <option value="vocational">{t("values.vocational")}</option>
          <option value="licence">{t("values.licence")}</option>
        </select>
      </label>
      <label
        >{t("qualifications.framework")}
        <input bind:value={framework} maxlength="40" />
      </label>
      <label
        >{t("qualifications.level")}
        <input bind:value={level} inputmode="numeric" />
      </label>
      <button type="submit" data-testid="role-requirements-add">{t("qualifications.reqAdd")}</button>
    </form>
  {/if}
</section>

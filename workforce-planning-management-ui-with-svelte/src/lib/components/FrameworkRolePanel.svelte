<!--
  "My role and skills" in one external framework (UK GDAD PCF or ESCO): choose
  the role you hold now, then tick the skills you have and set your own level
  (WPM 1–5). The framework's wording is a prompt, not a verdict — you decide
  your level. Selected skills become ordinary skill declarations.
-->
<script lang="ts">
  import { t, tf } from "#lib/i18n.svelte.js";
  import {
    clearFrameworkRole,
    frameworkRoleSkills,
    listFrameworkRoles,
    listRoleProfiles,
    searchEscoOccupations,
    searchEscoSkills,
    setFrameworkRole,
    setFrameworkSkills,
    type FrameworkRole,
    type FrameworkRoleSkill,
  } from "#lib/api/wpm.js";

  let {
    workerPid,
    framework,
    title,
    actingFor,
  }: {
    workerPid: string;
    framework: "uk-gdad-pcf" | "esco";
    title: string;
    /** Set when someone else is editing this person's record (HR). */
    actingFor?: string;
  } = $props();

  type Profiles = Awaited<ReturnType<typeof listRoleProfiles>>;

  let role = $state<FrameworkRole | null>(null);
  let skills = $state<FrameworkRoleSkill[]>([]);
  let rows = $state<Record<string, { have: boolean; level: number }>>({});
  let initial = $state<Record<string, number | null>>({});
  let profiles = $state<Profiles>([]);
  let pickProfile = $state("");
  let occupationQuery = $state("");
  let occupationHits = $state<Awaited<ReturnType<typeof searchEscoOccupations>>>([]);
  let extraQuery = $state("");
  let extraHits = $state<Awaited<ReturnType<typeof searchEscoSkills>>>([]);
  let changing = $state(false);
  let notice = $state<string | null>(null);
  let error = $state<string | null>(null);

  const message = (cause: unknown) => (cause instanceof Error ? cause.message : String(cause));

  const groups = $derived.by(() => {
    const byLabel = new Map<string, Profiles>();
    for (const p of profiles) {
      const label = `${p.profession ?? t("comp.frameworkRolePanel.other")} — ${p.role_name ?? ""}`;
      byLabel.set(label, [...(byLabel.get(label) ?? []), p]);
    }
    return [...byLabel.entries()]
      .sort(([a], [b]) => a.localeCompare(b))
      .map(([label, items]) => ({
        label,
        items: items.sort((x, y) => (x.level_order ?? 0) - (y.level_order ?? 0)),
      }));
  });

  async function load() {
    error = null;
    try {
      role = (await listFrameworkRoles(workerPid)).find((r) => r.framework === framework) ?? null;
      skills = [];
      rows = {};
      initial = {};
      if (role) {
        const detail = await frameworkRoleSkills(workerPid, framework);
        skills = detail.skills;
        for (const s of detail.skills) {
          rows[s.ref] = { have: s.declared !== null, level: s.declared ?? s.role_expects ?? 3 };
          initial[s.ref] = s.declared;
        }
      }
      if (framework === "uk-gdad-pcf" && profiles.length === 0) {
        profiles = await listRoleProfiles("uk-gdad-pcf");
      }
    } catch (cause) {
      error = message(cause);
    }
  }

  $effect(() => {
    void workerPid;
    void load();
  });

  async function choose(body: { role_profile_pid?: string; occupation_uri?: string }) {
    error = null;
    notice = null;
    try {
      await setFrameworkRole(workerPid, framework, body);
      changing = false;
      occupationHits = [];
      occupationQuery = "";
      await load();
    } catch (cause) {
      error = message(cause);
    }
  }

  async function clear() {
    error = null;
    try {
      await clearFrameworkRole(workerPid, framework);
      notice = t("comp.frameworkRolePanel.role_cleared");
      await load();
    } catch (cause) {
      error = message(cause);
    }
  }

  async function searchOccupations() {
    error = null;
    try {
      occupationHits =
        occupationQuery.trim().length >= 2 ? await searchEscoOccupations(occupationQuery.trim()) : [];
    } catch (cause) {
      error = message(cause);
    }
  }

  async function searchExtra() {
    error = null;
    try {
      extraHits = extraQuery.trim().length >= 2 ? await searchEscoSkills(extraQuery.trim()) : [];
    } catch (cause) {
      error = message(cause);
    }
  }

  function addExtra(hit: { uri: string; label: string }) {
    if (!skills.some((s) => s.ref === hit.uri)) {
      skills = [...skills, { ref: hit.uri, label: hit.label, declared: null, relation: undefined }];
      initial[hit.uri] = null;
    }
    rows[hit.uri] = { have: true, level: 3 };
    extraHits = [];
    extraQuery = "";
  }

  /** Only what changed: ticked skills at their level, and unticked ones that were declared. */
  const changes = $derived.by(() => {
    const out: Array<{ ref: string; proficiency: number | null }> = [];
    for (const s of skills) {
      const row = rows[s.ref];
      if (!row) continue;
      if (row.have && initial[s.ref] !== row.level) {
        out.push({ ref: s.ref, proficiency: row.level });
      } else if (!row.have && initial[s.ref] !== null && initial[s.ref] !== undefined) {
        out.push({ ref: s.ref, proficiency: null });
      }
    }
    return out;
  });

  async function save() {
    error = null;
    notice = null;
    try {
      const result = await setFrameworkSkills(workerPid, framework, changes);
      notice = tf("comp.frameworkRolePanel.saved", { declared: result.declared, removed: result.removed });
      await load();
    } catch (cause) {
      error = message(cause);
    }
  }
</script>

<section class="panel" data-testid={`framework-${framework}`}>
  <h2>{title}</h2>
  {#if actingFor}
    <p class="muted" data-testid="acting-for">
      {tf("comp.frameworkRolePanel.you_are_setting_this_for_on_their", { actingFor: actingFor })}
    </p>
  {/if}
  {#if error}<p class="error" data-testid="error">{error}</p>{/if}
  {#if notice}<p data-testid="notice">{notice}</p>{/if}

  {#if role && !changing}
    <p data-testid="current-role">
      {t("comp.frameworkRolePanel.current_role")} <strong>{role.role_label}</strong>
      <span class="muted">{tf("comp.frameworkRolePanel.chosen", { selected_on: role.selected_on })}</span>
      <button type="button" onclick={() => (changing = true)}>{t("comp.frameworkRolePanel.change")}</button>
      <button type="button" onclick={() => void clear()}>{t("comp.frameworkRolePanel.clear")}</button>
    </p>
  {:else}
    <p class="muted">
      {role ? t("comp.frameworkRolePanel.choose_different") : t("comp.frameworkRolePanel.choose_current")}
      {#if role}<button type="button" onclick={() => (changing = false)}>{t("comp.frameworkRolePanel.cancel")}</button>{/if}
    </p>
    {#if framework === "uk-gdad-pcf"}
      <form
        onsubmit={(event) => {
          event.preventDefault();
          if (pickProfile) void choose({ role_profile_pid: pickProfile });
        }}
      >
        <label>
          {t("comp.frameworkRolePanel.role_level")}
          <select data-testid="pcf-role" bind:value={pickProfile} required>
            <option value="" disabled>{t("comp.frameworkRolePanel.choose")}</option>
            {#each groups as g (g.label)}
              <optgroup label={g.label}>
                {#each g.items as p (p.pid)}<option value={p.pid}>{p.job_title}</option>{/each}
              </optgroup>
            {/each}
          </select>
        </label>
        <button type="submit">{t("comp.frameworkRolePanel.select")}</button>
      </form>
    {:else}
      <form
        onsubmit={(event) => {
          event.preventDefault();
          void searchOccupations();
        }}
      >
        <label>{t("comp.frameworkRolePanel.occupation")} <input data-testid="esco-role-query" bind:value={occupationQuery} minlength="2" /></label>
        <button type="submit">{t("comp.frameworkRolePanel.search")}</button>
      </form>
      <ul>
        {#each occupationHits as hit (hit.uri)}
          <li>
            <button type="button" onclick={() => void choose({ occupation_uri: hit.uri })}>{hit.label}</button>
            <span class="muted">{tf("comp.frameworkRolePanel.isco", { isco_code: hit.isco_code ?? "—" })}</span>
          </li>
        {/each}
      </ul>
    {/if}
  {/if}

  {#if role && !changing}
    <h3>{t("comp.frameworkRolePanel.my_skills_in_this_role")}</h3>
    <p class="muted">
      {t("comp.frameworkRolePanel.tick_the_skills_you_have_and_set")}
    </p>
    <table data-testid={`skills-${framework}`}>
      <thead>
        <tr><th>{t("comp.frameworkRolePanel.i_have_it")}</th><th>{t("comp.frameworkRolePanel.skill")}</th><th>{framework === "esco" ? t("comp.frameworkRolePanel.relation") : t("comp.frameworkRolePanel.role_expects")}</th><th>{t("comp.frameworkRolePanel.my_level")}</th></tr>
      </thead>
      <tbody>
        {#each skills as s (s.ref)}
          <tr>
            <td>
              <input
                type="checkbox"
                aria-label={`I have ${s.label}`}
                checked={rows[s.ref]?.have ?? false}
                onchange={(event) => {
                  rows[s.ref] = { have: event.currentTarget.checked, level: rows[s.ref]?.level ?? 3 };
                }}
              />
            </td>
            <td>
              {s.label}
              {#if s.wording}<details><summary class="muted">{t("comp.frameworkRolePanel.what_the_framework_says")}</summary><p>{s.wording}</p></details>{/if}
            </td>
            <td class="muted">
              {#if s.role_expects !== undefined}
                {s.role_expects}{s.framework_level ? ` (framework ${s.framework_level} of ${s.framework_scale_max})` : ""}
              {:else}{s.relation ?? t("comp.frameworkRolePanel.added_by_me")}{/if}
            </td>
            <td>
              <select
                aria-label={`My level in ${s.label}`}
                disabled={!rows[s.ref]?.have}
                value={rows[s.ref]?.level ?? 3}
                onchange={(event) => {
                  rows[s.ref] = { have: true, level: Number(event.currentTarget.value) };
                }}
              >
                {#each [1, 2, 3, 4, 5] as level (level)}<option value={level}>{level}</option>{/each}
              </select>
            </td>
          </tr>
        {:else}
          <tr><td colspan="4" class="muted">{t("comp.frameworkRolePanel.this_role_lists_no_skills")}</td></tr>
        {/each}
      </tbody>
    </table>
    {#if framework === "esco"}
      <form
        onsubmit={(event) => {
          event.preventDefault();
          void searchExtra();
        }}
      >
        <label>{t("comp.frameworkRolePanel.add_another_esco_skill")} <input bind:value={extraQuery} minlength="2" /></label>
        <button type="submit">{t("comp.frameworkRolePanel.search")}</button>
      </form>
      <ul>
        {#each extraHits as hit (hit.uri)}
          <li><button type="button" onclick={() => addExtra(hit)}>{hit.label}</button></li>
        {/each}
      </ul>
    {/if}
    <p>
      <button type="button" data-testid={`save-${framework}`} disabled={changes.length === 0} onclick={() => void save()}>
        {tf("comp.frameworkRolePanel.save_my_skills_change", { changes: changes.length, changes2: changes.length === 1 ? "" : "s" })}
      </button>
    </p>
  {/if}
</section>

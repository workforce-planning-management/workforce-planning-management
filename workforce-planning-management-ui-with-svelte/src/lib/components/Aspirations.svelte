<!--
  Aspirations, learning goals, and growth ideas: a future role or a skill level
  to aim for, with a horizon, a status, and a note in the person's own words.
  Private unless the person shares it. Progress is measured against the skills
  they have declared today.
-->
<script lang="ts">
  import { t, tf, tv } from "#lib/i18n.svelte.js";
  import {
    ASPIRATION_HORIZONS,
    ASPIRATION_STATUSES,
    addAspiration,
    deleteAspiration,
    listAspirations,
    listSkills,
    updateAspiration,
    VISIBILITIES,
    type Visibility,
    type Aspiration,
  } from "#lib/api/wpm.js";
  import RolePicker from "#lib/components/RolePicker.svelte";

  let { workerPid }: { workerPid: string } = $props();

  let items = $state<Aspiration[]>([]);
  let isPerson = $state(true);
  let catalogue = $state<Awaited<ReturnType<typeof listSkills>>>([]);
  let error = $state<string | null>(null);

  let kind = $state<"skill" | "role">("skill");
  let skillPid = $state("");
  let target = $state(4);
  let framework = $state<"uk-gdad-pcf" | "esco">("uk-gdad-pcf");
  let picked = $state<{ role_profile_pid?: string; occupation_uri?: string; label: string } | null>(null);
  let horizon = $state<string>("within_1y");
  let note = $state("");
  let visibility = $state<Visibility>("private");

  const message = (cause: unknown) => (cause instanceof Error ? cause.message : String(cause));

  async function load() {
    try {
      const [list, skills] = await Promise.all([listAspirations(workerPid), listSkills()]);
      items = list.aspirations;
      isPerson = list.viewer_is_the_person;
      catalogue = skills;
    } catch (cause) {
      error = message(cause);
    }
  }

  $effect(() => {
    void workerPid;
    void load();
  });

  async function run(action: () => Promise<unknown>) {
    error = null;
    try {
      await action();
      await load();
    } catch (cause) {
      error = message(cause);
    }
  }

  function describe(a: Aspiration): string {
    const p = a.progress as Record<string, number | boolean | null> | null;
    if (!p) return "";
    if (a.kind === "skill") {
      return p.achieved
        ? t("comp.aspirations.reached")
        : p.current_level === null ? t("comp.aspirations.not_declared_yet") : tf("comp.aspirations.at_level_gap", { level: p.current_level, gap: p.gap });
    }
    if ("requirements" in p) return tf("comp.aspirations.requirements_met", { met: p.met, requirements: p.requirements });
    if ("essential_skills" in p) return tf("comp.aspirations.essential_skills_have", { have: p.have, essential: p.essential_skills });
    return "";
  }
</script>

<section class="panel" data-testid="aspirations">
  <h2>{t("comp.aspirations.aspirations_and_growth_ideas")}</h2>
  {#if error}<p class="error" data-testid="error">{error}</p>{/if}
  {#if !isPerson}
    <p class="muted">{t("comp.aspirations.you_see_only_what_this_person_has")}</p>
  {/if}
  <table data-testid="aspiration-list">
    <thead><tr><th>{t("comp.aspirations.goal")}</th><th>{t("comp.aspirations.when")}</th><th>{t("comp.aspirations.status")}</th><th>{t("comp.aspirations.progress")}</th><th>{t("comp.aspirations.who_can_see")}</th><th></th></tr></thead>
    <tbody>
      {#each items as a (a.pid)}
        <tr>
          <td>
            {a.kind === "skill" ? tf("comp.aspirations.skill_to_level", { skill: a.skill, level: a.target_level }) : tf("comp.aspirations.role_framework", { role: a.role_label, framework: a.framework === "esco" ? "ESCO" : "PCF" })}
            {#if a.note}<br /><span class="muted">{a.note}</span>{/if}
            {#if a.on_behalf}<span class="chip">{t("comp.aspirations.added_on_their_behalf")}</span>{/if}
          </td>
          <td>{tv(a.horizon)}</td>
          <td>
            <select
              aria-label={t("comp.aspirations.status")}
              value={a.status}
              onchange={(event) => void run(() => updateAspiration(a.pid, { status: event.currentTarget.value }))}
            >
              {#each ASPIRATION_STATUSES as s (s)}<option value={s}>{tv(s)}</option>{/each}
            </select>
          </td>
          <td class="muted">{describe(a)}</td>
          <td>
            {#if isPerson}
              <select
                aria-label={t("comp.aspirations.who_can_see_this")}
                value={a.visibility}
                onchange={(event) => void run(() => updateAspiration(a.pid, { visibility: event.currentTarget.value as Visibility }))}
              >
                {#each VISIBILITIES as v (v)}<option value={v}>{v === "manager" ? t("comp.aspirations.visibility_manager") : tv(v)}</option>{/each}
              </select>
            {:else}
              {a.visibility}
            {/if}
          </td>
          <td><button type="button" onclick={() => void run(() => deleteAspiration(a.pid))}>{t("comp.aspirations.remove")}</button></td>
        </tr>
      {:else}
        <tr><td colspan="6" class="muted">{t("comp.aspirations.nothing_here_yet")}</td></tr>
      {/each}
    </tbody>
  </table>

  <h3>{t("comp.aspirations.add_one")}</h3>
  <form
    onsubmit={(event) => {
      event.preventDefault();
      const body =
        kind === "skill"
          ? { kind, skill_pid: skillPid, target_level: target }
          : picked
            ? { kind, framework_slug: framework, ...(picked.role_profile_pid ? { role_profile_pid: picked.role_profile_pid } : {}), ...(picked.occupation_uri ? { occupation_uri: picked.occupation_uri } : {}) }
            : null;
      if (!body) return;
      void run(async () => {
        await addAspiration(workerPid, { ...body, horizon, visibility, ...(note.trim() ? { note: note.trim() } : {}) });
        note = "";
        picked = null;
      });
    }}
  >
    <label>
      {t("comp.aspirations.i_want_to")}
      <select bind:value={kind}><option value="skill">{t("comp.aspirations.grow_a_skill")}</option><option value="role">{t("comp.aspirations.move_into_a_role")}</option></select>
    </label>
    {#if kind === "skill"}
      <label>
        {t("comp.aspirations.skill")}
        <select bind:value={skillPid} required>
          <option value="" disabled>{t("comp.aspirations.choose")}</option>
          {#each catalogue as s (s.pid)}<option value={s.pid}>{s.name}</option>{/each}
        </select>
      </label>
      <label>{t("comp.aspirations.level")} <select bind:value={target}>{#each [1, 2, 3, 4, 5] as l (l)}<option value={l}>{l}</option>{/each}</select></label>
    {:else}
      <label>{t("comp.aspirations.framework")} <select bind:value={framework} onchange={() => (picked = null)}><option value="uk-gdad-pcf">{t("comp.aspirations.uk_gdad_pcf")}</option><option value="esco">{t("comp.aspirations.esco")}</option></select></label>
      <RolePicker {framework} onpick={(p) => (picked = p)} />
      {#if picked}<strong>{picked.label}</strong>{/if}
    {/if}
    <label>{t("comp.aspirations.when")} <select bind:value={horizon}>{#each ASPIRATION_HORIZONS as h (h)}<option value={h}>{tv(h)}</option>{/each}</select></label>
    <label>{t("comp.aspirations.in_my_own_words")} <input bind:value={note} maxlength="1000" placeholder={t("comp.aspirations.a_growth_idea_a_learning_goal")} /></label>
    <label>{t("comp.aspirations.who_can_see_it")} <select bind:value={visibility}>{#each VISIBILITIES as v (v)}<option value={v}>{v === "manager" ? "my managers (everyone above me)" : v === "everyone" ? "everyone who can view my record" : "only me"}</option>{/each}</select></label>
    <button type="submit" data-testid="aspiration-add">{t("comp.aspirations.add")}</button>
  </form>
</section>

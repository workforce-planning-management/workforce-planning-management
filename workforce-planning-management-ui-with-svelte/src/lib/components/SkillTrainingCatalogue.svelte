<!--
  The training catalogue for one skill: the courses that build it (hours, and
  how many levels each typically adds) and the skill's own hours-per-level
  planning figure. Recommendations prefer these courses; without any they use
  the per-level estimate and say so. Changes are for whoever owns the skills
  catalogue (the service enforces it).
-->
<script lang="ts">
  import {
    addSkillCourse,
    listSkills,
    removeSkillCourse,
    setSkillTrainingHours,
    skillCourses,
  } from "#lib/api/wpm.js";
  import type { SkillCourse } from "#lib/api/types.js";
  import { t } from "#lib/i18n.svelte.js";

  let skills = $state<Array<{ pid: string; name: string }>>([]);
  let chosen = $state("");
  let courses = $state<SkillCourse[]>([]);
  let perLevel = $state<number | null>(null);
  let defaultPerLevel = $state(30);
  let error = $state<string | null>(null);
  let ref = $state("");
  let title = $state("");
  let hours = $state<number | null>(null);
  let levels = $state(1);

  const message = (cause: unknown) => (cause instanceof Error ? cause.message : String(cause));

  $effect(() => {
    void (async () => {
      try {
        skills = await listSkills();
      } catch (cause) {
        error = message(cause);
      }
    })();
  });

  async function load() {
    if (!chosen) return;
    try {
      const view = await skillCourses(chosen);
      courses = view.courses;
      perLevel = view.hours_per_level;
      defaultPerLevel = view.default_hours_per_level;
    } catch (cause) {
      error = message(cause);
    }
  }

  $effect(() => {
    void chosen;
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
</script>

<section class="panel" data-testid="skill-catalogue">
  <h2>{t("training.catalogueTitle")}</h2>
  {#if error}<p class="error" data-testid="error">{error}</p>{/if}
  <select bind:value={chosen} data-testid="catalogue-skill" aria-label={t("nav.skills")}>
    <option value="" disabled>{t("training.pickSkill")}</option>
    {#each skills as s (s.pid)}<option value={s.pid}>{s.name}</option>{/each}
  </select>

  {#if chosen}
    <form
      onsubmit={(event) => {
        event.preventDefault();
        void run(() => setSkillTrainingHours(chosen, perLevel));
      }}
    >
      <label>
        {t("training.hoursPerLevel")}
        <input type="number" min="1" max="500" placeholder={String(defaultPerLevel)} bind:value={perLevel} data-testid="catalogue-hpl" />
      </label>
      <button type="submit" data-testid="catalogue-save-hours">{t("training.saveHours")}</button>
    </form>

    <ul data-testid="catalogue-courses">
      {#each courses as c (c.pid)}
        <li>
          <strong>{c.title}</strong>
          <span class="muted">· {c.course_ref} · {c.hours} {t("training.hours")} · +{c.levels}</span>
          <button type="button" onclick={() => void run(() => removeSkillCourse(c.pid))}>{t("training.removeCourse")}</button>
        </li>
      {:else}
        <li class="muted">{t("training.noCourses")}</li>
      {/each}
    </ul>

    <form
      onsubmit={(event) => {
        event.preventDefault();
        void run(async () => {
          await addSkillCourse(chosen, {
            course_ref: ref.trim(),
            title: title.trim(),
            hours: hours ?? 0,
            levels,
          });
          ref = title = "";
          hours = null;
          levels = 1;
        });
      }}
    >
      <label>{t("training.courseRef")} <input bind:value={ref} required maxlength="200" data-testid="course-ref" /></label>
      <label>{t("training.courseTitle")} <input bind:value={title} required maxlength="200" data-testid="course-title" /></label>
      <label>{t("training.courseHours")} <input type="number" min="1" max="1000" bind:value={hours} required data-testid="course-hours" /></label>
      <label>{t("training.courseLevels")} <input type="number" min="1" max="4" bind:value={levels} /></label>
      <button type="submit" data-testid="course-add">{t("training.addCourse")}</button>
    </form>
  {/if}
</section>

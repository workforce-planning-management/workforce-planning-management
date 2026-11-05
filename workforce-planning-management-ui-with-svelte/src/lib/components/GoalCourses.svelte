<!--
  The catalogue courses that build a goal's skill (WPM-R235): hours, the fee, and where the person already stands on each. These are
  suggestions and nothing here enrols anyone (WPM-D181). A course with no fee recorded says so, because unknown is never free.
-->
<script lang="ts">
  import { guardedLoad } from "#lib/loader.svelte.js";
  import { errorText } from "#lib/write.js";
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import { listTraining, money, skillCourses } from "#lib/api/wpm.js";
  import type { SkillCourse } from "#lib/api/types.js";
  import { t, tv } from "#lib/i18n.svelte.js";

  let { skillPid, workerPid }: { skillPid: string; workerPid: string } = $props();

  let courses = $state<SkillCourse[] | null>(null);
  let standing = $state<Map<string, string>>(new Map());
  let error = $state<string | null>(null);

  const load = guardedLoad(
    // Standing is an extra: a reader who may not see the person's training still sees the catalogue, with no status claimed.
    () => Promise.all([skillCourses(skillPid), listTraining(workerPid).catch(() => [])]),
    ([found, training]) => {
      courses = found.courses;
      standing = new Map(training.map((row) => [row.course_ref, row.status]));
    },
    (cause) => {
      error = errorText(cause);
    },
  );

  $effect(() => {
    void skillPid;
    void workerPid;
    void load();
  });
</script>

{#if error}
  <ErrorNote testid="goal-courses-error" message={error} />
{:else if courses === null}
  <p>{t("common.loading")}</p>
{:else if courses.length === 0}
  <p class="muted" data-testid="goal-courses-none">{t("goals.coursesNone")}</p>
{:else}
  <ul>
    {#each courses as row (row.pid)}
      <li data-testid="goal-course">
        <bdi>{row.title}</bdi>
        <span class="muted">
          · {row.hours} {t("training.hours")} ·
          <span data-testid="goal-course-fee">{row.fee_minor === null ? t("training.noFee") : money(row.fee_minor, row.fee_currency)}</span>
          {#if standing.has(row.course_ref)}· <span data-testid="goal-course-standing">{tv(standing.get(row.course_ref))}</span>{/if}
        </span>
      </li>
    {/each}
  </ul>
  <p class="muted">{t("goals.coursesHint")}</p>
{/if}

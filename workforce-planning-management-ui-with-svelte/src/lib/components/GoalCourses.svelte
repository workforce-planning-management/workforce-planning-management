<!--
  The catalogue courses that build a goal's skill (WPM-R235): hours, the fee, and where the person stands on each. The person can start
  one from here: a recorded fee of zero enrols them, and anything else asks a manager above them, because unknown is never free
  (WPM-R236, WPM-D182). The service decides which, so a fee that has just changed cannot slip through. Anyone else reads the list only.
-->
<script lang="ts">
  import { guardedLoad } from "#lib/loader.svelte.js";
  import { errorText } from "#lib/write.js";
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import { listTraining, money, myCourseRequests, skillCourses, startCourse, withdrawCourseRequest, type CourseRequest } from "#lib/api/wpm.js";
  import type { SkillCourse, TrainingEnrollment } from "#lib/api/types.js";
  import { t, tv } from "#lib/i18n.svelte.js";

  let { skillPid, workerPid, own = false }: { skillPid: string; workerPid: string; own?: boolean } = $props();

  let courses = $state<SkillCourse[] | null>(null);
  let training = $state<TrainingEnrollment[] | null>(null);
  let requests = $state<CourseRequest[]>([]);
  let hasManager = $state(true);
  let error = $state<string | null>(null);
  let busy = $state(false);

  const load = guardedLoad(
    // Standing is an extra: a reader who may not see the person's training still sees the catalogue, with no status claimed.
    () =>
      Promise.all([
        skillCourses(skillPid),
        listTraining(workerPid).catch(() => null),
        own ? myCourseRequests() : Promise.resolve({ has_manager: true, requests: [] }),
      ]),
    ([found, enrolled, mine]) => {
      courses = found.courses;
      training = enrolled;
      requests = mine.requests;
      hasManager = mine.has_manager;
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

  const standing = (ref: string) => training?.find((row) => row.course_ref === ref)?.status ?? null;
  // Newest first, so this is the latest word on the course.
  const request = (ref: string) => requests.find((row) => row.course_ref === ref) ?? null;
  const waiting = (ref: string) => request(ref)?.status === "requested";
  const offered = (ref: string) => own && training !== null && standing(ref) !== "enrolled" && !waiting(ref);

  async function run(action: () => Promise<unknown>) {
    error = null;
    busy = true;
    try {
      await action();
      await load();
    } catch (cause) {
      error = errorText(cause);
    } finally {
      busy = false;
    }
  }
</script>

{#if error}<ErrorNote testid="goal-courses-error" message={error} />{/if}
{#if courses === null}
  {#if !error}<p>{t("common.loading")}</p>{/if}
{:else if courses.length === 0}
  <p class="muted" data-testid="goal-courses-none">{t("goals.coursesNone")}</p>
{:else}
  <ul>
    {#each courses as row (row.pid)}
      {@const status = standing(row.course_ref)}
      {@const asked = own ? request(row.course_ref) : null}
      <li data-testid="goal-course">
        <bdi>{row.title}</bdi>
        <span class="muted">
          · {row.hours} {t("training.hours")} ·
          <span data-testid="goal-course-fee">{row.fee_minor === null ? t("training.noFee") : money(row.fee_minor, row.fee_currency)}</span>
          {#if status}· <span data-testid="goal-course-standing">{tv(status)}</span>{/if}
          {#if asked && asked.status !== "approved"}· <span data-testid="goal-course-request">{tv(asked.status)}</span>{/if}
        </span>
        {#if offered(row.course_ref)}
          <div>
            <button
              type="button"
              disabled={busy}
              aria-label={`${row.fee_minor === 0 ? t("goals.enrol") : t("goals.askToEnrol")}: ${row.title}`}
              data-testid={row.fee_minor === 0 ? "goal-course-enrol" : "goal-course-ask"}
              onclick={() => void run(() => startCourse(row.course_ref))}
            >
              {row.fee_minor === 0 ? t("goals.enrol") : t("goals.askToEnrol")}
            </button>
          </div>
        {:else if asked && waiting(row.course_ref)}
          <div>
            <button
              type="button"
              disabled={busy}
              aria-label={`${t("goals.withdrawRequest")}: ${row.title}`}
              data-testid="goal-course-withdraw"
              onclick={() => void run(() => withdrawCourseRequest(asked.pid))}
            >
              {t("goals.withdrawRequest")}
            </button>
          </div>
        {/if}
      </li>
    {/each}
  </ul>
  <p class="muted">{own ? t("goals.coursesHintOwn") : t("goals.coursesHint")}</p>
  {#if own && !hasManager}<p class="muted" data-testid="goal-courses-no-manager">{t("goals.noManagerToApprove")}</p>{/if}
{/if}

<!--
  Course requests waiting for the reader's decision (WPM-R236): who asked, the course, its hours and its fee as the catalogue has it
  now, where unknown says so. Approving enrols the person. No reason is asked for or kept. With nothing waiting, the panel is not there.
-->
<script lang="ts">
  import { guardedLoad } from "#lib/loader.svelte.js";
  import { errorText } from "#lib/write.js";
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import { approveCourseRequest, declineCourseRequest, money, teamCourseRequests, type TeamCourseRequest } from "#lib/api/wpm.js";
  import { t } from "#lib/i18n.svelte.js";

  let rows = $state<TeamCourseRequest[]>([]);
  let error = $state<string | null>(null);
  let busy = $state(false);

  const load = guardedLoad(
    async () => (await teamCourseRequests()).waiting,
    (found) => {
      rows = found;
    },
    (cause) => {
      error = errorText(cause);
    },
  );

  $effect(() => {
    void load();
  });

  async function decide(action: () => Promise<unknown>) {
    error = null;
    busy = true;
    try {
      await action();
    } catch (cause) {
      error = errorText(cause);
    } finally {
      busy = false;
      await load();
    }
  }
</script>

{#if error}<ErrorNote testid="course-requests-error" message={error} />{/if}
{#if rows.length > 0}
  <section class="panel" data-testid="course-requests">
    <h2>{t("goals.courseRequests")}</h2>
    <ul>
      {#each rows as row (row.pid)}
        <li data-testid="course-request">
          <strong>{row.display_name}</strong>:
          <bdi>{row.title ?? row.course_ref}</bdi>
          <span class="muted">
            {#if row.hours !== null}· {row.hours} {t("training.hours")}{/if}
            · <span data-testid="course-request-fee">{row.fee_minor === null ? t("training.noFee") : money(row.fee_minor, row.fee_currency)}</span>
          </span>
          <div>
            <button
              type="button"
              disabled={busy}
              aria-label={`${t("goals.approve")}: ${row.display_name}, ${row.title ?? row.course_ref}`}
              data-testid="course-approve"
              onclick={() => void decide(() => approveCourseRequest(row.pid))}
            >
              {t("goals.approve")}
            </button>
            <button
              type="button"
              disabled={busy}
              aria-label={`${t("goals.decline")}: ${row.display_name}, ${row.title ?? row.course_ref}`}
              data-testid="course-decline"
              onclick={() => void decide(() => declineCourseRequest(row.pid))}
            >
              {t("goals.decline")}
            </button>
          </div>
        </li>
      {/each}
    </ul>
  </section>
{/if}

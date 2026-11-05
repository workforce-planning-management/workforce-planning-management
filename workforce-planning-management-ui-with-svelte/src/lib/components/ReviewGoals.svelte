<!--
  The goals of a review (WPM-R225): those of its period, with the reviewer's and the reviewee's outcomes kept apart. A shared
  review shows what it froze, so a later change to a goal never reaches it (WPM-D165).
-->
<script lang="ts">
  import { guardedLoad } from "#lib/loader.svelte.js";
  import { errorText } from "#lib/write.js";
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import { recordGoalOutcome, reviewGoals, type ReviewGoalsView } from "#lib/api/wpm.js";
  import type { Review } from "#lib/api/types.js";
  import { t, tv } from "#lib/i18n.svelte.js";

  let { reviews }: { reviews: Review[] } = $props();

  const OUTCOMES = ["achieved", "partly_achieved", "not_achieved", "withdrawn"];

  let chosen = $state("");
  let view = $state<ReviewGoalsView | null>(null);
  let error = $state<string | null>(null);
  let drafts = $state<Record<string, string>>({});

  const load = guardedLoad(
    async () => (chosen ? await reviewGoals(chosen) : null),
    (found) => {
      view = found;
    },
    (cause) => {
      error = errorText(cause);
    },
  );

  $effect(() => {
    void chosen;
    void load();
  });

  const outcome = (value: string | null) => (value ? tv(value) : "—");

  async function record(goal: string) {
    const value = drafts[goal];
    if (!value) return;
    error = null;
    try {
      await recordGoalOutcome(chosen, goal, value);
      await load();
    } catch (cause) {
      error = errorText(cause);
    }
  }
</script>

{#if reviews.length > 0}
  <section class="panel" data-testid="review-goals">
    <h2>{t("goals.reviewGoals")}</h2>
    {#if error}<ErrorNote testid="review-goals-error" message={error} />{/if}
    <select bind:value={chosen} aria-label={t("goals.reviewGoals")} data-testid="review-goals-pick">
      <option value="" disabled>{t("goals.pickReview")}</option>
      {#each reviews as review (review.pid)}
        <option value={review.pid}>{t("goals.reviewStatus")}: {tv(review.status)}</option>
      {/each}
    </select>
    {#if view?.frozen}
      <p class="muted" data-testid="review-goals-frozen">{t("goals.frozen")}</p>
      <ul>
        {#each view.goals as row (row.goal_pid)}
          <li data-testid="review-goal">
            {row.snapshot?.title} · {row.snapshot ? tv(row.snapshot.status) : "—"}
            <span class="muted">
              · {t("goals.reviewerOutcome")}: {outcome(row.reviewer_outcome)} · {t("goals.revieweeOutcome")}:
              {outcome(row.reviewee_outcome)}
            </span>
          </li>
        {:else}
          <li class="muted">{t("goals.reviewNone")}</li>
        {/each}
      </ul>
    {:else if view}
      <ul>
        {#each view.goals as row (row.goal_pid)}
          <li data-testid="review-goal">
            {row.view?.goal.title}
            <span class="muted">
              · {t("goals.reviewerOutcome")}: {outcome(row.reviewer_outcome)} · {t("goals.revieweeOutcome")}:
              {outcome(row.reviewee_outcome)}
            </span>
            <label>
              {t("goals.myOutcome")}
              <select bind:value={drafts[row.goal_pid]} data-testid="review-goal-outcome">
                <option value=""></option>
                {#each OUTCOMES as value (value)}<option {value}>{tv(value)}</option>{/each}
              </select>
            </label>
            <button type="button" data-testid="review-goal-record" onclick={() => void record(row.goal_pid)}>
              {t("goals.record")}
            </button>
          </li>
        {:else}
          <li class="muted">{t("goals.reviewNone")}</li>
        {/each}
      </ul>
    {/if}
  </section>
{/if}

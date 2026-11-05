<!--
  One goal (WPM-R224): its status, its measures with their own start, current and target and nothing worked out from them, its
  check-ins, and what the reader may do next. An unknown value shows a dash, never a zero, and nothing here is a score.
-->
<script lang="ts">
  import { formatDate, formatDateRange } from "#lib/locale-format.js";
  import { errorText } from "#lib/write.js";
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import { agreeGoal, checkInGoal, moveGoal, type GoalView } from "#lib/api/wpm.js";
  import { t, tv } from "#lib/i18n.svelte.js";

  let {
    view,
    party,
    onchange,
  }: { view: GoalView; party: "worker" | "manager"; onchange: () => void | Promise<void> } = $props();

  const NEXT: Record<string, string[]> = {
    proposed: ["withdrawn"],
    agreed: ["in_progress", "achieved", "partly_achieved", "not_achieved", "withdrawn"],
    in_progress: ["achieved", "partly_achieved", "not_achieved", "withdrawn"],
  };

  const goal = $derived(view.goal);
  const moves = $derived(NEXT[goal.status] ?? []);
  const open = $derived(goal.status === "agreed" || goal.status === "in_progress");
  const canAgree = $derived(goal.status === "proposed" && goal.proposed_by !== party);

  let error = $state<string | null>(null);
  let to = $state("");
  let standing = $state("on_track");
  let note = $state("");
  let measure = $state("");
  let current = $state<number | null>(null);

  const shown = (value: number | null) => value ?? "—";

  async function run(action: () => Promise<unknown>) {
    error = null;
    try {
      await action();
      await onchange();
    } catch (cause) {
      error = errorText(cause);
    }
  }

  function checkIn(event: SubmitEvent) {
    event.preventDefault();
    void run(async () => {
      await checkInGoal(goal.pid, {
        status: standing,
        ...(note.trim() ? { note: note.trim() } : {}),
        ...(measure && current !== null ? { measure_pid: measure, current_value: current } : {}),
      });
      note = "";
      current = null;
    });
  }
</script>

<article class="panel" data-testid="goal">
  <h3>{goal.title}</h3>
  <p class="muted">
    <span data-testid="goal-status">{tv(goal.status)}</span>
    · {formatDateRange(goal.period_start, goal.period_end)}
  </p>
  {#if goal.description}<p>{goal.description}</p>{/if}
  {#if error}<ErrorNote testid="goal-error" message={error} />{/if}

  {#if view.measures.length > 0}
    <h4>{t("goals.measures")}</h4>
    <ul>
      {#each view.measures as row (row.pid)}
        <li data-testid="goal-measure">
          {row.name}:
          {#if row.kind === "milestone"}
            {row.done === null ? "—" : row.done ? t("goals.done") : t("goals.notDone")}
          {:else}
            {t("goals.start")}
            {shown(row.start_value)} · {t("goals.current")}
            {shown(row.current_value)} · {t("goals.target")}
            {shown(row.target_value)}
            {#if row.unit}<span class="muted">{row.unit}</span>{/if}
          {/if}
        </li>
      {/each}
    </ul>
  {/if}

  {#if canAgree}
    <button type="button" data-testid="goal-agree" onclick={() => void run(() => agreeGoal(goal.pid))}>
      {t("goals.agree")}
    </button>
  {:else if goal.status === "proposed"}
    <p class="muted" data-testid="goal-waiting">{t("goals.waiting")}</p>
  {/if}

  {#if moves.length > 0}
    <form
      onsubmit={(event) => {
        event.preventDefault();
        void run(() => moveGoal(goal.pid, to));
      }}
    >
      <label>
        {t("goals.moveTo")}
        <select bind:value={to} required data-testid="goal-move-to">
          <option value="" disabled></option>
          {#each moves as next (next)}<option value={next}>{tv(next)}</option>{/each}
        </select>
      </label>
      <button type="submit" data-testid="goal-move">{t("goals.move")}</button>
    </form>
  {/if}

  {#if open}
    <form onsubmit={checkIn}>
      <label>
        {t("goals.checkIn")}
        <select bind:value={standing} data-testid="checkin-status">
          <option value="on_track">{tv("on_track")}</option>
          <option value="at_risk">{tv("at_risk")}</option>
          <option value="off_track">{tv("off_track")}</option>
        </select>
      </label>
      <label>{t("goals.note")} <input bind:value={note} maxlength="2000" data-testid="checkin-note" /></label>
      {#if view.measures.some((row) => row.kind === "number")}
        <label>
          {t("goals.measures")}
          <select bind:value={measure} data-testid="checkin-measure">
            <option value=""></option>
            {#each view.measures.filter((row) => row.kind === "number") as row (row.pid)}
              <option value={row.pid}>{row.name}</option>
            {/each}
          </select>
        </label>
        <label>{t("goals.current")} <input type="number" bind:value={current} data-testid="checkin-value" /></label>
      {/if}
      <button type="submit" data-testid="checkin-add">{t("goals.addCheckIn")}</button>
    </form>
  {/if}

  {#if view.check_ins.length > 0}
    <h4>{t("goals.checkIns")}</h4>
    <ul>
      {#each view.check_ins as row (row.pid)}
        <li data-testid="goal-check-in">
          {formatDate(row.checked_on)}: {tv(row.status)}
          {#if row.note}<span class="muted">· {row.note}</span>{/if}
        </li>
      {/each}
    </ul>
  {/if}
</article>

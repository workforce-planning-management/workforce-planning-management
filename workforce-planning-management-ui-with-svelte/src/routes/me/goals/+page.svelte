<!--
  My goals (WPM-R224, WPM-R226): the person's own goals, the goals of the people below them by name, and a form to propose one.
  The person or a manager proposes and the other agrees; HR sees and does not agree. Nothing here is a score, and no pay,
  progression or employment decision is made from a goal.
-->
<script lang="ts">
  import { page } from "$app/state";
  import { guardedLoad } from "#lib/loader.svelte.js";
  import { errorText } from "#lib/write.js";
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import GoalCard from "#lib/components/GoalCard.svelte";
  import GoalForm from "#lib/components/GoalForm.svelte";
  import { myTeamGoals, proposeGoalFor, proposeMyGoal, workerGoals, type GoalBody, type GoalView, type TeamGoals } from "#lib/api/wpm.js";
  import type { MyOrganization } from "#lib/api/types.js";
  import { t, tf } from "#lib/i18n.svelte.js";

  const me = $derived(
    ((page.data.organizations ?? []) as MyOrganization[]).find((m) => m.worker_pid !== null)?.worker_pid ?? null,
  );

  let mine = $state<GoalView[] | null>(null);
  let hasManager = $state(true);
  let team = $state<TeamGoals["team"]>([]);
  let error = $state<string | null>(null);
  // The form for one person below the reader, which mounts only when opened.
  let openFor = $state<string | null>(null);

  // What is waiting on the reader: a manager's proposals to them, and their reports' proposals to them.
  const waiting = $derived(
    (mine ?? []).filter((v) => v.goal.status === "proposed" && v.goal.proposed_by === "manager").length +
      team.flatMap((p) => p.goals).filter((v) => v.goal.status === "proposed" && v.goal.proposed_by === "worker").length,
  );
  const withNobody = $derived(
    !hasManager && (mine ?? []).some((v) => v.goal.status === "proposed" && v.goal.proposed_by === "worker"),
  );

  const load = guardedLoad(
    () => Promise.all([workerGoals(me ?? ""), myTeamGoals()]),
    ([own, reports]) => {
      mine = own.goals;
      hasManager = own.has_manager;
      team = reports.team;
    },
    (cause) => {
      error = errorText(cause);
    },
  );

  $effect(() => {
    if (me) void load();
  });

  async function propose(body: GoalBody) {
    await proposeMyGoal(body);
    await load();
  }

  async function proposeFor(worker: string, body: GoalBody) {
    await proposeGoalFor(worker, body);
    openFor = null;
    await load();
  }
</script>

<svelte:head><title>{t("titles.myGoals")}</title></svelte:head>

<h1>{t("goals.title")}</h1>
<p class="muted">{t("goals.hint")}</p>

{#if error}<ErrorNote testid="error" prefixed message={error} />{/if}

{#if !me}
  <p class="muted" data-testid="goals-no-worker">{t("kanban.noWorker")}</p>
{:else if mine === null}
  <p>{t("common.loading")}</p>
{:else}
  {#if waiting > 0}
    <p class="panel" data-testid="goals-waiting-count"><strong>{tf("goals.waitingCount", { count: waiting })}</strong></p>
  {/if}
  {#if withNobody}<p class="muted" data-testid="goals-no-manager">{t("goals.noManager")}</p>{/if}

  <h2>{t("goals.mine")}</h2>
  {#each mine as view (view.goal.pid)}
    <GoalCard {view} party="worker" onchange={load} />
  {:else}
    <p class="muted" data-testid="goals-none">{t("goals.none")}</p>
  {/each}

  <section class="panel">
    <h2>{t("goals.propose")}</h2>
    <GoalForm workerPid={me} submitLabel={t("goals.proposeButton")} submit={propose} />
  </section>

  {#if team.length > 0}
    <h2>{t("goals.team")}</h2>
    {#each team as person (person.worker_pid)}
      <section data-testid="team-person">
        <h3>{person.display_name}</h3>
        {#each person.goals as view (view.goal.pid)}
          <GoalCard {view} party="manager" onchange={load} />
        {:else}
          <p class="muted">{t("goals.noneForPerson")}</p>
        {/each}
        <details
          data-testid="report-propose"
          open={openFor === person.worker_pid}
          ontoggle={(event) => {
            const opened = (event.currentTarget as HTMLDetailsElement).open;
            if (opened) openFor = person.worker_pid;
            else if (openFor === person.worker_pid) openFor = null;
          }}
        >
          <summary>{t("goals.proposeFor")}</summary>
          {#if openFor === person.worker_pid}
            <GoalForm
              workerPid={person.worker_pid}
              submitLabel={t("goals.proposeButton")}
              submit={(body) => proposeFor(person.worker_pid, body)}
            />
          {/if}
        </details>
      </section>
    {/each}
  {/if}
{/if}

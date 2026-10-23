<!--
  My board (`/kanban`): the signed-in person's own development steps (the skill steps of their development
  plans) as a kanban board, one column for each status. A card moves by drag or by its "Move to…" menu; the
  move is saved at once, and a move the service refuses is put back by reading the board again. Marking a
  step achieved does not change the declared level of the skill: that is a separate, evidenced act.
-->
<script lang="ts">
  import { errorText } from "#lib/write.js";
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import type { KanbanCard, KanbanColumn } from "@lilydesignsystem/svelte-kanban-board";
  import { page } from "$app/state";
  import {
    listDevelopmentPlans,
    listSkills,
    setDevelopmentItemStatus,
    type DevelopmentItem,
    type DevelopmentPlans,
  } from "#lib/api/wpm.js";
  import type { MyOrganization } from "#lib/api/types.js";
  import LilyKanban from "#lib/components/LilyKanban.svelte";
  import { t, tf, tv } from "#lib/i18n.svelte.js";

  const STATUSES: DevelopmentItem["status"][] = ["planned", "in_progress", "achieved", "abandoned"];

  // The person is the one the layout says this sign-in is: the first membership that is a worker record.
  const me = $derived(
    ((page.data.organizations ?? []) as MyOrganization[]).find((m) => m.worker_pid !== null)?.worker_pid ?? null,
  );

  let plans = $state<DevelopmentPlans["plans"] | null>(null);
  let skills = $state<Record<string, string>>({});
  let error = $state<string | null>(null);
  let notice = $state<string | null>(null);
  // Why a move was refused. Kept apart from `error` (the board could not be read): reading the board again,
  // which puts the card back, must not hide the reason.
  let moveError = $state<string | null>(null);

  const message = (cause: unknown) => (errorText(cause));

  async function load(worker: string) {
    try {
      const [loaded, catalogue] = await Promise.all([listDevelopmentPlans(worker), listSkills()]);
      plans = loaded.plans;
      skills = Object.fromEntries(catalogue.map((s) => [s.pid, s.name]));
      error = null;
    } catch (cause) {
      error = message(cause);
    }
  }

  $effect(() => {
    if (me) void load(me);
  });

  const columns = $derived<KanbanColumn[]>(STATUSES.map((s) => ({ id: s, title: tv(s) })));
  const items = $derived((plans ?? []).flatMap((p) => p.items));
  const cards = $derived<KanbanCard[]>(
    items.map((i) => ({
      id: i.pid,
      columnId: i.status,
      title: `${skills[i.skill_pid] ?? t("kanban.unknownSkill")}: ${i.current_level} → ${i.target_level}`,
    })),
  );

  async function onMove(cardId: string, toColumnId: string) {
    notice = null;
    moveError = null;
    try {
      await setDevelopmentItemStatus(cardId, toColumnId as DevelopmentItem["status"]);
      if (toColumnId === "achieved") notice = t("kanban.achievedNote");
    } catch (cause) {
      moveError = message(cause);
    }
    // The service is the truth: read the board again, so a refused move is put back.
    if (me) await load(me);
  }
</script>

<svelte:head><title>{t("titles.kanban")}</title></svelte:head>

<h1>{t("kanban.title")}</h1>
<p class="muted">{t("kanban.hint")}</p>

{#if !me}
  <p class="muted" data-testid="kanban-no-worker">{t("kanban.noWorker")}</p>
{:else if error}
  <ErrorNote testid="error" prefixed message={error} />
{:else if plans === null}
  <p>{t("common.loading")}</p>
{:else if items.length === 0}
  <p class="muted" data-testid="kanban-empty">{t("kanban.empty")}</p>
{:else}
  <p class="muted" data-testid="kanban-count">{tf("kanban.steps", { count: items.length })}</p>
  <div data-testid="kanban-board">
    <LilyKanban label={t("kanban.title")} {columns} {cards} {onMove} />
  </div>
  {#if moveError}
    <ErrorNote testid="move-error" message={moveError} />
  {/if}
  {#if notice}<p class="muted" role="status" data-testid="kanban-notice">{notice}</p>{/if}
{/if}

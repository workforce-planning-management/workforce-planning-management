<script lang="ts">
  import { Kanban, Willow, getCardShape } from "@svar-ui/svelte-kanban";
  import type { KanbanInstanceApi } from "@svar-ui/svelte-kanban";
  import { page } from "$app/state";
  import {
    createRequisition,
    listRequisitions,
    money,
    requisitionStatus,
  } from "#lib/api/wpm.js";
  import { i18n, t, tf, tv } from "#lib/i18n.svelte.js";
  import type { Requisition } from "#lib/api/types.js";

  /** Board columns: the live pipeline statuses (cancelled stays off
   * the board; the state machine still owns which drags are legal). */
  const COLUMNS = ["draft", "open", "interviewing", "offer", "filled"] as const;

  let requisitions = $state<Requisition[] | null>(null);
  let error = $state<string | null>(null);
  let actionError = $state<string | null>(null);

  // The new-requisition form (WPM-R82): a fixed-term or contractor post needs a duration, a permanent one has none.
  const BASES = ["permanent", "fixed_term", "contractor", "intern"] as const;
  const organizationRefs = $derived((page.data.scope ?? []) as string[]);
  let org = $state("");
  let department = $state("");
  let jobTitle = $state("");
  let headcount = $state(1);
  let basis = $state<(typeof BASES)[number]>("permanent");
  let months = $state<number | null>(null);
  let programme = $state("");

  $effect(() => {
    if (!org && organizationRefs.length > 0) org = organizationRefs[0] ?? "";
  });

  async function create(event: SubmitEvent) {
    event.preventDefault();
    actionError = null;
    try {
      await createRequisition({
        organization_ref: org,
        department,
        job_title: jobTitle,
        headcount,
        basis,
        ...(basis !== "permanent" && months !== null ? { expected_duration_months: months } : {}),
        ...(programme.trim() ? { programme_ref: programme.trim() } : {}),
      });
      department = "";
      jobTitle = "";
      months = null;
      programme = "";
      await load();
    } catch (cause) {
      actionError = cause instanceof Error ? cause.message : String(cause);
    }
  }

  async function load() {
    try {
      requisitions = await listRequisitions();
    } catch (cause) {
      error = cause instanceof Error ? cause.message : String(cause);
    }
  }

  $effect(() => {
    void load();
  });

  const columns = $derived(
    COLUMNS.map((status) => ({ id: status, label: status, addCard: false })),
  );

  const cards = $derived(
    (requisitions ?? [])
      .filter((r) => r.status !== "cancelled")
      .map((r) => ({
        id: r.pid,
        label: r.job_title,
        description:
          `${r.department} · ${t("req.headcount")} ${r.headcount}` +
          (r.basis && r.basis !== "permanent"
            ? ` · ${tv(r.basis)}` +
              (r.expected_duration_months
                ? ` ${tf("req.months", { months: r.expected_duration_months })}`
                : "")
            : "") +
          (r.programme_ref ? ` · ${r.programme_ref}` : "") +
          (r.salary_min_minor !== null && r.salary_max_minor !== null
            ? ` · ${money(r.salary_min_minor, r.salary_currency, i18n.locale)}–${money(r.salary_max_minor, r.salary_currency, i18n.locale)}`
            : ""),
        status: r.status,
      })),
  );

  // Drag = a pipeline transition through the API; the service's state
  // machine refuses illegal moves (422), and the reload puts the card
  // back where the truth says it belongs.
  function init(api: KanbanInstanceApi) {
    api.on("move-card", (raw) => {
      const ev = raw as { id: string | number; column?: string | number };
      if (!ev.column) return;
      actionError = null;
      void requisitionStatus(String(ev.id), String(ev.column))
        .catch((cause) => {
          actionError = cause instanceof Error ? cause.message : String(cause);
        })
        .finally(load);
    });
  }
</script>

<h1>{t("nav.requisitions")}</h1>

<form onsubmit={create} data-testid="requisition-form">
  <h2>{t("req.new")}</h2>
  <label>
    {t("pages.planning.organization")}
    {#if organizationRefs.length > 0}
      <select data-testid="req-org" bind:value={org} required>
        {#each organizationRefs as ref (ref)}<option value={ref}>{ref}</option>{/each}
      </select>
    {:else}
      <!-- No membership to choose from (a demo service with enforcement off): the URN is typed. -->
      <input data-testid="req-org" bind:value={org} required />
    {/if}
  </label>
  <label>{t("common.department")} <input data-testid="req-department" bind:value={department} required /></label>
  <label>{t("common.jobTitle")} <input data-testid="req-title" bind:value={jobTitle} required /></label>
  <label>{t("req.headcount")} <input data-testid="req-headcount" type="number" min="1" bind:value={headcount} required /></label>
  <label>
    {t("engagement.basis")}
    <select data-testid="req-basis" bind:value={basis}>
      {#each BASES as b (b)}<option value={b}>{tv(b)}</option>{/each}
    </select>
  </label>
  {#if basis !== "permanent"}
    <label>
      {t("req.duration")}
      <input data-testid="req-months" type="number" min="1" max="120" bind:value={months} required={basis !== "intern"} />
    </label>
  {/if}
  <label>{t("plan.programmeOptional")} <input data-testid="req-programme" bind:value={programme} /></label>
  <button type="submit" data-testid="req-create">{t("req.create")}</button>
</form>

{#if error}
  <p class="error" data-testid="error">{t("common.error")}: {error}</p>
{:else if requisitions === null}
  <p>{t("common.loading")}</p>
{:else}
  {#if actionError}
    <p class="error" data-testid="action-error">{actionError}</p>
  {/if}
  <div class="board-wrap" data-testid="requisition-board">
    <Willow>
      <Kanban
        {cards}
        {columns}
        columnAccessor="status"
        card={{ ...getCardShape(), menu: false }}
        {init}
      />
    </Willow>
  </div>
{/if}

<style>
  .board-wrap {
    height: 620px;
    overflow-x: auto;
  }
</style>

<script lang="ts">
  import { guardedLoad } from "#lib/loader.svelte.js";
  import { errorText } from "#lib/write.js";
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import LilyKanban from "#lib/components/LilyKanban.svelte";
  import LoadMore from "#lib/components/LoadMore.svelte";
  import { page } from "$app/state";
  import {
    createRequisition,
    listRequisitionsPage,
    money,
    requisitionStatus,
  } from "#lib/api/wpm.js";
  import { i18n, t, tf, tv } from "#lib/i18n.svelte.js";
  import type { Requisition } from "#lib/api/types.js";

  /** Board columns: the live pipeline statuses (cancelled stays off
   * the board; the state machine still owns which drags are legal). */
  const COLUMNS = ["draft", "open", "interviewing", "offer", "filled"] as const;

  /** The service serves a page at a time (500 at most); the board shows what has been read so far. */
  const PAGE = 500;
  let requisitions = $state<Requisition[] | null>(null);
  let total = $state<number | null>(null);
  let more = $state(false);
  let loadingMore = $state(false);
  /** How many pages the board holds, so a reload after a move or a create keeps them all. */
  let pages = 1;
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
      actionError = errorText(cause);
    }
  }

  const load = guardedLoad(
    async () => {
      const rows: Requisition[] = [];
      let last = await listRequisitionsPage({ limit: PAGE });
      rows.push(...last.items);
      for (let n = 1; n < pages && last.items.length === PAGE; n++) {
        last = await listRequisitionsPage({ limit: PAGE, offset: n * PAGE });
        rows.push(...last.items);
      }
      return { rows, total: last.total, full: last.items.length === PAGE };
    },
    (found) => {
      requisitions = found.rows;
      total = found.total;
      more = found.total !== null ? found.rows.length < found.total : found.full;
    },
    (cause) => {
      error = errorText(cause);
    },
  );

  async function showMore() {
    loadingMore = true;
    actionError = null;
    try {
      const next = await listRequisitionsPage({ limit: PAGE, offset: requisitions?.length ?? 0 });
      requisitions = [...(requisitions ?? []), ...next.items];
      total = next.total;
      pages += 1;
      more = next.total !== null ? requisitions.length < next.total : next.items.length === PAGE;
    } catch (cause) {
      actionError = errorText(cause);
    } finally {
      loadingMore = false;
    }
  }

  $effect(() => {
    void load();
  });

  const columns = $derived(
    COLUMNS.map((status) => ({ id: status, title: tv(status) })),
  );

  const cards = $derived(
    (requisitions ?? [])
      .filter((r) => r.status !== "cancelled")
      .map((r) => ({
        id: r.pid,
        title: r.job_title,
        subtitle:
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
        columnId: r.status,
      })),
  );

  // Drag = a pipeline transition through the API; the service's state
  // machine refuses illegal moves (422), and the reload puts the card
  // back where the truth says it belongs.
  function onMove(cardId: string, toColumnId: string) {
    actionError = null;
    void requisitionStatus(cardId, toColumnId)
      .catch((cause) => {
        actionError = errorText(cause);
      })
      .finally(load);
  }
</script>

<svelte:head><title>{page.data.title}</title></svelte:head>

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
  <ErrorNote testid="error" prefixed message={error} />
{:else if requisitions === null}
  <p>{t("common.loading")}</p>
{:else}
  {#if actionError}
    <ErrorNote testid="action-error" message={actionError} />
  {/if}
  <div data-testid="requisition-board">
    <LilyKanban label={t("nav.requisitions")} {columns} {cards} {onMove} />
  </div>
  <LoadMore shown={requisitions.length} {total} {more} loading={loadingMore} onmore={showMore} />
{/if}

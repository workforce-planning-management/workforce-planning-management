<!--
  Positions (`/positions`): budgeted seats that exist whether or not anyone holds them (WPM-R152). The establishment
  per cost centre counts approved, open, filled and frozen positions; over-establishment is never netted against a
  vacancy elsewhere, an unknown holder FTE is shown as unknown, and a figure under the floor is withheld. A position
  carries a budget in FTE and no pay. Who holds a seat is HR's to see; other readers see whether it is held.
  Linking a holder or a requisition is done through the API for now.
-->
<script lang="ts">
  import { formatCentiFte } from "#lib/locale-format.js";
  import { guardedLoad } from "#lib/loader.svelte.js";
  import { errorText } from "#lib/write.js";
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import {
    createPosition,
    listPositions,
    movePosition,
    positionEstablishment,
    POSITION_STATES,
    type CostCentre,
    type Position,
    type PositionFte,
  } from "#lib/api/wpm.js";
  import { page } from "$app/state";
  import { t } from "#lib/i18n.svelte.js";
  import DataTable from "#lib/components/DataTable.svelte";
  import type { DataGridColumn, DataGridRow } from "@lilydesignsystem/svelte-data-grid";

  const organizations = $derived((page.data.scope ?? []) as string[]);
  let organization = $state<string>("");
  $effect(() => {
    if (!organization && organizations.length > 0) organization = organizations[0] ?? "";
  });

  let positions = $state<Position[]>([]);
  let centres = $state<CostCentre[]>([]);
  let error = $state<string | null>(null);
  let costCentre = $state("");
  let title = $state("");
  let budget = $state(100);
  let funding = $state<string>("core");
  let fundingEnds = $state("");
  let programme = $state("");
  const FUNDING_KINDS = ["core", "time_limited", "external", "programme"] as const;

  const fte = (f: PositionFte) =>
    f.status === "known"
      ? formatCentiFte(f.percent)
      : f.status === "withheld"
        ? t("pages.positions.withheld")
        : t("pages.positions.unknown");

  const centreColumns = $derived<DataGridColumn[]>([
    { id: "cost_centre", header: t("pages.positions.cost_centre"), sortable: true, rowHeader: true },
    { id: "positions", header: t("pages.positions.positions"), sortable: true, align: "end" },
    { id: "budgeted", header: t("pages.positions.budgeted"), align: "end", accessor: (r) => fte(r.budgeted as PositionFte) },
    { id: "held", header: t("pages.positions.held_fte"), align: "end", accessor: (r) => fte(r.held as PositionFte) },
    { id: "vacant", header: t("pages.positions.vacant"), align: "end", accessor: (r) => fte(r.vacant as PositionFte) },
    { id: "over", header: t("pages.positions.over"), align: "end", accessor: (r) => fte(r.over as PositionFte) },
    { id: "frozen_vacant", header: t("pages.positions.frozen_vacant"), align: "end", accessor: (r) => fte(r.frozen_vacant as PositionFte) },
  ]);

  const seatColumns = $derived<DataGridColumn[]>([
    { id: "cost_centre", header: t("pages.positions.cost_centre"), sortable: true, rowHeader: true },
    { id: "title", header: t("pages.positions.title"), sortable: true },
    { id: "budget_fte_percent", header: t("pages.positions.budgeted"), sortable: true, align: "end", format: (v) => formatCentiFte(v as number) },
    { id: "funding_kind", header: t("pages.positions.funding"), sortable: true, accessor: (r) => t(`values.${r.funding_kind as string}` as never) },
    { id: "state", header: t("pages.positions.state"), sortable: true, accessor: (r) => t(`values.${r.state as string}` as never) },
    { id: "held", header: t("pages.positions.holder"), accessor: (r) => (r.holder_worker_pid ? String(r.holder_worker_pid) : r.held ? t("pages.positions.held") : t("pages.positions.not_held")) },
    { id: "move", header: t("pages.positions.move"), cell: moveCell },
  ]);

  const load = guardedLoad(
    async () =>
      organization
        ? await Promise.all([listPositions(organization), positionEstablishment(organization)])
        : null,
    (found) => {
      positions = found?.[0].positions ?? [];
      centres = found?.[1].cost_centres ?? [];
    },
    (cause) => {
      error = errorText(cause);
    },
  );
  $effect(() => {
    void organization;
    void load();
  });

  let moving = $state<Record<string, string>>({});
  async function move(pid: string) {
    const to = moving[pid];
    if (!to) return;
    error = null;
    try {
      await movePosition(pid, to);
      await load();
    } catch (cause) {
      error = errorText(cause);
    }
  }

  async function plan(event: SubmitEvent) {
    event.preventDefault();
    error = null;
    try {
      await createPosition({
        organization_ref: organization,
        cost_centre: costCentre.trim(),
        title: title.trim(),
        budget_fte_percent: budget,
        funding_kind: funding,
        ...(fundingEnds ? { funding_ends_on: fundingEnds } : {}),
        ...(funding === "programme" && programme.trim() ? { programme_ref: programme.trim() } : {}),
      });
      title = "";
      await load();
    } catch (cause) {
      error = errorText(cause);
    }
  }
</script>

<svelte:head><title>{t("pages.positions.page_title")}</title></svelte:head>

{#snippet moveCell({ row }: { row: DataGridRow })}
  {#if row.state !== "abolished"}
    <select
      aria-label={t("pages.positions.move_to")}
      value={moving[String(row.pid)] ?? ""}
      onchange={(e) => (moving[String(row.pid)] = e.currentTarget.value)}
    >
      <option value="">{t("pages.positions.move_to")}</option>
      {#each POSITION_STATES.filter((s) => s !== row.state) as s (s)}<option value={s}>{t(`values.${s}` as never)}</option>{/each}
    </select>
    <button type="button" onclick={() => void move(String(row.pid))}>{t("pages.positions.move")}</button>
  {/if}
{/snippet}

<h1>{t("nav.positions")}</h1>
<p class="muted">{t("pages.positions.intro")}</p>
{#if error}<ErrorNote testid="error" message={error} />{/if}
{#if organizations.length === 0}
  <p class="muted">{t("org.noOrganizations")}</p>
{:else}
  <p>
    <label>
      {t("pages.positions.organization")}
      <select bind:value={organization} data-testid="position-organization">
        {#each organizations as o (o)}<option value={o}>{o}</option>{/each}
      </select>
    </label>
  </p>

  <h2>{t("pages.positions.establishment")}</h2>
  <p class="muted">{t("pages.positions.establishment_note")}</p>
  {#if centres.length === 0}
    <p class="muted" data-testid="establishment">{t("pages.positions.no_positions")}</p>
  {:else}
    <DataTable
      testid="establishment"
      label={t("pages.positions.establishment")}
      columns={centreColumns}
      rows={centres as unknown as DataGridRow[]}
      rowId={(row) => String(row.cost_centre)}
      pageSize={25}
    />
  {/if}

  <h2>{t("pages.positions.positions")}</h2>
  {#if positions.length === 0}
    <p class="muted" data-testid="position-list">{t("pages.positions.no_positions")}</p>
  {:else}
    <DataTable
      testid="position-list"
      label={t("pages.positions.positions")}
      columns={seatColumns}
      rows={positions as unknown as DataGridRow[]}
      rowId={(row) => String(row.pid)}
      pageSize={25}
    />
  {/if}

  <h2>{t("pages.positions.plan_a_seat")}</h2>
  <form onsubmit={plan}>
    <label>{t("pages.positions.cost_centre")} <input bind:value={costCentre} required maxlength="80" /></label>
    <label>{t("pages.positions.title")} <input bind:value={title} required maxlength="120" /></label>
    <label>{t("pages.positions.budget_percent")} <input type="number" bind:value={budget} min="1" max="100" required /></label>
    <label>
      {t("pages.positions.funding")}
      <select bind:value={funding}>{#each FUNDING_KINDS as k (k)}<option value={k}>{t(`values.${k}` as never)}</option>{/each}</select>
    </label>
    {#if funding === "time_limited" || funding === "external"}
      <label>{t("pages.positions.funding_ends")} <input type="date" bind:value={fundingEnds} required /></label>
    {/if}
    {#if funding === "programme"}
      <label>{t("pages.positions.programme")} <input bind:value={programme} required /></label>
    {/if}
    <button type="submit">{t("pages.positions.plan")}</button>
  </form>
{/if}

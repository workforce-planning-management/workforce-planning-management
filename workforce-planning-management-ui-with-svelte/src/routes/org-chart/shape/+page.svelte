<!--
  The organization now and as a plan intends it (`/org-chart/shape`, WPM-R153): read-only. Pick a plan to see layers, span of
  control and each role added, removed, moved or resized against the live structure. It is by role and names no one; a count of
  people under the floor is withheld together with the change that would give it back. The plan's target is set through the
  API for now.
-->
<script lang="ts">
  import { formatDecimal } from "#lib/locale-format.js";
  import { guardedLoad } from "#lib/loader.svelte.js";
  import { errorText } from "#lib/write.js";
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import { listWorkforcePlans, organizationDiff, type OrganizationDiff, type ShapeRole } from "#lib/api/wpm.js";
  import { t, tf, l } from "#lib/i18n.svelte.js";
  import DataTable from "#lib/components/DataTable.svelte";
  import type { DataGridColumn, DataGridRow } from "@lilydesignsystem/svelte-data-grid";

  let plans = $state<Awaited<ReturnType<typeof listWorkforcePlans>>>([]);
  let chosen = $state("");
  let shown = $state<OrganizationDiff | null>(null);
  let error = $state<string | null>(null);

  const loadPlans = guardedLoad(
    () => listWorkforcePlans(),
    (found) => {
      plans = found;
      if (!chosen && found.length > 0) chosen = found[0]?.pid ?? "";
    },
    (cause) => {
      error = errorText(cause);
    },
  );
  void loadPlans();

  const loadDiff = guardedLoad(
    async () => (chosen ? await organizationDiff(chosen) : null),
    (found) => {
      shown = found;
    },
    (cause) => {
      error = errorText(cause);
    },
  );
  $effect(() => {
    void chosen;
    shown = null;
    void loadDiff();
  });

  const span = (mean: number | null) => (mean === null ? "—" : formatDecimal(Number(`${mean}e-2`), 2));
  const changeWord = (c: ShapeRole["change"]) => t(`pages.orgshape.change_${c}` as never);
  const roleColumns = $derived<DataGridColumn[]>([
    { id: "title", header: t("pages.orgshape.role"), sortable: true, rowHeader: true },
    {
      id: "current",
      header: t("pages.orgshape.now"),
      sortable: true,
      align: "end",
      accessor: (r) => {
        const c = r.current as ShapeRole["current"];
        return c.status === "shown" ? c.value : t("pages.orgshape.withheld");
      },
    },
    { id: "target", header: t("pages.orgshape.planned"), sortable: true, align: "end" },
    { id: "change", header: t("pages.orgshape.change"), sortable: true, accessor: (r) => changeWord(r.change as ShapeRole["change"]) },
    {
      id: "moved",
      header: t("pages.orgshape.departments"),
      accessor: (r) =>
        r.moved
          ? tf("pages.orgshape.moved_from_to", {
              from: (r.current_departments as string[]).join(", "),
              to: (r.target_departments as string[]).join(", "),
            })
          : "",
    },
  ]);
</script>

<svelte:head><title>{t("pages.orgshape.page_title")}</title></svelte:head>

<h1>{t("pages.orgshape.heading")}</h1>
<p class="muted">{t("pages.orgshape.intro")}</p>
<p><a href={l("/org-chart")}>{t("nav.orgChart")}</a></p>
{#if error}<ErrorNote testid="error" message={error} />{/if}

{#if plans.length === 0}
  <p class="muted" data-testid="shape-none">{t("pages.orgshape.no_plans")}</p>
{:else}
  <p>
    <label>
      {t("pages.orgshape.plan")}
      <select bind:value={chosen} data-testid="shape-plan">
        {#each plans as p (p.pid)}<option value={p.pid}>{p.name}</option>{/each}
      </select>
    </label>
  </p>

  {#if shown && !shown.target_set}
    <p class="muted" data-testid="shape-no-target">{t("pages.orgshape.no_target")}</p>
  {:else if shown && shown.target_set}
    <table data-testid="shape-sides">
      <thead>
        <tr>
          <th></th>
          <th>{t("pages.orgshape.now")}</th>
          <th>{t("pages.orgshape.planned")}</th>
        </tr>
      </thead>
      <tbody>
        <tr><th scope="row">{t("pages.orgshape.layers")}</th><td>{shown.diff.current.layers}</td><td>{shown.diff.target.layers}</td></tr>
        <tr><th scope="row">{t("pages.orgshape.headcount")}</th><td>{shown.diff.current.headcount}</td><td>{shown.diff.target.headcount}</td></tr>
        <tr><th scope="row">{t("pages.orgshape.manager_seats")}</th><td>{shown.diff.current.span.managers}</td><td>{shown.diff.target.span.managers}</td></tr>
        <tr><th scope="row">{t("pages.orgshape.mean_span")}</th><td>{span(shown.diff.current.span.mean_hundredths)}</td><td>{span(shown.diff.target.span.mean_hundredths)}</td></tr>
        <tr><th scope="row">{t("pages.orgshape.widest_span")}</th><td>{shown.diff.current.span.max}</td><td>{shown.diff.target.span.max}</td></tr>
      </tbody>
    </table>
    <h2>{t("pages.orgshape.roles")}</h2>
    <p class="muted">{tf("pages.orgshape.floor_note", { floor: shown.small_group_floor })}</p>
    <DataTable
      testid="shape-roles"
      label={t("pages.orgshape.roles")}
      columns={roleColumns}
      rows={shown.diff.roles as unknown as DataGridRow[]}
      rowId={(row) => String(row.title)}
      pageSize={50}
    />
  {/if}
{/if}

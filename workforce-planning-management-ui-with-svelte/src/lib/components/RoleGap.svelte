<!--
  "Compare to a role": one worker's declared proficiency against a role
  profile's requirements. Compares declarations; `undeclared` is unknown, not
  below. A development conversation starter, never a selection decision.
-->
<script lang="ts">
  import { errorText } from "#lib/write.js";
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import { t, tf, tv } from "#lib/i18n.svelte.js";
  import { listRoleProfiles, workerRoleGap } from "#lib/api/wpm.js";
  import { percentWithWorkings } from "#lib/format.js";
  import DataTable from "#lib/components/DataTable.svelte";
  import type {
    DataGridColumn,
    DataGridRow,
  } from "@lilydesignsystem/svelte-data-grid";

  let { workerPid }: { workerPid: string } = $props();

  type Profiles = Awaited<ReturnType<typeof listRoleProfiles>>;
  type Gap = Awaited<ReturnType<typeof workerRoleGap>>;

  let profiles = $state<Profiles>([]);
  let roleId = $state("");
  let gap = $state<Gap | null>(null);
  let error = $state<string | null>(null);

  $effect(() => {
    void (async () => {
      try {
        profiles = await listRoleProfiles();
      } catch (cause) {
        error = errorText(cause);
      }
    })();
  });

  async function compare(id: string) {
    roleId = id;
    gap = null;
    error = null;
    if (!id) return;
    try {
      gap = await workerRoleGap(workerPid, id);
    } catch (cause) {
      error = errorText(cause);
    }
  }

  type Requirement = Gap["requirements"][number];

  const columns = $derived<DataGridColumn[]>([
    { id: "skill", header: t("comp.roleGap.skill"), sortable: true },
    {
      id: "importance",
      header: t("comp.roleGap.importance"),
      sortable: true,
      format: (v) => tv(String(v)),
    },
    { id: "min_proficiency", header: t("comp.roleGap.needs"), sortable: true, align: "end" },
    { id: "declared", header: t("comp.roleGap.declared"), sortable: true, align: "end" },
    {
      id: "grade",
      header: t("comp.roleGap.status"),
      sortable: true,
      cellClass: ({ row }) => ((row as unknown as Requirement).grade !== "met" ? "cx-warn" : ""),
      cell: gradeCell,
    },
  ]);
</script>

{#snippet gradeCell({ row }: { row: DataGridRow })}
  {@const r = row as unknown as Requirement}
  {tv(r.grade)}{#if r.shortfall !== null} (−{r.shortfall}){/if}
{/snippet}

{#if profiles.length > 0}
  <div class="panel" data-testid="role-gap">
    <h3>{t("comp.roleGap.compare_to_a_role")}</h3>
    <label>
      {t("comp.roleGap.role")}
      <select
        data-testid="role-gap-select"
        value={roleId}
        onchange={(event) => void compare(event.currentTarget.value)}
      >
        <option value="">{t("comp.roleGap.choose")}</option>
        {#each profiles as p (p.pid)}<option value={p.pid}>{p.job_title}</option>{/each}
      </select>
    </label>
    {#if error}<ErrorNote message={error} />{/if}
    {#if gap}
      <p class="muted">{gap.derivation}</p>
      <p>
        {tf("comp.roleGap.critical_requirements_met_all", { critical_met: percentWithWorkings(gap.critical_met), all_met: percentWithWorkings(
          gap.all_met,
        ) })}
      </p>
      <DataTable
        label={t("comp.roleGap.compare_to_a_role")}
        columns={columns}
        rows={gap.requirements as unknown as DataGridRow[]}
        rowId={(row) => String(row.skill_pid)}
      />
    {/if}
  </div>
{/if}

<style>
  :global(td.cx-warn) { color: #b45309; font-weight: 600; }
</style>

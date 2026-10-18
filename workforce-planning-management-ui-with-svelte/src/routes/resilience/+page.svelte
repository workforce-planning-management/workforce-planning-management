<!--
  Resilience (`/resilience`, WPM-R61, WPM-R64): the critical skills and how many people hold, back up
  and could succeed to each, with the number of single points of failure; and the senior hours a
  month that oversight commitments absorb, by role. Counts and hours only: nobody is named, and the
  skills are read for the chosen organization (a named read is a separate, audited route).
-->
<script lang="ts">
  import {
    governanceLoad,
    keyPersonRisk,
    type GovernanceLoad,
    type KeyPersonView,
  } from "#lib/api/wpm.js";
  import OrgPicker from "#lib/components/OrgPicker.svelte";
  import { t, tf, tv } from "#lib/i18n.svelte.js";
  import DataTable from "#lib/components/DataTable.svelte";
  import type {
    DataGridColumn,
    DataGridRow,
  } from "@lilydesignsystem/svelte-data-grid";

  let org = $state("");
  let people = $state<KeyPersonView | null>(null);
  let load = $state<GovernanceLoad | null>(null);
  let peopleError = $state<string | null>(null);
  let loadError = $state<string | null>(null);

  const message = (cause: unknown) => (cause instanceof Error ? cause.message : String(cause));

  $effect(() => {
    const chosen = org;
    if (!chosen) return;
    let current = true;
    people = null;
    peopleError = null;
    void keyPersonRisk(chosen)
      .then((v) => current && (people = v))
      .catch((cause) => current && (peopleError = message(cause)));
    return () => {
      current = false;
    };
  });

  $effect(() => {
    const chosen = org;
    if (!chosen) return;
    let current = true;
    load = null;
    loadError = null;
    void governanceLoad(chosen)
      .then((v) => current && (load = v))
      .catch((cause) => current && (loadError = message(cause)));
    return () => {
      current = false;
    };
  });

  // Skills across the whole scope (the department rows repeat them), worst first.
  const skills = $derived((people?.skills ?? []).filter((s) => !s.department));
  const hours = (n: number) => n.toFixed(1);

  const skillColumns = $derived<DataGridColumn[]>([
    { id: "skill", header: t("res.skill"), sortable: true, rowHeader: true },
    { id: "minimum_proficiency", header: t("res.minimum"), sortable: true, align: "end" },
    { id: "holders", header: t("res.holders"), sortable: true, align: "end" },
    { id: "backups", header: t("res.backups"), sortable: true, align: "end" },
    { id: "ready_successors", header: t("res.successors"), sortable: true, align: "end" },
    { id: "status", header: t("common.status"), sortable: true, accessor: (r) => tv(String(r.status)), cell: statusCell },
  ]);
  const roleColumns = $derived<DataGridColumn[]>([
    { id: "role", header: t("res.role"), sortable: true, rowHeader: true },
    { id: "senior", header: t("res.senior"), sortable: true, accessor: (r) => (r.senior ? "✓" : "—") },
    {
      id: "hours_per_month",
      header: t("res.hoursMonth"),
      sortable: true,
      align: "end",
      format: (v) => (typeof v === "number" ? hours(v) : "—"),
    },
  ]);
</script>

<svelte:head><title>{t("titles.resilience")}</title></svelte:head>

{#snippet statusCell({ row }: { row: DataGridRow })}
  <span
    class="chip"
    class:danger={row.status === "single_point_of_failure" || row.status === "no_holder"}
  >
    {tv(String(row.status))}
  </span>
{/snippet}

<h1>{t("res.title")}</h1>
<p class="muted">{t("res.hint")}</p>

<div class="form"><OrgPicker bind:value={org} /></div>

<section class="panel" data-testid="key-person">
  <h2>{t("res.keyPerson")}</h2>
  {#if !org}
    <p class="muted">{t("common.noOrganization")}</p>
  {:else if peopleError}
    <p class="error" data-testid="error">{t("common.error")}: {peopleError}</p>
  {:else if people === null}
    <p>{t("common.loading")}</p>
  {:else}
    <p>
      <strong data-testid="spof-count">
        {tf("res.spof", { count: people.single_points_of_failure })}
      </strong>
    </p>
    {#if skills.length === 0}
      <p class="muted" data-testid="no-skills">{t("res.noSkills")}</p>
    {:else}
      <DataTable
        testid="key-person-table"
        label={t("res.keyPerson")}
        columns={skillColumns}
        rows={skills as unknown as DataGridRow[]}
        rowId={(row) => String(row.skill)}
        pageSize={25}
      />
    {/if}
    <details>
      <summary>{t("cap.basis")}</summary>
      <p class="muted">{t("res.basisKeyPerson")}</p>
      <p class="muted small">{t("common.fromService")}: {people.basis}</p>
    </details>
  {/if}
</section>

<section class="panel" data-testid="governance">
  <h2>{t("res.governance")}</h2>
  {#if !org}
    <p class="muted">{t("common.noOrganization")}</p>
  {:else if loadError}
    <p class="error" data-testid="error">{t("common.error")}: {loadError}</p>
  {:else if load === null}
    <p>{t("common.loading")}</p>
  {:else if load.commitments === 0}
    <p class="muted" data-testid="no-commitments">{t("res.noCommitments")}</p>
  {:else}
    <p>
      <strong data-testid="senior-hours">
        {tf("res.seniorHours", { hours: hours(load.senior_hours_per_month) })}
      </strong>
      <span class="muted">· {tf("res.allHours", { hours: hours(load.all_hours_per_month) })}</span>
    </p>
    <DataTable
      testid="governance-table"
      label={t("res.governance")}
      columns={roleColumns}
      rows={load.roles as unknown as DataGridRow[]}
      rowId={(row) => String(row.role)}
    />
    <details>
      <summary>{t("cap.basis")}</summary>
      <p class="muted">{t("res.basisGovernance")}</p>
      <p class="muted small">{t("common.fromService")}: {load.basis}</p>
    </details>
  {/if}
</section>

<style>
  .small {
    font-size: 0.82rem;
  }
</style>

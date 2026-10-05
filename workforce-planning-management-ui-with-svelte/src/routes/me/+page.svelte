<!--
  My profile (`/me`): choose your current role in the UK GDAD Profession
  Capability Framework and in ESCO, and select the skills you have in each at
  your own level. Your worker record is found from your organization
  memberships; selected skills become ordinary skill declarations.
-->
<script lang="ts">
  import { page } from "$app/state";
  import { listSelectableFrameworks } from "#lib/api/wpm.js";
  import Backups from "#lib/components/Backups.svelte";
  import OnCall from "#lib/components/OnCall.svelte";
  import DottedLinePanel from "#lib/components/DottedLinePanel.svelte";
  import EmergencyContacts from "#lib/components/EmergencyContacts.svelte";
  import GroupsPanel from "#lib/components/GroupsPanel.svelte";
  import TeamAspirations from "#lib/components/TeamAspirations.svelte";
  import Aspirations from "#lib/components/Aspirations.svelte";
  import CareerHistory from "#lib/components/CareerHistory.svelte";
  import FrameworkRolePanel from "#lib/components/FrameworkRolePanel.svelte";
  import { t } from "#lib/i18n.svelte.js";

  type Org = { pid: string; organization_ref: string; worker_pid: string | null; employed: boolean };
  const memberships = $derived(((page.data.organizations ?? []) as Org[]).filter((m) => m.worker_pid));

  let chosen = $state("");
  const workerPid = $derived(chosen || memberships[0]?.worker_pid || "");

  let frameworks = $state<Awaited<ReturnType<typeof listSelectableFrameworks>>>([]);
  let error = $state<string | null>(null);
  $effect(() => {
    void (async () => {
      try {
        frameworks = await listSelectableFrameworks();
      } catch (cause) {
        error = cause instanceof Error ? cause.message : String(cause);
      }
    })();
  });
  const available = (slug: string) => frameworks.find((f) => f.slug === slug);
</script>

<svelte:head><title>{t("nav.me")} — WPM</title></svelte:head>

<h1>{t("nav.me")}</h1>
{#if error}<p class="error" data-testid="error">{error}</p>{/if}

{#if memberships.length === 0}
  <p class="muted" data-testid="no-worker">
    You do not have a worker record in any organization, so there is no role to select.
  </p>
{:else}
  {#if memberships.length > 1}
    <p>
      <label>
        Organization
        <select bind:value={chosen}>
          {#each memberships as m (m.pid)}<option value={m.worker_pid}>{m.organization_ref}</option>{/each}
        </select>
      </label>
    </p>
  {/if}
  <p class="muted">
    Say what you do now in each framework, then pick the skills you have and your own level. This is your
    own description, not an assessment.
  </p>
  {#if available("uk-gdad-pcf")?.available}
    <FrameworkRolePanel {workerPid} framework="uk-gdad-pcf" title="UK Government Digital and Data Profession Capability Framework" />
  {/if}
  {#if available("esco")?.available}
    <FrameworkRolePanel {workerPid} framework="esco" title="ESCO — European Skills, Competences, Qualifications and Occupations" />
  {/if}
  <CareerHistory {workerPid} />
  <Aspirations {workerPid} />
  <EmergencyContacts {workerPid} />
  <Backups {workerPid} />
  <OnCall {workerPid} />
  <DottedLinePanel {workerPid} />
  <GroupsPanel {workerPid} />
  <TeamAspirations {workerPid} />
  {#if frameworks.length > 0 && !frameworks.some((f) => f.available)}
    <p class="muted">No framework has been loaded yet; ask an administrator to import one.</p>
  {/if}
{/if}

<!--
  My profile (`/me`): choose your current role in the UK GDAD Profession
  Capability Framework and in ESCO, and select the skills you have in each at
  your own level. Your worker record is found from your organization
  memberships; selected skills become ordinary skill declarations.
-->
<script lang="ts">
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import { page } from "$app/state";
  import { listSelectableFrameworks } from "#lib/api/wpm.js";
  import Backups from "#lib/components/Backups.svelte";
  import OnCall from "#lib/components/OnCall.svelte";
  import SkillGaps from "#lib/components/SkillGaps.svelte";
  import DottedLinePanel from "#lib/components/DottedLinePanel.svelte";
  import ExpenseClaims from "#lib/components/ExpenseClaims.svelte";
  import MyPolicies from "#lib/components/MyPolicies.svelte";
  import EquipmentPanel from "#lib/components/EquipmentPanel.svelte";
  import MyCredentials from "#lib/components/MyCredentials.svelte";
  import PayPosition from "#lib/components/PayPosition.svelte";
  import JobLevel from "#lib/components/JobLevel.svelte";
  import EmergencyContacts from "#lib/components/EmergencyContacts.svelte";
  import GroupsPanel from "#lib/components/GroupsPanel.svelte";
  import TeamAspirations from "#lib/components/TeamAspirations.svelte";
  import Aspirations from "#lib/components/Aspirations.svelte";
  import CareerHistory from "#lib/components/CareerHistory.svelte";
  import FrameworkRolePanel from "#lib/components/FrameworkRolePanel.svelte";
  import { l, t, tf } from "#lib/i18n.svelte.js";
  import { createLoader } from "#lib/loader.svelte.js";

  type Org = { pid: string; organization_ref: string; worker_pid: string | null; employed: boolean };
  const memberships = $derived(((page.data.organizations ?? []) as Org[]).filter((m) => m.worker_pid));

  let chosen = $state("");
  const workerPid = $derived(chosen || memberships[0]?.worker_pid || "");

  const loader = createLoader(() => listSelectableFrameworks());
  const frameworks = $derived(loader.data ?? []);
  const error = $derived(loader.error);
  $effect(() => {
    void loader.load();
  });
  const available = (slug: string) => frameworks.find((f) => f.slug === slug);
</script>

<svelte:head><title>{tf("pages.me.page_title", { me: t("nav.me") })}</title></svelte:head>

<h1>{t("nav.me")}</h1>
<p><a href={l("/me/details")} data-testid="my-details-link">{t("details.link")}</a></p>
{#if error}<ErrorNote testid="error" message={error} />{/if}

{#if memberships.length === 0}
  <p class="muted" data-testid="no-worker">
    {t("pages.me.you_do_not_have_a_worker_record_in")}
  </p>
{:else}
  {#if memberships.length > 1}
    <p>
      <label>
        {t("pages.me.organization")}
        <select bind:value={chosen}>
          {#each memberships as m (m.pid)}<option value={m.worker_pid}>{m.organization_ref}</option>{/each}
        </select>
      </label>
    </p>
  {/if}
  <p class="muted">
    {t("pages.me.say_what_you_do_now_in_each")}
  </p>
  {#if available("uk-gdad-pcf")?.available}
    <FrameworkRolePanel {workerPid} framework="uk-gdad-pcf" title={t("pages.me.uk_government_digital_and_data")} />
  {/if}
  {#if available("esco")?.available}
    <FrameworkRolePanel {workerPid} framework="esco" title={t("pages.me.esco_european_skills_competences")} />
  {/if}
  <CareerHistory {workerPid} />
  <Aspirations {workerPid} />
  <SkillGaps {workerPid} />
  <JobLevel {workerPid} />
  <PayPosition {workerPid} />
  <MyPolicies {workerPid} />
  <EquipmentPanel {workerPid} organizationRef={memberships.find((m) => m.worker_pid === workerPid)?.organization_ref ?? ""} />
  <MyCredentials {workerPid} />
  <ExpenseClaims {workerPid} />
  <EmergencyContacts {workerPid} />
  <Backups {workerPid} />
  <OnCall {workerPid} />
  <DottedLinePanel {workerPid} />
  <GroupsPanel {workerPid} />
  <TeamAspirations {workerPid} />
  {#if frameworks.length > 0 && !frameworks.some((f) => f.available)}
    <p class="muted">{t("pages.me.no_framework_has_been_loaded_yet")}</p>
  {/if}
{/if}

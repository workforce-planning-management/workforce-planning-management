<!--
  My details (`/me/details`, WPM-R124–R130): one screen for what a worker does about themself — contact details,
  emergency contacts, time off and leave requests, flexible working requests, a resignation, equality monitoring
  and workplace health requirements. Everything is the caller's own: the person is the signed-in token, never a
  path parameter, so there is no one else's record to open from here.
-->
<script lang="ts">
  import { page } from "$app/state";
  import EmergencyContacts from "#lib/components/EmergencyContacts.svelte";
  import ContactDetailsPanel from "#lib/components/me/ContactDetailsPanel.svelte";
  import EqualityPanel from "#lib/components/me/EqualityPanel.svelte";
  import FlexibleWorkingPanel from "#lib/components/me/FlexibleWorkingPanel.svelte";
  import HealthPanel from "#lib/components/me/HealthPanel.svelte";
  import ResignationPanel from "#lib/components/me/ResignationPanel.svelte";
  import TimeOffPanel from "#lib/components/me/TimeOffPanel.svelte";
  import { l, t } from "#lib/i18n.svelte.js";

  type Org = { pid: string; worker_pid: string | null };
  const workerPid = $derived(
    ((page.data.organizations ?? []) as Org[]).find((m) => m.worker_pid)?.worker_pid ?? "",
  );
</script>

<svelte:head><title>{t("titles.my_details")}</title></svelte:head>

<h1>{t("details.title")}</h1>
<p><a href={l("/me")} data-testid="back-to-profile">{t("details.back")}</a></p>

{#if !workerPid}
  <p class="muted" data-testid="no-worker">{t("details.no_worker")}</p>
{:else}
  <ContactDetailsPanel />
  <EmergencyContacts {workerPid} />
  <TimeOffPanel />
  <FlexibleWorkingPanel />
  <ResignationPanel />
  <EqualityPanel />
  <HealthPanel />
{/if}

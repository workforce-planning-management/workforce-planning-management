<!--
  The separation records of a period, as a CSV for HR and a payroll service to submit (WPM-R222). The link is only a link: the server
  decides who may read it. Only a person with HR's view of an organization needs this, and anyone else is refused there.
-->
<script lang="ts">
  import { page } from "$app/state";
  import { separationExportUrl } from "#lib/api/wpm.js";
  import { t } from "#lib/i18n.svelte.js";

  const organization = $derived(((page.data.scope ?? []) as string[])[0] ?? "");
  let from = $state("");
  let to = $state("");
  const ready = $derived(Boolean(organization && from && to && from <= to));
</script>

{#if organization}
  <details class="panel" data-testid="separation-export">
    <summary>{t("separation.exportTitle")}</summary>
    <p class="muted">{t("separation.exportHint")}</p>
    <label>{t("separation.from")} <input type="date" bind:value={from} data-testid="separation-from" /></label>
    <label>{t("separation.to")} <input type="date" bind:value={to} data-testid="separation-to" /></label>
    {#if ready}
      <p><a href={separationExportUrl(organization, from, to)} download data-testid="separation-download">{t("separation.download")}</a></p>
    {/if}
  </details>
{/if}

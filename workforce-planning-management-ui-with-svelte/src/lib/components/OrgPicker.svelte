<!--
  Which organization a page reads. The caller's organizations come from the layout (`page.data.scope`);
  with none to choose from (a demo service with enforcement off) the reference is typed.
-->
<script lang="ts">
  import { page } from "$app/state";
  import { t } from "#lib/i18n.svelte.js";

  let { value = $bindable("") }: { value?: string } = $props();

  const refs = $derived((page.data.scope ?? []) as string[]);

  $effect(() => {
    if (!value && refs.length > 0) value = refs[0] ?? "";
  });
</script>

<label>
  {t("pages.planning.organization")}
  {#if refs.length > 0}
    <select bind:value data-testid="org-select">
      {#each refs as ref (ref)}<option value={ref}>{ref}</option>{/each}
    </select>
  {:else}
    <input bind:value data-testid="org-input" />
  {/if}
</label>

<style>
  /* An organization reference is long: let it shrink to the phone instead of widening the page. */
  label {
    display: block;
    max-width: 100%;
  }
  select,
  input {
    max-width: 100%;
    width: 100%;
  }
</style>

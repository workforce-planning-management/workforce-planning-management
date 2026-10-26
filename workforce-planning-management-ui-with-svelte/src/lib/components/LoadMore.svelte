<!--
  "Showing 500 of 1,240" and a button for the next page (WPM-T254). The service serves a list a page at a time (500 by
  default); without this a long list silently stops at the first page. `total` is `null` when the service did not say,
  and then only the button shows, never a made-up count. Nothing shows when there is no more.
-->
<script lang="ts">
  import { i18n, t, tf } from "#lib/i18n.svelte.js";

  let {
    shown,
    total,
    more,
    loading = false,
    onmore,
  }: {
    shown: number;
    total: number | null;
    more: boolean;
    loading?: boolean;
    onmore: () => void;
  } = $props();

  const fmt = $derived(new Intl.NumberFormat(i18n.locale));
</script>

{#if more}
  <p class="muted" data-testid="load-more">
    {#if total !== null}
      <span data-testid="load-more-count"
        >{tf("paging.showing", { shown: fmt.format(shown), total: fmt.format(total) })}</span
      >
    {/if}
    <button type="button" data-testid="load-more-button" disabled={loading} onclick={onmore}
      >{t("paging.more")}</button
    >
  </p>
{/if}

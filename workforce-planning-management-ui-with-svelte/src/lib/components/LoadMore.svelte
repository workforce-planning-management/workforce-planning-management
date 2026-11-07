<!--
  A button for the next page (WPM-T254). The service serves a list a page at a time (500 by default); without this a long
  list silently stops at the first page. There is no "of N": the service says only whether more rows follow, never a
  total (WPM-T478), so no count is made up. Nothing shows when there is no more.
-->
<script lang="ts">
  import { t } from "#lib/i18n.svelte.js";

  let {
    more,
    loading = false,
    onmore,
  }: {
    more: boolean;
    loading?: boolean;
    onmore: () => void;
  } = $props();
</script>

{#if more}
  <p class="muted" data-testid="load-more">
    <button type="button" data-testid="load-more-button" disabled={loading} onclick={onmore}
      >{t("paging.more")}</button
    >
  </p>
{/if}

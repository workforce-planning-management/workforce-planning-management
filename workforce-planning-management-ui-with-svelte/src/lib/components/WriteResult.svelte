<!-- The outcome of a write that did not go through: the refusal in words, or the service's message. -->
<script lang="ts">
  import type { Failure } from "#lib/write.js";
  import { t } from "#lib/i18n.svelte.js";

  let { failed }: { failed: Failure | null } = $props();
</script>

{#if failed}
  <p class="error" role="alert" data-testid="write-error">
    {failed.notAllowed ? t("common.notAllowed") : failed.message}
  </p>
{/if}

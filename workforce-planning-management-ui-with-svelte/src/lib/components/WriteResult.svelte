<!-- The outcome of a write that did not go through: the refusal in words, or the service's message. -->
<script lang="ts">
  import type { Failure } from "#lib/write.js";
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import { t } from "#lib/i18n.svelte.js";

  let { failed }: { failed: Failure | null } = $props();
</script>

{#if failed}
  <ErrorNote testid="write-error" message={failed.notAllowed ? t("common.notAllowed") : (failed.message ?? "")} />
{/if}

<!--
  One place a failure is shown (WPM-T261): a `role="alert"` paragraph, so a screen reader announces it.
  `message` is already text for a person (see `errorText` in `#lib/write`); `prefixed` puts the
  catalogue's "Error" word in front, as the page-level load errors do.
-->
<script lang="ts">
  import { t } from "#lib/i18n.svelte.js";

  let {
    message,
    testid,
    prefixed = false,
  }: { message: string; testid?: string; prefixed?: boolean } = $props();
</script>

<p class="error" role="alert" data-testid={testid}>{prefixed ? `${t("common.error")}: ` : ""}{message}</p>

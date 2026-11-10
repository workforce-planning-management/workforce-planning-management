<!--
  One place a failure is shown (WPM-T261): a `role="alert"` paragraph, so a screen reader announces it.
  `message` is already text for a person (see `errorText` in `#lib/write`); `prefixed` puts the
  catalogue's "Error" word in front, as the page-level load errors do.
  Beneath it, what to try next and a link to the help page (WPM-R243): every error says how to get help.
-->
<script lang="ts">
  import { l, t } from "#lib/i18n.svelte.js";
  import { nextStepKey } from "#lib/help.js";

  let {
    message,
    testid,
    prefixed = false,
  }: { message: string; testid?: string; prefixed?: boolean } = $props();
</script>

<p class="error" role="alert" data-testid={testid}>{prefixed ? `${t("common.error")}: ` : ""}{message}</p>
<p class="error-next" data-testid={testid ? `${testid}-next` : "error-next"}>
  {t(nextStepKey(message))}
  <a href={l("/help")}>{t("errors.getHelp")}</a>
</p>

<style>
  .error-next {
    margin-block-start: 0;
    font-size: 0.85rem;
    color: var(--muted);
  }
  /* A link inside running text is told apart by more than its colour (WCAG link-in-text-block). */
  .error-next a {
    text-decoration: underline;
  }
</style>

<!--
  The page for an address that does not exist or a page that failed to load (WPM-R243): says so in the reader's language, what
  to try, and where help is. It never shows the error's own message, which is English and for developers.
-->
<script lang="ts">
  import { page } from "$app/state";
  import { l, t } from "#lib/i18n.svelte.js";

  const heading = $derived(page.status === 404 ? t("errorPage.notFound") : t("errorPage.failed"));
</script>

<svelte:head><title>{heading}</title></svelte:head>

<h1 data-testid="error-page-title">{heading}</h1>
<p>{t("errorPage.hint")}</p>
<p>
  <a class="btn primary" href={l("/")} data-testid="error-page-home">{t("errorPage.home")}</a>
  <a class="btn" href={l("/help")} data-testid="error-page-help">{t("errors.getHelp")}</a>
</p>

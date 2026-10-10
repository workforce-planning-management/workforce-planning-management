<!--
  SSO callback page (BFF, Keycloak via the authentication service). The
  exchange happens entirely in the server `load`, which sets the
  httpOnly session cookie and redirects home on success — so this
  renders ONLY on a missing/invalid code. Mirrors `../+page.svelte`.
-->
<script lang="ts">
  import { l, t } from "#lib/i18n.svelte.js";
  import type { PageData } from "./$types";

  let { data }: { data: PageData } = $props();

  const message = $derived(
    data.error === "missingToken"
      ? t("pages.verifySso.missing")
      : data.error === "serviceUnavailable"
        ? t("pages.verifySso.unavailable")
        : t("pages.verifySso.invalid"),
  );
</script>

<svelte:head
  ><title>{t("pages.verifySso.page_title")}</title></svelte:head
>

<h1>{t("pages.verifySso.sign_in_link")}</h1>
<div class="panel">
  <p class="error" role="alert">{message}</p>
  <p><a href={l("/signin")}>{t("pages.verifySso.request_a_new_link")}</a></p>
</div>

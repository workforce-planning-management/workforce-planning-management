<!--
  Verify page (BFF, WPM-T18). The magic-link exchange happens entirely
  in the server `load`, which sets the httpOnly session cookie and
  redirects home on success — so this renders ONLY on a
  missing/invalid link.
-->
<script lang="ts">
  import { l, t } from "#lib/i18n.svelte.js";
  import type { PageData } from "./$types";

  let { data }: { data: PageData } = $props();

  const message = $derived(
    data.error === "missingToken"
      ? t("pages.verify.missing")
      : data.error === "serviceUnavailable"
        ? t("pages.verify.unavailable")
        : t("pages.verify.invalid"),
  );
</script>

<svelte:head
  ><title>{t("pages.verify.page_title")}</title></svelte:head
>

<h1>{t("pages.verify.sign_in_link")}</h1>
<div class="panel">
  <p class="error" role="alert">{message}</p>
  <p><a href={l("/signin")}>{t("pages.verify.request_a_new_link")}</a></p>
</div>

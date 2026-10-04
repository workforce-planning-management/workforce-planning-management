<!--
  SSO callback page (BFF, Keycloak via the authentication service). The
  exchange happens entirely in the server `load`, which sets the
  httpOnly session cookie and redirects home on success — so this
  renders ONLY on a missing/invalid code. Mirrors `../+page.svelte`.
-->
<script lang="ts">
  import { l } from "#lib/i18n.svelte.js";
  import type { PageData } from "./$types";

  let { data }: { data: PageData } = $props();

  const message = $derived(
    data.error === "missingToken"
      ? "This sign-in link is missing its code."
      : data.error === "serviceUnavailable"
        ? "We could not reach the sign-in service. Please try again in a moment."
        : "This sign-in link is invalid or has expired.",
  );
</script>

<svelte:head
  ><title>Sign-in link — Workforce Planning Management</title></svelte:head
>

<h1>Sign-in link</h1>
<div class="panel">
  <p class="error" role="alert">{message}</p>
  <p><a href={l("/signin")}>Request a new link</a></p>
</div>

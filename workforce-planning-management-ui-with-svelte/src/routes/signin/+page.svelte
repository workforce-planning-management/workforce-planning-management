<!--
  Sign-in page (BFF, per-app magic-link login, WPM-T18). Posts to the
  `default` server action, which calls the authentication service
  server-side with a return URL pointing back at THIS app's /verify.
  No token is held in the browser.
-->
<script lang="ts">
  import { l, t } from "#lib/i18n.svelte.js";
  import type { ActionData } from "./$types";
  import { enhance } from "$app/forms";

  let { form }: { form: ActionData } = $props();
</script>

<svelte:head><title>{t("pages.signin.page_title")}</title></svelte:head
>

<h1>{t("pages.signin.sign_in")}</h1>

{#if form?.sent}
  <div class="panel">
    <p>{t("pages.signin.check_your_email_for_a_sign_in")}</p>
  </div>
{:else}
  <div class="panel">
    <form class="row" method="POST" use:enhance>
      <label>
        {t("pages.signin.email")}
        <input type="email" name="email" required autocomplete="email" />
      </label>
      <button class="primary" type="submit">{t("pages.signin.send_magic_link")}</button>
    </form>
    {#if form?.error}
      <p class="error" role="alert">
        {t("pages.signin.could_not_send_the_sign_in_link")}
      </p>
    {/if}
    <p class="divider">{t("pages.signin.or")}</p>
    <a class="btn" href={l("/signin/sso")}>{t("pages.signin.sign_in_with_sso")}</a>
  </div>
{/if}

<style>
  .divider {
    text-align: center;
    color: var(--muted);
    margin: 0.75rem 0;
  }
  a.btn {
    display: inline-block;
    text-decoration: none;
  }
</style>


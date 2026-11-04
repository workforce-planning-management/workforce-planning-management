<script lang="ts">
  import { href } from "#lib/href.js";
  import { catalogues, ui } from "#lib/i18n.js";

  // The address is unknown, so its locale is too: the message is shown in each language the site's words are in.
  const languages = Object.values(catalogues);
</script>

<svelte:head><title>{ui(null).notFound.title} · {ui(null).brand}</title></svelte:head>

{#each languages as s (s.locale)}
  <section lang={s.locale} data-testid="not-found">
    {#if s.locale === ui(null).locale}<h1>{s.notFound.title}</h1>{:else}<h2>{s.notFound.title}</h2>{/if}
    <p>{s.notFound.body} <a href={href(s.locale === ui(null).locale ? "/" : `/${s.locale}/`)}>{s.notFound.home}</a></p>
  </section>
{/each}

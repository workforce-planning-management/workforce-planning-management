<!--
  The guided tour of the demo (`/demo`): reached by signing in as the example address where a demo is
  offered. Everything behind it is fictitious. Lists every page by the tour section it belongs to; a
  banner on each page then offers Previous and Next, so the whole product can be walked in order.
-->
<script lang="ts">
  import { DEMO_SECTIONS, DEMO_STEPS } from "#lib/demoTour.js";
  import { l, t, tf } from "#lib/i18n.svelte.js";
</script>

<svelte:head><title>{t("titles.demo")}</title></svelte:head>

<h1>{t("demo.title")}</h1>
<p>{t("demo.hint")}</p>
<p>
  <a class="btn primary" href={l(DEMO_STEPS[0]!.path)} data-testid="demo-start">
    {t("demo.start")}
  </a>
  <span class="muted">{tf("demo.count", { total: DEMO_STEPS.length })}</span>
</p>

{#each DEMO_SECTIONS as section (section.id)}
  <section class="panel" data-testid="demo-section">
    <h2>{t(`tour.${section.id}.title` as never)}</h2>
    <p class="muted">{t(`tour.${section.id}.body` as never)}</p>
    <ul>
      {#each section.steps as step (step.path)}
        <li><a href={l(step.path)}>{t(step.name)}</a></li>
      {/each}
    </ul>
  </section>
{/each}

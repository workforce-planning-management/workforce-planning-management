<!--
  A walkthrough for one role (WPM-R244): the tasks in order, each saying where to click and what you see, with a link to the page
  it starts from. Reached from the tour and from help; the pages it links to open once the reader is signed in.
-->
<script lang="ts">
  import { page } from "$app/state";
  import { l, t, type StringKey } from "#lib/i18n.svelte.js";
  import { GUIDES, GUIDE_ROLES, type GuideRole } from "#lib/guides.js";

  const role = $derived(page.data.role as GuideRole);
  const key = (suffix: string) => `guides.${role}.${suffix}` as StringKey;
  const roleTitle = (r: GuideRole) => t(`guides.${r}.title` as StringKey);
</script>

<svelte:head><title>{roleTitle(role)} — {t("guides.title")}</title></svelte:head>

<div class="guidepage">
  <p class="guidepage-eyebrow"><a href={l("/tour")}>{t("guides.title")}</a></p>
  <h1 data-testid="guide-title">{roleTitle(role)}</h1>
  <p>{t(key("intro"))}</p>
  {#if role === "hr"}<p>{t("guides.hr.chooseOrg")}</p>{/if}
  <p class="muted">{t("guides.signin")}</p>

  <ol class="guidepage-tasks" data-testid="guide-tasks">
    {#each GUIDES[role] as task (task.id)}
      <li class="panel" id={task.id} data-testid="guide-task">
        <h2>{t(key(`${task.id}.title`))}</h2>
        <p>{t(key(`${task.id}.body`))}</p>
        <p><a href={l(task.path)} data-testid="guide-open">{t("guides.open")}</a></p>
      </li>
    {/each}
  </ol>

  <nav aria-label={t("guides.others")}>
    <h2>{t("guides.others")}</h2>
    <ul>
      {#each GUIDE_ROLES.filter((r) => r !== role) as other (other)}
        <li><a href={l(`/tour/${other}`)}>{roleTitle(other)}</a></li>
      {/each}
      <li><a href={l("/help")} data-testid="guide-help">{t("errors.getHelp")}</a></li>
    </ul>
  </nav>
</div>

<style>
  .guidepage {
    max-width: 44rem;
  }
  .guidepage-eyebrow {
    margin: 0;
    font-size: 0.85rem;
  }
  .guidepage-tasks {
    padding-inline-start: 1.5rem;
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }
  .guidepage-tasks h2 {
    margin-block-start: 0;
    font-size: 1.15rem;
  }
</style>

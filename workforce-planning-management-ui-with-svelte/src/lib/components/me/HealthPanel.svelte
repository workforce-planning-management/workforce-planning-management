<!--
  My workplace health requirements (WPM-R128): each requirement that applies to me, with my own standing and dates.
  Only occupational health records these, and only I and occupational health see the detail; a manager and HR are told
  only whether I am cleared. No reason is held. Off unless a deployer enables it.
-->
<script lang="ts">
  import { getMyHealthRequirements, type MyHealthRequirements } from "#lib/api/wpm.js";
  import { t, tf, tv } from "#lib/i18n.svelte.js";

  let view = $state<MyHealthRequirements | null>(null);
  let error = $state<string | null>(null);

  $effect(() => {
    void (async () => {
      try {
        view = await getMyHealthRequirements();
      } catch (cause) {
        error = cause instanceof Error ? cause.message : String(cause);
      }
    })();
  });
</script>

<section class="panel" data-testid="my-health">
  <h2>{t("details.health.title")}</h2>
  {#if error}<p class="error" data-testid="health-error">{error}</p>{/if}
  {#if view && !view.enabled}
    <p class="muted" data-testid="health-off">{t("details.health.off")}</p>
  {:else if view?.enabled}
    <p class="muted">{view.notice}</p>
    <ul data-testid="health-list">
      {#each view.requirements as r (r.requirement.pid)}
        <li data-testid="health-{r.requirement.pid}">
          <strong>{r.requirement.name}</strong> — {tv(r.standing)}
          <span class="muted">· {r.cleared ? t("details.health.cleared") : t("details.health.not_cleared")}</span>
          {#if r.recorded_on}<span class="muted"> · {tf("details.health.recorded", { date: r.recorded_on })}</span>{/if}
          {#if r.next_due}<span class="muted"> · {tf("details.health.due", { date: r.next_due })}</span>{/if}
          {#if r.reminder}<strong class="warn" data-testid="health-reminder"> {t("details.health.reminder")}</strong>{/if}
          {#if r.requirement.description}<br /><span class="muted">{r.requirement.description}</span>{/if}
        </li>
      {:else}
        <li class="muted">{t("details.health.none")}</li>
      {/each}
    </ul>
  {/if}
</section>

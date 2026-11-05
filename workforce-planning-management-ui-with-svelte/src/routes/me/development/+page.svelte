<!--
  My development plans (WPM-R227, WPM-R228, WPM-R230): each of the person's plans with what it costs, the cost of its hours
  beside the fees, and whether it is worth doing on the benefits the person states. Estimates for the person to weigh.
-->
<script lang="ts">
  import { page } from "$app/state";
  import { guardedLoad } from "#lib/loader.svelte.js";
  import { errorText } from "#lib/write.js";
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import PlanEconomics from "#lib/components/PlanEconomics.svelte";
  import { listDevelopmentPlans, type DevelopmentPlans } from "#lib/api/wpm.js";
  import type { MyOrganization } from "#lib/api/types.js";
  import { t, tv } from "#lib/i18n.svelte.js";

  const me = $derived(
    ((page.data.organizations ?? []) as MyOrganization[]).find((m) => m.worker_pid !== null)?.worker_pid ?? null,
  );

  let plans = $state<DevelopmentPlans["plans"] | null>(null);
  let error = $state<string | null>(null);

  const load = guardedLoad(
    () => listDevelopmentPlans(me ?? ""),
    (found) => {
      plans = found.plans;
    },
    (cause) => {
      error = errorText(cause);
    },
  );

  $effect(() => {
    if (me) void load();
  });
</script>

<svelte:head><title>{t("titles.myDevelopment")}</title></svelte:head>

<h1>{t("economics.title")}</h1>
<p class="muted">{t("economics.hint")}</p>

{#if error}
  <ErrorNote testid="error" prefixed message={error} />
{:else if !me}
  <p class="muted" data-testid="development-no-worker">{t("kanban.noWorker")}</p>
{:else if plans === null}
  <p>{t("common.loading")}</p>
{:else}
  {#each plans as plan (plan.pid)}
    <section class="panel" data-testid="development-plan">
      <h2>{plan.target_job_title ?? tv(plan.kind)}</h2>
      <p class="muted">{tv(plan.kind)} · {tv(plan.status)}</p>
      <PlanEconomics pid={plan.pid} />
    </section>
  {:else}
    <p class="muted" data-testid="development-none">{t("economics.none")}</p>
  {/each}
{/if}

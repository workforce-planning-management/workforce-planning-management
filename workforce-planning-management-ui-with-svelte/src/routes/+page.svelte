<script lang="ts">
  import { page } from "$app/state";
  import { listWorkers, listRequisitions, successionGaps } from "#lib/api/wpm.js";
  import { t } from "#lib/i18n.svelte.js";
  import type { Worker, Requisition, MyOrganization } from "#lib/api/types.js";

  const signedIn = $derived(page.data.signedIn === true);
  // Fetched once in the root layout's server load and inherited here —
  // no switcher: every org this person belongs to, shown together.
  const organizations = $derived(
    (page.data.organizations ?? []) as MyOrganization[],
  );

  let active = $state<Worker[] | null>(null);
  let open = $state<Requisition[] | null>(null);
  let gapCount = $state<number | null>(null);
  let error = $state<string | null>(null);

  // Only a signed-in visitor reaches the dashboard tiles; a signed-out
  // one gets the splash below and never touches the API.
  $effect(() => {
    if (!signedIn) return;
    void (async () => {
      try {
        const [workers, requisitions, gaps] = await Promise.all([
          listWorkers({ status: "active" }),
          listRequisitions("open"),
          successionGaps(),
        ]);
        active = workers;
        open = requisitions;
        gapCount = gaps.gaps.length;
      } catch (cause) {
        error = cause instanceof Error ? cause.message : String(cause);
      }
    })();
  });

  const FEATURES = [
    ["home.features.f1.title", "home.features.f1.body"],
    ["home.features.f2.title", "home.features.f2.body"],
    ["home.features.f3.title", "home.features.f3.body"],
    ["home.features.f4.title", "home.features.f4.body"],
    ["home.features.f5.title", "home.features.f5.body"],
    ["home.features.f6.title", "home.features.f6.body"],
  ] as const;

  const BENEFITS = [
    ["home.benefits.b1.title", "home.benefits.b1.body"],
    ["home.benefits.b2.title", "home.benefits.b2.body"],
    ["home.benefits.b3.title", "home.benefits.b3.body"],
    ["home.benefits.b4.title", "home.benefits.b4.body"],
    ["home.benefits.b5.title", "home.benefits.b5.body"],
    ["home.benefits.b6.title", "home.benefits.b6.body"],
  ] as const;
</script>

{#if signedIn}
  <h1>{t("dash.title")}</h1>

  {#if error}
    <p class="error" data-testid="error">{t("common.error")}: {error}</p>
  {:else if active === null}
    <p>{t("common.loading")}</p>
  {:else}
    <div class="tiles">
      <a class="tile" href="/workers" data-testid="tile-active">
        <strong>{active.length}</strong>
        <span>{t("dash.activeWorkers")}</span>
      </a>
      <a class="tile" href="/requisitions" data-testid="tile-open">
        <strong>{open?.length ?? 0}</strong>
        <span>{t("dash.openRequisitions")}</span>
      </a>
      <a class="tile" href="/development" data-testid="tile-gaps">
        <strong>{gapCount ?? 0}</strong>
        <span>{t("dash.successionGaps")}</span>
      </a>
    </div>
  {/if}

  <section class="my-organizations" data-testid="my-organizations">
    <h2>{t("org.myOrganizations")}</h2>
    {#if organizations.length === 0}
      <p class="muted">{t("org.noOrganizations")}</p>
    {:else}
      <ul class="org-list">
        {#each organizations as membership (membership.pid)}
          <li class="org-row">
            <code class="org-ref">{membership.organization_ref}</code>
            <span class="chip">{membership.role}</span>
            <span class="chip" class:ok={membership.employed}>
              {membership.employed ? t("org.employed") : t("org.staffOnly")}
            </span>
          </li>
        {/each}
      </ul>
    {/if}
  </section>
{:else}
  <!-- home-* class prefix throughout (not hero/card/cta/hint/etc.): the
       active Lily theme inlines CSS for its own ~492 components under
       those exact generic names (`.hero`, `.card`, `.feature-card`,
       `.hint`, …), so a plain class here silently inherits their
       styling — verified for `.hero`, which comes with an unreadable
       dark background none of this page's CSS ever sets. -->
  <div class="home-splash" data-testid="splash">
    <section class="home-hero">
      <p class="home-eyebrow">{t("home.hero.eyebrow")}</p>
      <h1>{t("home.hero.headline")}</h1>
      <p class="home-subhead">{t("home.hero.subhead")}</p>
      <div class="home-hero-actions">
        <a class="btn primary" href="/signin" data-testid="hero-cta">
          {t("home.hero.cta")}
        </a>
        <span class="home-hint">{t("home.hero.ctaHint")}</span>
      </div>
    </section>

    <section class="home-features">
      <h2>{t("home.features.title")}</h2>
      <div class="home-feature-grid">
        {#each FEATURES as [titleKey, bodyKey] (titleKey)}
          <div class="home-feature-card">
            <h3>{t(titleKey)}</h3>
            <p>{t(bodyKey)}</p>
          </div>
        {/each}
      </div>
    </section>

    <section class="home-benefits">
      <h2>{t("home.benefits.title")}</h2>
      <div class="home-benefit-grid">
        {#each BENEFITS as [titleKey, bodyKey] (titleKey)}
          <div class="home-benefit-card">
            <h3>{t(titleKey)}</h3>
            <p>{t(bodyKey)}</p>
          </div>
        {/each}
      </div>
    </section>

    <section class="home-trust panel">
      <h2>{t("home.trust.title")}</h2>
      <p>{t("home.trust.body")}</p>
    </section>

    <section class="home-final-cta">
      <h2>{t("home.cta.title")}</h2>
      <p>{t("home.cta.body")}</p>
      <a class="btn primary" href="/signin" data-testid="final-cta">
        {t("home.cta.button")}
      </a>
    </section>

    <p class="home-demo-notice">{t("home.demoNotice")}</p>
  </div>
{/if}

<style>
  .tiles {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
    gap: 1rem;
  }
  .tile {
    background: var(--panel);
    border: 1px solid var(--line);
    border-radius: 8px;
    padding: 1rem;
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    color: inherit;
  }
  .tile strong {
    font-size: 2rem;
  }

  .my-organizations {
    margin-top: 1.5rem;
  }
  .org-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
  }
  .org-row {
    background: var(--panel);
    border: 1px solid var(--line);
    border-radius: 8px;
    padding: 0.5rem 0.75rem;
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex-wrap: wrap;
  }
  .org-ref {
    font-size: 0.82rem;
    color: var(--muted);
    overflow-wrap: anywhere;
  }

  .home-splash {
    display: flex;
    flex-direction: column;
    gap: 2.5rem;
    padding: 1rem 0 2rem;
  }

  .home-hero {
    text-align: center;
    max-width: 46rem;
    margin: 0 auto;
    padding: 1.5rem 0 0.5rem;
  }
  .home-eyebrow {
    color: var(--accent);
    font-weight: 600;
    font-size: 0.82rem;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    margin: 0 0 0.5rem;
  }
  .home-hero h1 {
    font-size: 2.1rem;
    line-height: 1.2;
    margin: 0 0 0.75rem;
  }
  .home-subhead {
    color: var(--muted);
    font-size: 1.05rem;
    margin: 0 auto 1.5rem;
    max-width: 40rem;
  }
  .home-hero-actions {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.5rem;
  }
  .home-hero-actions .btn {
    font-size: 1rem;
    padding: 0.6rem 1.5rem;
  }
  .home-hint {
    color: var(--muted);
    font-size: 0.82rem;
  }

  .home-benefits h2,
  .home-features h2,
  .home-trust h2,
  .home-final-cta h2 {
    text-align: center;
    font-size: 1.4rem;
    margin: 0 0 1.25rem;
  }

  /* Six tiles per area (benefits, features), deliberately breakpointed
     rather than auto-fit/auto-fill: 1x6 stacked on mobile, 2x3 or 3x2
     at intermediate widths, 6x1 in one row on a wide screen. */
  .home-benefit-grid,
  .home-feature-grid {
    display: grid;
    grid-template-columns: 1fr;
    gap: 1rem;
  }
  @media (min-width: 640px) {
    .home-benefit-grid,
    .home-feature-grid {
      grid-template-columns: repeat(2, 1fr);
    }
  }
  @media (min-width: 960px) {
    .home-benefit-grid,
    .home-feature-grid {
      grid-template-columns: repeat(3, 1fr);
    }
  }
  @media (min-width: 1280px) {
    .home-benefit-grid,
    .home-feature-grid {
      grid-template-columns: repeat(6, 1fr);
    }
  }
  .home-benefit-card {
    background: var(--panel);
    border: 1px solid var(--line);
    border-radius: 8px;
    padding: 1.1rem;
  }
  .home-benefit-card h3 {
    margin: 0 0 0.4rem;
    font-size: 1rem;
  }
  .home-benefit-card p {
    margin: 0;
    color: var(--muted);
    font-size: 0.9rem;
  }

  .home-feature-card {
    background: var(--panel);
    border: 1px solid var(--line);
    border-left: 4px solid var(--accent);
    border-radius: 8px;
    padding: 1.1rem;
  }
  .home-feature-card h3 {
    margin: 0 0 0.4rem;
    font-size: 1rem;
  }
  .home-feature-card p {
    margin: 0;
    color: var(--muted);
    font-size: 0.9rem;
  }

  .home-trust {
    max-width: 40rem;
    margin: 0 auto;
    text-align: center;
  }
  .home-trust p {
    color: var(--muted);
    margin: 0;
  }

  .home-final-cta {
    text-align: center;
    background: var(--panel);
    border: 1px solid var(--line);
    border-radius: 8px;
    padding: 1.75rem 1.25rem;
  }
  .home-final-cta p {
    color: var(--muted);
    margin: 0 0 1rem;
  }
  .home-final-cta .btn {
    font-size: 1rem;
    padding: 0.6rem 1.5rem;
  }

  .home-demo-notice {
    text-align: center;
    color: var(--muted);
    font-size: 0.78rem;
  }
</style>

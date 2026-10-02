<script lang="ts">
  import { t } from "#lib/i18n.svelte.js";

  // tourpage-* class prefix throughout: the active Lily theme styles a
  // real `.tour` component of its own (an onboarding card), plus
  // `.tour-list`/`.tour-list-item` — a plain "tour" name here would
  // silently inherit that styling rather than this page's own.
  const SECTIONS = [
    ["hiring", "tour.hiring.title", "tour.hiring.body"],
    ["people", "tour.people.title", "tour.people.body"],
    ["rota", "tour.rota.title", "tour.rota.body"],
    ["growth", "tour.growth.title", "tour.growth.body"],
    ["wellbeing", "tour.wellbeing.title", "tour.wellbeing.body"],
    ["payroll", "tour.payroll.title", "tour.payroll.body"],
    ["privacy", "tour.privacy.title", "tour.privacy.body"],
    ["multiorg", "tour.multiorg.title", "tour.multiorg.body"],
    ["dashboard", "tour.dashboard.title", "tour.dashboard.body"],
    ["yours", "tour.yours.title", "tour.yours.body"],
  ] as const;
</script>

<svelte:head>
  <title>{t("tour.hero.title")} — {t("brand.name")}</title>
</svelte:head>

<div class="tourpage">
  <header class="tourpage-hero">
    <p class="tourpage-eyebrow">{t("brand.name")}</p>
    <h1>{t("tour.hero.title")}</h1>
    <p class="tourpage-subhead">{t("tour.hero.subhead")}</p>
  </header>

  <nav class="tourpage-toc" aria-label={t("tour.toc.label")}>
    <p class="tourpage-toc-label">{t("tour.toc.label")}</p>
    <ul>
      {#each SECTIONS as [id, titleKey] (id)}
        <li><a href={`#${id}`}>{t(titleKey)}</a></li>
      {/each}
    </ul>
  </nav>

  {#each SECTIONS as [id, titleKey, bodyKey] (id)}
    <section class="tourpage-section" {id}>
      <h2>{t(titleKey)}</h2>
      <p>{t(bodyKey)}</p>
    </section>
  {/each}

  <section class="tourpage-cta panel">
    <h2>{t("tour.cta.title")}</h2>
    <p>{t("tour.cta.body")}</p>
    <a class="btn primary" href="/signin">{t("tour.cta.button")}</a>
  </section>
</div>

<style>
  .tourpage {
    display: flex;
    flex-direction: column;
    gap: 2rem;
    padding: 1rem 0 2rem;
  }

  .tourpage-hero {
    text-align: center;
    max-width: 42rem;
    margin: 0 auto;
  }
  .tourpage-eyebrow {
    color: var(--accent);
    font-weight: 600;
    font-size: 0.82rem;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    margin: 0 0 0.5rem;
  }
  .tourpage-hero h1 {
    font-size: 1.9rem;
    line-height: 1.2;
    margin: 0 0 0.6rem;
  }
  .tourpage-subhead {
    color: var(--muted);
    margin: 0;
  }

  .tourpage-toc {
    background: var(--panel);
    border: 1px solid var(--line);
    border-radius: 8px;
    padding: 1rem 1.25rem;
    max-width: 32rem;
    margin: 0 auto;
  }
  .tourpage-toc-label {
    margin: 0 0 0.5rem;
    font-size: 0.78rem;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--muted);
  }
  .tourpage-toc ul {
    list-style: none;
    margin: 0;
    padding: 0;
    columns: 2;
    gap: 1rem;
  }
  .tourpage-toc li {
    break-inside: avoid;
    margin-bottom: 0.35rem;
  }

  .tourpage-section {
    max-width: 42rem;
    margin: 0 auto;
    scroll-margin-top: 1rem;
  }
  .tourpage-section h2 {
    margin: 0 0 0.5rem;
  }
  .tourpage-section p {
    margin: 0;
    color: var(--ink);
    line-height: 1.6;
  }

  .tourpage-cta {
    text-align: center;
    max-width: 32rem;
    margin: 0 auto;
    padding: 1.75rem 1.25rem;
  }
  .tourpage-cta p {
    color: var(--muted);
    margin: 0 0 1rem;
  }
  .tourpage-cta .btn {
    font-size: 1rem;
    padding: 0.6rem 1.5rem;
  }
</style>

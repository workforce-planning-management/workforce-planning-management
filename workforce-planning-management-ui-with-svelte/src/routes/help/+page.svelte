<!--
  Help (WPM-R243): how to report a bug, how to ask when something is confusing, where each task is explained, and what to try
  when an error appears. Public, so a person who cannot sign in can still find it. The channels are the project's own (WPM-D191).
-->
<script lang="ts">
  import { l, t } from "#lib/i18n.svelte.js";
  import { DOCS_URL, ISSUES_URL, SECURITY_URL } from "#lib/help.js";
</script>

<svelte:head><title>{t("titles.help")}</title></svelte:head>

<div class="helppage">
  <h1>{t("help.title")}</h1>
  <p class="muted">{t("help.intro")}</p>

  <section class="panel" id="bug" data-testid="help-bug">
    <h2>{t("help.bugTitle")}</h2>
    <p>{t("help.bugBody")}</p>
    <p><a class="btn primary" href={ISSUES_URL} rel="noopener" data-testid="help-issues">{t("help.bugLink")}</a></p>
    <p class="muted">
      {t("help.securityBody")}
      <a href={SECURITY_URL} rel="noopener" data-testid="help-security">{t("help.securityLink")}</a>
    </p>
  </section>

  <section class="panel" id="question" data-testid="help-question">
    <h2>{t("help.questionTitle")}</h2>
    <p>{t("help.questionBody")}</p>
    <p><a href={ISSUES_URL} rel="noopener">{t("help.questionLink")}</a></p>
  </section>

  <section class="panel" id="learn" data-testid="help-learn">
    <h2>{t("help.learnTitle")}</h2>
    <p>{t("help.learnBody")}</p>
    <ul>
      <li><a href={l("/tour")} data-testid="help-tour">{t("help.tourLink")}</a></li>
      <li><a href={DOCS_URL} rel="noopener" data-testid="help-docs">{t("help.docsLink")}</a></li>
    </ul>
  </section>

  <section class="panel" id="errors" data-testid="help-errors">
    <h2>{t("help.errorTitle")}</h2>
    <p>{t("help.errorBody")}</p>
  </section>

  <p class="muted" data-testid="help-deployer">{t("help.deployerNote")}</p>
</div>

<style>
  .helppage {
    max-width: 44rem;
  }
  .helppage section {
    margin-block-end: 1rem;
  }
  .helppage p a:not(.btn) {
    text-decoration: underline;
  }
  .helppage h2 {
    margin-block-start: 0;
  }
</style>

<!--
  My mandatory training and certifications (WPM-R245, WPM-D192): one place, from /me, listing the statutory and mandatory learning
  that applies to the person (status, last completed, expiry) and the qualifications their role asks for (standing, and the day the
  record that answers it expires), most urgent first, each with what to do next and a link to do it. It reads the same two answers
  the panels on /me read; it decides nothing itself.
-->
<script lang="ts">
  import { page } from "$app/state";
  import { guardedLoad } from "#lib/loader.svelte.js";
  import { errorText } from "#lib/write.js";
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import { workerMandatoryLearning, workerQualifications } from "#lib/api/wpm.js";
  import type { MyLearningRow, MyOrganization, QualificationList } from "#lib/api/types.js";
  import { formatDate } from "#lib/locale-format.js";
  import { todayIso } from "#lib/reader-calendar.js";
  import { l, t, type StringKey } from "#lib/i18n.svelte.js";
  import {
    answering,
    certificationNeedsAttention,
    learningNeedsAttention,
    orderCertifications,
    orderLearning,
  } from "#lib/required.js";

  const me = $derived(
    ((page.data.organizations ?? []) as MyOrganization[]).find((m) => m.worker_pid !== null)?.worker_pid ?? null,
  );

  let learning = $state<MyLearningRow[] | null>(null);
  let learningError = $state<string | null>(null);
  let quals = $state<QualificationList | null>(null);
  let qualsError = $state<string | null>(null);

  const loadLearning = guardedLoad(
    () => workerMandatoryLearning(me ?? ""),
    (answer) => {
      learning = orderLearning(answer.requirements);
    },
    (cause) => {
      learningError = errorText(cause);
    },
  );
  const loadQuals = guardedLoad(
    () => workerQualifications(me ?? ""),
    (answer) => {
      quals = answer;
    },
    (cause) => {
      qualsError = errorText(cause);
    },
  );

  $effect(() => {
    if (me) {
      void loadLearning();
      void loadQuals();
    }
  });

  const today = todayIso();
  const certs = $derived(quals ? orderCertifications(quals.gap) : []);
  const allClear = $derived(
    learning !== null &&
      quals !== null &&
      !learning.some((r) => learningNeedsAttention(r.status)) &&
      !certs.some((c) => certificationNeedsAttention(c.standing)),
  );

  const STANDING: Record<string, StringKey> = {
    met: "qualifications.standingMet",
    declared_only: "qualifications.standingDeclared",
    expired: "qualifications.standingExpired",
    missing: "qualifications.standingMissing",
  };
  const CERT_HOW: Record<string, StringKey | null> = {
    met: null,
    declared_only: "required.certDeclaredHow",
    expired: "required.certExpiredHow",
    missing: "required.certMissingHow",
  };
</script>

<svelte:head><title>{t("titles.required")}</title></svelte:head>

<h1>{t("required.title")}</h1>
<p class="muted">{t("required.hint")}</p>

{#if !me}
  <p class="muted" data-testid="required-no-worker">{t("kanban.noWorker")}</p>
{:else}
  {#if allClear}
    <p class="rq-clear" data-testid="required-all-clear">✓ {t("required.allClear")}</p>
  {/if}

  <section class="panel" data-testid="required-learning">
    <h2>{t("required.learningTitle")}</h2>
    {#if learningError}
      <ErrorNote testid="required-learning-error" message={learningError} />
    {:else if learning === null}
      <p>{t("common.loading")}</p>
    {:else if learning.length === 0}
      <p>{t("mandatoryLearning.none")}</p>
    {:else}
      <ul class="rq-list">
        {#each learning as row (row.requirement_pid)}
          {@const attention = learningNeedsAttention(row.status)}
          <li class={["rq-item", { "rq-attention": attention }]} data-testid={`required-learning-${row.status}`}>
            <p class="rq-head">
              <strong>{row.subject}</strong>
              <span class="chip">
                {attention ? "▲" : row.status === "compliant" ? "✓" : "●"}
                {t(`mandatoryLearning.${row.status}` as StringKey)}
              </span>
            </p>
            <p class="rq-dates">
              {#if row.last_completed_on}
                <span>{t("mandatoryLearning.lastDone")}: {formatDate(row.last_completed_on)}</span>
              {/if}
              {#if row.expires_on}
                <span>{t("mandatoryLearning.expires")}: {formatDate(row.expires_on)}</span>
              {/if}
            </p>
            {#if attention}
              <p>{t("required.learningHow")}</p>
              <p><a href={l(`/workers/${me}#training`)} data-testid="required-learning-act">{t("required.learningAct")}</a></p>
            {:else if row.status === "unknown"}
              <p>{t("mandatoryLearning.unknownNote")}</p>
            {/if}
          </li>
        {/each}
      </ul>
    {/if}
  </section>

  <section class="panel" data-testid="required-certs">
    <h2>{t("required.certsTitle")}</h2>
    {#if qualsError}
      <ErrorNote testid="required-certs-error" message={qualsError} />
    {:else if quals === null}
      <p>{t("common.loading")}</p>
    {:else if quals.role === null}
      <p data-testid="required-no-role">{t("qualifications.noRole")}</p>
    {:else if certs.length === 0}
      <p>{t("qualifications.noRequirements")}</p>
    {:else}
      <ul class="rq-list">
        {#each certs as cert (cert.title)}
          {@const attention = certificationNeedsAttention(cert.standing)}
          {@const held = answering(cert, quals.qualifications, today)}
          {@const how = CERT_HOW[cert.standing]}
          <li class={["rq-item", { "rq-attention": attention }]} data-testid={`required-cert-${cert.standing}`}>
            <p class="rq-head">
              <strong>{cert.title}</strong>
              <span class="chip">{attention ? "▲" : "✓"} {t(STANDING[cert.standing] ?? "qualifications.standingMissing")}</span>
            </p>
            {#if held}
              <p class="rq-dates">
                <span data-testid="required-cert-expiry">
                  {held.expires_on
                    ? `${t("qualifications.expiresOn")}: ${formatDate(held.expires_on)}`
                    : t("required.noExpiry")}
                </span>
              </p>
            {/if}
            {#if how}
              <p>{t(how)}</p>
              <p><a href={l("/me#my-qualifications")} data-testid="required-cert-act">{t("required.certAct")}</a></p>
            {/if}
          </li>
        {/each}
      </ul>
    {/if}
  </section>
{/if}

<style>
  .rq-list {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }
  .rq-item {
    border-inline-start: 3px solid var(--line);
    padding-inline-start: 0.75rem;
  }
  .rq-item p {
    margin: 0.2rem 0;
  }
  .rq-attention {
    border-inline-start-color: var(--state-closed);
  }
  .rq-head {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.5rem;
  }
  .rq-dates {
    display: flex;
    flex-wrap: wrap;
    gap: 0 1rem;
    color: var(--muted);
    font-size: 0.9rem;
  }
  .rq-item a {
    text-decoration: underline;
  }
  .rq-clear {
    font-weight: 600;
  }
</style>

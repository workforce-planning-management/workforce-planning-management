<!--
  My qualifications (WPM-R171): what the person holds, declared or verified, and how each qualification their current role
  asks for stands. A declared one is never shown as met; only HR's verification, with a reference to where the proof is
  held, makes it so. No certificate is stored here. With no role recorded the gap says so in words, never an empty "all met".
-->
<script lang="ts">
  import { formatDate } from "#lib/locale-format.js";
  import { guardedLoad } from "#lib/loader.svelte.js";
  import { attempt, errorText } from "#lib/write.js";
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import {
    addQualification,
    removeQualification,
    verifyQualification,
    workerQualifications,
  } from "#lib/api/wpm.js";
  import type { QualificationList } from "#lib/api/types.js";
  import { t } from "#lib/i18n.svelte.js";

  let { workerPid }: { workerPid: string } = $props();

  let data = $state<QualificationList | null>(null);
  let error = $state<string | null>(null);
  let message = $state<string | null>(null);
  let reads = $state(0);

  let kind = $state("professional");
  let title = $state("");
  let awardingBody = $state("");
  let framework = $state("");
  let level = $state("");
  let awardedOn = $state("");
  let expiresOn = $state("");
  let evidence = $state<Record<string, string>>({});

  const load = guardedLoad(
    () => workerQualifications(workerPid),
    (answer) => {
      data = answer;
    },
    (cause) => {
      error = errorText(cause);
    },
  );

  $effect(() => {
    void workerPid;
    void reads;
    void load();
  });

  async function done(action: () => Promise<unknown>) {
    const failed = await attempt(action);
    message = failed?.message ?? null;
    if (failed === null) reads += 1;
    return failed === null;
  }

  async function add(event: SubmitEvent) {
    event.preventDefault();
    const n = Number.parseInt(level, 10);
    const withLevel = framework.trim() !== "" && Number.isFinite(n);
    const ok = await done(() =>
      addQualification(workerPid, {
        kind,
        title,
        awarding_body: awardingBody.trim() || undefined,
        framework: withLevel ? framework.trim() : undefined,
        level: withLevel ? n : undefined,
        awarded_on: awardedOn || undefined,
        expires_on: expiresOn || undefined,
      }),
    );
    if (ok) {
      title = awardingBody = framework = level = awardedOn = expiresOn = "";
    }
  }

  const standing = (s: string) =>
    s === "met"
      ? t("qualifications.standingMet")
      : s === "declared_only"
        ? t("qualifications.standingDeclared")
        : s === "expired"
          ? t("qualifications.standingExpired")
          : t("qualifications.standingMissing");
</script>

<section class="panel" data-testid="my-qualifications">
  <h2>{t("qualifications.title")}</h2>
  {#if error}
    <ErrorNote testid="my-qualifications-error" message={error} />
  {:else if data === null}
    <p>{t("common.loading")}</p>
  {:else}
    {#if data.qualifications.length === 0}
      <p>{t("qualifications.none")}</p>
    {:else}
      <ul>
        {#each data.qualifications as q (q.pid)}
          <li data-testid={`qualification-${q.status}`}>
            <strong>{q.title}</strong>
            <span class="chip"
              >{q.status === "verified" ? t("qualifications.verified") : t("qualifications.declared")}</span
            >
            {#if q.expired}<span class="chip">{t("qualifications.standingExpired")}</span>{/if}
            {#if q.framework && q.level !== null}
              <span class="muted">{q.framework} {q.level}</span>
            {/if}
            {#if q.expires_on}
              <span class="muted">{t("qualifications.expiresOn")}: {formatDate(q.expires_on)}</span>
            {/if}
            {#if q.status === "declared"}
              <input
                bind:value={evidence[q.pid]}
                placeholder={t("qualifications.evidence")}
                aria-label={t("qualifications.evidence")}
                maxlength="200"
              />
              <button
                type="button"
                data-testid="qualification-verify"
                onclick={() => void done(() => verifyQualification(q.pid, evidence[q.pid] ?? ""))}
                >{t("qualifications.verify")}</button
              >
            {/if}
            <button type="button" onclick={() => void done(() => removeQualification(q.pid))}
              >{t("qualifications.remove")}</button
            >
          </li>
        {/each}
      </ul>
    {/if}

    <h3>{t("qualifications.gapTitle")}</h3>
    {#if data.role === null}
      <p data-testid="qualification-no-role">{t("qualifications.noRole")}</p>
    {:else if data.gap.length === 0}
      <p>{t("qualifications.noRequirements")}</p>
    {:else}
      <ul>
        {#each data.gap as g (g.title)}
          <li data-testid={`gap-${g.standing}`}>
            <strong>{g.title}</strong>
            <span class="chip">{standing(g.standing)}</span>
          </li>
        {/each}
      </ul>
    {/if}
  {/if}

  <h3>{t("qualifications.add")}</h3>
  <form onsubmit={add}>
    <label
      >{t("qualifications.kind")}
      <select bind:value={kind} data-testid="qualification-kind">
        <option value="academic">{t("values.academic")}</option>
        <option value="professional">{t("values.professional")}</option>
        <option value="vocational">{t("values.vocational")}</option>
        <option value="licence">{t("values.licence")}</option>
      </select>
    </label>
    <label
      >{t("qualifications.name")}
      <input bind:value={title} required maxlength="200" data-testid="qualification-title" /></label
    >
    <label>{t("qualifications.awardingBody")} <input bind:value={awardingBody} maxlength="200" /></label>
    <label>{t("qualifications.framework")} <input bind:value={framework} maxlength="40" /></label>
    <label>{t("qualifications.level")} <input bind:value={level} inputmode="numeric" /></label>
    <label>{t("qualifications.awardedOn")} <input type="date" bind:value={awardedOn} /></label>
    <label>{t("qualifications.expiresOn")} <input type="date" bind:value={expiresOn} /></label>
    <button type="submit" data-testid="qualification-save">{t("qualifications.save")}</button>
  </form>
  {#if message}<ErrorNote testid="my-qualifications-message" {message} />{/if}
</section>

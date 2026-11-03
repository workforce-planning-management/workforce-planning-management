<!--
  Coverage evidence (`/coverage-evidence`, WPM-T420): HR keeps what shows an organization is covered where it employs people (a
  statutory scheme, a private policy, self-insurance, or a recorded choice of none) and sees each record's standing today. A record
  with no end date is "no end date", never "valid". A certificate is a reference to where it is kept, never the document. The service
  decides who may; a refusal is said in its own words. A daily reminder tells HR before an end date.
-->
<script lang="ts">
  import { page } from "$app/state";
  import { attempt, errorText } from "#lib/write.js";
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import { formatDate } from "#lib/locale-format.js";
  import { i18n, t } from "#lib/i18n.svelte.js";
  import {
    addCoverageEvidence,
    listCoverageEvidence,
    money,
    retireCoverageEvidence,
  } from "#lib/api/wpm.js";
  import type { CoverageRecord } from "#lib/api/types.js";

  const organizations = $derived((page.data.scope ?? []) as string[]);
  let organization = $state("");
  let records = $state<CoverageRecord[] | null>(null);
  let error = $state<string | null>(null);
  let message = $state<string | null>(null);
  let saved = $state(false);
  // Bumped after HR changes something, so the list is read again.
  let reads = $state(0);

  let jurisdiction = $state("");
  let kind = $state("statutory_scheme");
  let insurer = $state("");
  let policy = $state("");
  let cover = $state("");
  let currency = $state("");
  let validFrom = $state("");
  let validTo = $state("");
  let certificate = $state("");

  $effect(() => {
    const chosen = organization;
    void reads;
    error = null;
    if (!chosen) {
      records = null;
      return;
    }
    let stale = false;
    void (async () => {
      try {
        const found = await listCoverageEvidence(chosen);
        if (!stale) records = found;
      } catch (cause) {
        if (!stale) error = errorText(cause);
      }
    })();
    return () => {
      stale = true;
    };
  });

  const isPrivate = $derived(kind === "private_insurance");

  async function done(action: () => Promise<unknown>) {
    saved = false;
    const failed = await attempt(action);
    message = failed?.message ?? null;
    if (failed === null) {
      saved = true;
      reads += 1;
    }
    return failed === null;
  }

  async function add(event: SubmitEvent) {
    event.preventDefault();
    const amount = Number.parseInt(cover, 10);
    const ok = await done(() =>
      addCoverageEvidence({
        organization,
        jurisdiction: jurisdiction.trim(),
        kind,
        insurer: insurer.trim() || undefined,
        policy_reference: policy.trim() || undefined,
        minimum_cover_minor: Number.isFinite(amount) ? amount : undefined,
        currency: currency.trim() || undefined,
        valid_from: validFrom || undefined,
        valid_to: validTo || undefined,
        certificate_ref: certificate.trim() || undefined,
      }),
    );
    if (ok) {
      jurisdiction = insurer = policy = cover = currency = certificate = "";
      validFrom = validTo = "";
    }
  }

  const standing = (r: CoverageRecord) =>
    r.standing === "ending_soon" && r.days_left !== null
      ? `${t("coverageEvidence.ending_soon")} (${r.days_left})`
      : t(`coverageEvidence.${r.standing}` as "coverageEvidence.in_force");
</script>

<svelte:head><title>{page.data.title}</title></svelte:head>

<h1>{t("coverageEvidence.title")}</h1>
<p class="muted">{t("coverageEvidence.hint")}</p>
<label>
  {t("coverageEvidence.organization")}
  <select bind:value={organization} data-testid="coverage-organization">
    <option value=""></option>
    {#each organizations as o (o)}<option value={o}>{o}</option>{/each}
  </select>
</label>
{#if error}<ErrorNote testid="error" prefixed message={error} />{/if}

{#if organization}
  {#if records}
    {#if records.length === 0}
      <p data-testid="coverage-none">{t("coverageEvidence.none")}</p>
    {:else}
      <table data-testid="coverage-list">
        <caption>{t("coverageEvidence.title")}</caption>
        <thead>
          <tr>
            <th scope="col">{t("coverageEvidence.jurisdiction")}</th>
            <th scope="col">{t("coverageEvidence.kind")}</th>
            <th scope="col">{t("coverageEvidence.standing")}</th>
            <th scope="col">{t("coverageEvidence.validTo")}</th>
            <th scope="col">{t("coverageEvidence.cover")}</th>
            <th scope="col">{t("coverageEvidence.actions")}</th>
          </tr>
        </thead>
        <tbody>
          {#each records as r (r.pid)}
            <tr data-testid="coverage-row">
              <td>{r.jurisdiction}</td>
              <td>{t(`coverageEvidence.${r.kind}` as "coverageEvidence.statutory_scheme")}</td>
              <td data-testid="coverage-standing">{standing(r)}</td>
              <td>{formatDate(r.valid_to)}</td>
              <td>
                {r.minimum_cover_minor !== null && r.currency
                  ? money(r.minimum_cover_minor, r.currency, i18n.locale)
                  : "—"}
              </td>
              <td>
                <button type="button" onclick={() => void done(() => retireCoverageEvidence(r.pid))}>
                  {t("coverageEvidence.retire")}
                </button>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    {/if}
  {/if}

  <section class="panel" data-testid="coverage-admin">
    <h2>{t("coverageEvidence.addTitle")}</h2>
    <form onsubmit={add}>
      <label>{t("coverageEvidence.jurisdiction")} <input bind:value={jurisdiction} required maxlength="16" data-testid="cov-jurisdiction" /></label>
      <label
        >{t("coverageEvidence.kind")}
        <select bind:value={kind} data-testid="cov-kind">
          <option value="statutory_scheme">{t("coverageEvidence.statutory_scheme")}</option>
          <option value="private_insurance">{t("coverageEvidence.private_insurance")}</option>
          <option value="self_insured">{t("coverageEvidence.self_insured")}</option>
          <option value="elective_none">{t("coverageEvidence.elective_none")}</option>
        </select>
      </label>
      {#if isPrivate}
        <label>{t("coverageEvidence.insurer")} <input bind:value={insurer} required maxlength="120" data-testid="cov-insurer" /></label>
        <label>{t("coverageEvidence.policy")} <input bind:value={policy} required maxlength="120" data-testid="cov-policy" /></label>
      {/if}
      <label>{t("coverageEvidence.coverMinor")} <input bind:value={cover} inputmode="numeric" data-testid="cov-cover" /></label>
      <label>{t("coverageEvidence.currency")} <input bind:value={currency} maxlength="3" data-testid="cov-currency" required={cover !== ""} /></label>
      <label>{t("coverageEvidence.validFrom")} <input type="date" bind:value={validFrom} data-testid="cov-from" /></label>
      <label>{t("coverageEvidence.validTo")} <input type="date" bind:value={validTo} data-testid="cov-to" /></label>
      <label>{t("coverageEvidence.certificate")} <input bind:value={certificate} maxlength="200" data-testid="cov-certificate" /></label>
      <button type="submit" data-testid="cov-save">{t("coverageEvidence.save")}</button>
    </form>
    {#if message}
      <ErrorNote testid="cov-error" message={message} />
    {:else if saved}
      <p role="status" data-testid="cov-saved">{t("coverageEvidence.saved")}</p>
    {/if}
  </section>
{/if}

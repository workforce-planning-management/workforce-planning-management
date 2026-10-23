<!--
  A worker's engagement (WPM-R79–R81, WPM-R98): the basis, the last day, where it stands, the
  extensions and the decision; beside it the post's funding and whether it matches the contract
  end (HR only); and a contractor's supplier, route and rate. The rate is masked like salary and
  renders as a state, never as 0. A permanent engagement with nothing to watch renders nothing,
  and so does anyone the service will not show it to.
-->
<script lang="ts">
  import { errorText } from "#lib/write.js";
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import {
    money,
    workerContractorDetails,
    workerEngagement,
    workerFunding,
    type ContractorDetails,
    type WorkerEngagement,
    type WorkerFunding,
  } from "#lib/api/wpm.js";
  import { ApiError } from "#lib/api/client.js";
  import { i18n, t, tv } from "#lib/i18n.svelte.js";

  let { workerPid }: { workerPid: string } = $props();

  let engagement = $state<WorkerEngagement | null>(null);
  let funding = $state<WorkerFunding | null>(null);
  let details = $state<ContractorDetails | null>(null);
  let error = $state<string | null>(null);

  /** The service refuses (or does not know) a read that is not theirs to see: show nothing. */
  const notTheirs = (cause: unknown) =>
    cause instanceof ApiError && [401, 403, 404].includes(cause.status);

  $effect(() => {
    const pid = workerPid;
    let current = true;
    void (async () => {
      try {
        const found = await workerEngagement(pid);
        if (!current) return;
        engagement = found;
        error = null;
        // Funding is HR's, and details are a contractor's: neither is an error when absent.
        funding = await workerFunding(pid).catch(() => null);
        details =
          found.employment_type === "contractor"
            ? ((await workerContractorDetails(pid).catch(() => null))?.contractor_details ?? null)
            : null;
      } catch (cause) {
        if (!current) return;
        engagement = null;
        error = notTheirs(cause) ? null : errorText(cause);
      }
    })();
    return () => {
      current = false;
    };
  });

  const watched = $derived(
    engagement !== null && (engagement.employment_type !== "permanent" || funding?.funding),
  );
  const attention = $derived(
    engagement?.standing === "past_end_undecided" ||
      engagement?.standing === "missing_end_date" ||
      funding?.needs_attention === true,
  );
</script>

{#if error}
  <ErrorNote testid="engagement-error" prefixed message={error} />
{:else if engagement && watched}
  <section class="panel" data-testid="engagement">
    <h2>{t("engagement.title")}</h2>
    <dl>
      <dt>{t("engagement.basis")}</dt>
      <dd data-testid="engagement-basis">{tv(engagement.employment_type)}</dd>
      <dt>{t("engagement.started")}</dt>
      <dd>{engagement.hired_on}</dd>
      <dt>{t("engagement.ends")}</dt>
      <dd data-testid="engagement-ends">{engagement.engagement_ends_on ?? t("engagement.noEnd")}</dd>
      {#if engagement.standing}
        <dt>{t("engagement.standing")}</dt>
        <dd data-testid="engagement-standing">
          {#if attention}<span aria-hidden="true">⚠ </span>{/if}{tv(engagement.standing)}
        </dd>
      {/if}
      {#if engagement.decision}
        <dt>{t("engagement.decision")}</dt>
        <dd data-testid="engagement-decision">{tv(engagement.decision)}</dd>
      {/if}
    </dl>

    <h3>{t("engagement.extensions")}</h3>
    {#if engagement.extensions.length === 0}
      <p class="muted" data-testid="engagement-no-extensions">{t("engagement.noExtensions")}</p>
    {:else}
      <ul data-testid="engagement-extensions">
        {#each engagement.extensions as e (e.pid)}
          <li>{e.previous_end} → {e.new_end}{#if e.reason} <span class="muted">· {e.reason}</span>{/if}</li>
        {/each}
      </ul>
    {/if}

    {#if funding}
      <h3>{t("engagement.funding")}</h3>
      <div data-testid="engagement-funding">
        {#if funding.funding}
          <p>
            {tv(funding.funding.kind)}
            {#if funding.funding.programme_ref}<span class="muted">· {funding.funding.programme_ref}</span>{/if}
            · {t("engagement.fundingEnds")}: {funding.funding.ends_on ?? t("engagement.noEnd")}
          </p>
          <p data-testid="engagement-alignment">
            {#if funding.needs_attention}<span aria-hidden="true">⚠ </span>{/if}{tv(funding.alignment)}
          </p>
        {:else}
          <p class="muted">{t("engagement.fundingNone")}</p>
        {/if}
      </div>
    {/if}

    {#if details}
      <h3>{t("engagement.contractor")}</h3>
      <dl data-testid="engagement-contractor">
        {#if details.route}
          <dt>{t("engagement.route")}</dt>
          <dd>{tv(details.route)}</dd>
        {/if}
        {#if details.supplier_ref}
          <dt>{t("engagement.supplier")}</dt>
          <dd>{details.supplier_ref}</dd>
        {/if}
        <dt>{t("engagement.rate")}</dt>
        <dd data-testid="engagement-rate">
          {#if details.rate_masked}
            <span class="muted">{t("engagement.rateMasked")}</span>
          {:else if details.rate_minor !== null && details.rate_currency}
            {money(details.rate_minor, details.rate_currency, i18n.locale)}{#if details.rate_basis}
              / {tv(details.rate_basis)}{/if}
          {:else}
            —
          {/if}
        </dd>
      </dl>
    {/if}
  </section>
{/if}

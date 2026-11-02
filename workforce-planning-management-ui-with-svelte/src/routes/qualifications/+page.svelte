<!--
  Qualifications awaiting verification (`/qualifications`, WPM-R171, WPM-T435): HR's queue of the qualifications people
  in an organization declared that nobody has verified, oldest first, each with the person's name. HR verifies one by
  naming where the proof is held (no certificate is stored). It is a list of records to check, not a ranking of people.
-->
<script lang="ts">
  import { page } from "$app/state";
  import { formatDate } from "#lib/locale-format.js";
  import { attempt, errorText } from "#lib/write.js";
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import { unverifiedQualifications, verifyQualification } from "#lib/api/wpm.js";
  import type { QueuedQualification } from "#lib/api/types.js";
  import { l, t } from "#lib/i18n.svelte.js";

  const organizations = $derived((page.data.scope ?? []) as string[]);
  let organization = $state("");
  let queue = $state<QueuedQualification[] | null>(null);
  let error = $state<string | null>(null);
  let evidence = $state<Record<string, string>>({});
  let reads = $state(0);

  $effect(() => {
    const chosen = organization;
    void reads;
    error = null;
    if (!chosen) {
      queue = null;
      return;
    }
    let stale = false;
    void (async () => {
      try {
        const found = await unverifiedQualifications(chosen);
        if (!stale) queue = found.queue;
      } catch (cause) {
        if (!stale) error = errorText(cause);
      }
    })();
    return () => {
      stale = true;
    };
  });

  async function verify(pid: string) {
    const failed = await attempt(() =>
      verifyQualification(pid, evidence[pid] ?? ""),
    );
    error = failed?.message ?? null;
    if (failed === null) reads += 1;
  }
</script>

<svelte:head><title>{page.data.title}</title></svelte:head>

<h1>{t("qualifications.queueTitle")}</h1>
<p class="muted">{t("qualifications.queueHint")}</p>
<label>
  {t("mandatoryLearning.organization")}
  <select bind:value={organization} data-testid="queue-organization">
    <option value=""></option>
    {#each organizations as o (o)}<option value={o}>{o}</option>{/each}
  </select>
</label>
{#if error}<ErrorNote testid="error" prefixed message={error} />{/if}
{#if queue}
  {#if queue.length === 0}
    <p data-testid="queue-none">{t("qualifications.queueNone")}</p>
  {:else}
    <ul data-testid="queue">
      {#each queue as q (q.pid)}
        <li data-testid="queue-item">
          <a href={l(`/workers/${q.worker_pid}`)}>{q.display_name}</a>:
          {q.title}
          <span class="muted"
            >({t(`values.${q.kind}`)}, {t("qualifications.declared")}
            {formatDate(q.declared_on)})</span
          >
          <input
            bind:value={evidence[q.pid]}
            placeholder={t("qualifications.evidence")}
            aria-label={t("qualifications.evidence")}
          />
          <button
            type="button"
            data-testid="queue-verify"
            onclick={() => void verify(q.pid)}>{t("qualifications.verify")}</button
          >
        </li>
      {/each}
    </ul>
  {/if}
{/if}

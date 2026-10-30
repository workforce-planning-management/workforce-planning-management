<!--
  My credentials: a worker may ask for an Open Badges credential for a CPD
  entry that has been verified. The signed credential is shown once, here, and
  the service keeps no copy: only that it was issued and whether it is
  revoked. The worker can revoke their own. The credential names no one;
  whoever holds it chooses who sees it.
-->
<script lang="ts">
  import { formatDate } from "#lib/locale-format.js";
  import { guardedLoad } from "#lib/loader.svelte.js";
  import { errorText } from "#lib/write.js";
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import {
    issueMyCredential,
    listCpdEntries,
    listMyCredentials,
    revokeMyCredential,
  } from "#lib/api/wpm.js";
  import type { CredentialIssuance } from "#lib/api/wpm.js";
  import { t, tf } from "#lib/i18n.svelte.js";

  let { workerPid }: { workerPid: string } = $props();

  type Entry = Awaited<ReturnType<typeof listCpdEntries>>[number];
  let entries = $state<Entry[]>([]);
  let issued = $state<CredentialIssuance[]>([]);
  let shown = $state<{ entry: string; token: string } | null>(null);
  let error = $state<string | null>(null);

  const load = guardedLoad(
    () => Promise.all([listCpdEntries(workerPid), listMyCredentials()]),
    ([rows, mine]) => {
      entries = rows.filter((e) => e.verified_on);
      issued = mine.credentials;
    },
    (cause) => {
      error = errorText(cause);
    },
  );

  $effect(() => {
    void workerPid;
    void load();
  });

  const liveFor = (entry: string) => issued.find((c) => c.source_pid === entry && !c.revoked_at);

  async function issue(entry: string) {
    error = null;
    try {
      const made = await issueMyCredential(entry);
      shown = { entry, token: made.credential };
      await load();
    } catch (cause) {
      error = errorText(cause);
    }
  }

  async function revoke(pid: string) {
    error = null;
    try {
      await revokeMyCredential(pid);
      shown = null;
      await load();
    } catch (cause) {
      error = errorText(cause);
    }
  }

  const download = $derived(shown ? `data:application/jwt;charset=utf-8,${encodeURIComponent(shown.token)}` : "");
</script>

<section class="panel" data-testid="my-credentials">
  <h2>{t("credentials.title")}</h2>
  <p class="muted">{t("credentials.hint")}</p>
  {#if error}<ErrorNote testid="error" message={error} />{/if}
  {#if entries.length === 0}
    <p class="muted" data-testid="credentials-none">{t("credentials.none")}</p>
  {:else}
    <ul data-testid="credential-list">
      {#each entries as e (e.pid)}
        {@const live = liveFor(e.pid)}
        {@const past = issued.find((c) => c.source_pid === e.pid && c.revoked_at)}
        <li data-testid="credential-row">
          <strong>{e.activity}</strong> <span class="muted">— {formatDate(e.entry_date)}</span>
          {#if live}
            <span data-testid="credential-state">{tf("credentials.issued", { day: formatDate(live.issued_at) })}</span>
            <button type="button" data-testid="credential-revoke" onclick={() => void revoke(live.pid)}>{t("credentials.revoke")}</button>
          {:else}
            {#if past}<span data-testid="credential-state" class="muted">{tf("credentials.revoked", { day: formatDate(past.revoked_at) })}</span>{/if}
            <button type="button" data-testid="credential-issue" onclick={() => void issue(e.pid)}>{t("credentials.issue")}</button>
          {/if}
        </li>
      {/each}
    </ul>
  {/if}
  {#if shown}
    <div data-testid="credential-token-box">
      <p><strong>{t("credentials.once")}</strong></p>
      <textarea readonly rows="4" data-testid="credential-token" aria-label={t("credentials.token")}>{shown.token}</textarea>
      <p>
        <a href={download} download="credential.jwt" data-testid="credential-download">{t("credentials.download")}</a>
        <button type="button" onclick={() => (shown = null)}>{t("credentials.hide")}</button>
      </p>
    </div>
  {/if}
</section>

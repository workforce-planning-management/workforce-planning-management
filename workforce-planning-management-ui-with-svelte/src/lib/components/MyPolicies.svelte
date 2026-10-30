<!--
  My policies: the newest version in force of each policy aimed at the
  worker, with the day it is due and a button to say they have read it. The
  acknowledgement is the worker's own act (`/api/me`), so there is no way to
  acknowledge for someone else. The service holds a reference to the
  document, not its text.
-->
<script lang="ts">
  import { formatDate } from "#lib/locale-format.js";
  import { guardedLoad } from "#lib/loader.svelte.js";
  import { errorText } from "#lib/write.js";
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import { acknowledgePolicy, getMyPolicies } from "#lib/api/wpm.js";
  import type { MyPolicies } from "#lib/api/wpm.js";
  import { t, tf, tv } from "#lib/i18n.svelte.js";

  let { workerPid }: { workerPid: string } = $props();

  let answer = $state<MyPolicies | null>(null);
  let error = $state<string | null>(null);

  const load = guardedLoad(
    () => getMyPolicies(),
    (rows) => {
      answer = rows;
    },
    (cause) => {
      error = errorText(cause);
    },
  );

  $effect(() => {
    void workerPid;
    void load();
  });

  async function acknowledge(versionPid: string) {
    error = null;
    try {
      await acknowledgePolicy(versionPid);
      await load();
    } catch (cause) {
      error = errorText(cause);
    }
  }
</script>

<section class="panel" data-testid="my-policies">
  <h2>{t("myPolicies.title")}</h2>
  <p class="muted">{t("myPolicies.hint")}</p>
  {#if error}<ErrorNote testid="error" message={error} />{/if}
  {#if answer && answer.policies.length === 0}
    <p class="muted" data-testid="policies-none">{t("myPolicies.none")}</p>
  {:else if answer}
    <ul data-testid="policy-list">
      {#each answer.policies as p (p.version.pid)}
        <li data-testid="policy-row">
          <strong>{p.policy}</strong>
          <span class="muted">
            — {tf("myPolicies.version", { version: p.version.version })} · {p.version.document_ref}
            · {tv(p.version.audience)}{#if p.version.audience_value}: {p.version.audience_value}{/if}
          </span>
          <br />
          {#if p.standing.kind === "done"}
            <span data-testid="policy-standing">{tf("myPolicies.done", { day: formatDate(p.standing.on) })}</span>
          {:else}
            <span data-testid="policy-standing">
              {#if p.standing.kind === "overdue"}
                {tf("myPolicies.overdue", { day: formatDate(p.standing.due_on), days: p.standing.calendar_days_late })}
              {:else}
                {tf("myPolicies.due", { day: formatDate(p.standing.due_on) })}
              {/if}
            </span>
            <button type="button" data-testid="policy-acknowledge" onclick={() => void acknowledge(p.version.pid)}>
              {t("myPolicies.acknowledge")}
            </button>
          {/if}
        </li>
      {/each}
    </ul>
  {/if}
</section>

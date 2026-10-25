<!--
  Policies (`/policies`): the organization's policies and their versions, for
  HR. A version holds a reference to the document, never its text, and is
  never edited: a change is a new version that everyone in its audience
  acknowledges again. HR can open the named record of one version (who has
  acknowledged and who has not, by worker number); opening it is audited by
  the service.
-->
<script lang="ts">
  import { page } from "$app/state";
  import { guardedLoad } from "#lib/loader.svelte.js";
  import { errorText } from "#lib/write.js";
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import {
    POLICY_AUDIENCES,
    addPolicyVersion,
    createPolicy,
    getPolicyAcknowledgements,
    listPolicies,
    renamePolicy,
  } from "#lib/api/wpm.js";
  import type { PolicyWithVersions } from "#lib/api/wpm.js";
  import { t, tf, tv } from "#lib/i18n.svelte.js";

  const organizations = $derived((page.data.scope ?? []) as string[]);
  let organization = $state("");
  $effect(() => {
    if (!organization && organizations.length > 0) organization = organizations[0] ?? "";
  });

  let policies = $state<PolicyWithVersions[] | null>(null);
  let error = $state<string | null>(null);
  let newName = $state("");
  let renaming = $state<string | null>(null);
  let renameTo = $state("");
  let target = $state("");
  let title = $state("");
  let documentRef = $state("");
  let effectiveOn = $state(new Date().toISOString().slice(0, 10));
  let audience = $state<string>("all");
  let audienceValue = $state("");
  let days = $state(14);
  let record = $state<Awaited<ReturnType<typeof getPolicyAcknowledgements>> | null>(null);

  const load = guardedLoad(
    () => (organization ? listPolicies(organization) : Promise.resolve({ policies: [] })),
    (answer) => {
      policies = answer.policies;
      error = null;
    },
    (cause) => {
      error = errorText(cause);
    },
  );

  $effect(() => {
    void organization;
    record = null;
    void load();
  });

  async function run(action: () => Promise<unknown>) {
    error = null;
    try {
      await action();
      await load();
    } catch (cause) {
      error = errorText(cause);
    }
  }

  async function open(versionPid: string) {
    error = null;
    try {
      record = await getPolicyAcknowledgements(versionPid);
    } catch (cause) {
      error = errorText(cause);
    }
  }
</script>

<svelte:head><title>{tf("pages.policies.page_title", { policies: t("nav.policies") })}</title></svelte:head>

<h1>{t("nav.policies")}</h1>
<p class="muted">{t("policies.hint")}</p>
{#if error}<ErrorNote testid="error" prefixed message={error} />{/if}

{#if organizations.length === 0}
  <p class="muted">{t("org.noOrganizations")}</p>
{:else}
  <p>
    <label>
      {t("policies.organization")}
      <select bind:value={organization} data-testid="policy-organization">
        {#each organizations as o (o)}<option value={o}>{o}</option>{/each}
      </select>
    </label>
  </p>

  {#if policies === null}
    {#if !error}<p>{t("common.loading")}</p>{/if}
  {:else if policies.length === 0}
    <p class="muted" data-testid="policies-empty">{t("policies.none")}</p>
  {:else}
    <ul data-testid="policy-admin-list">
      {#each policies as p (p.pid)}
        <li data-testid="policy-admin-row">
          <strong>{p.name}</strong>
          {#if renaming === p.pid}
            <input data-testid="policy-rename-input" aria-label={t("policies.name")} bind:value={renameTo} maxlength="120" />
            <button type="button" data-testid="policy-rename-save" disabled={!renameTo.trim()} onclick={() => void run(() => renamePolicy(p.pid, renameTo).then(() => { renaming = null; }))}>{t("policies.rename_save")}</button>
            <button type="button" onclick={() => (renaming = null)}>{t("policies.rename_cancel")}</button>
          {:else}
            <button type="button" data-testid="policy-rename" onclick={() => { renaming = p.pid; renameTo = p.name; }}>{t("policies.rename")}</button>
          {/if}
          <div class="data-grid-scroll">
            <table>
              <thead>
                <tr>
                  <th>{t("policies.version")}</th>
                  <th>{t("policies.title_label")}</th>
                  <th>{t("policies.document_ref")}</th>
                  <th>{t("policies.effective_on")}</th>
                  <th>{t("policies.audience")}</th>
                  <th>{t("policies.within_days")}</th>
                  <th></th>
                </tr>
              </thead>
              <tbody>
                {#each p.versions as v (v.pid)}
                  <tr>
                    <td>{v.version}</td>
                    <td>{v.title}</td>
                    <td>{v.document_ref}</td>
                    <td>{v.effective_on}</td>
                    <td>{tv(v.audience)}{#if v.audience_value}: {v.audience_value}{/if}</td>
                    <td>{v.acknowledge_within_days}</td>
                    <td><button type="button" data-testid="policy-record" onclick={() => void open(v.pid)}>{t("policies.record")}</button></td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        </li>
      {/each}
    </ul>
  {/if}

  {#if record}
    <section class="panel" data-testid="policy-record-panel">
      <h2>{tf("policies.record_of", { title: record.version.title, version: record.version.version })}</h2>
      <p class="muted">{t("policies.record_hint")}</p>
      <ul>
        {#each record.people as person (person.worker_pid)}
          <li data-testid="policy-record-row">
            {person.worker_number}
            — {person.acknowledged_on ? tf("myPolicies.done", { day: person.acknowledged_on }) : t("policies.not_yet")}
          </li>
        {/each}
      </ul>
      <button type="button" onclick={() => (record = null)}>{t("credentials.hide")}</button>
    </section>
  {/if}

  <h2>{t("policies.add_policy")}</h2>
  <p>
    <input data-testid="policy-name" placeholder={t("policies.name")} aria-label={t("policies.name")} bind:value={newName} maxlength="120" />
    <button type="button" data-testid="policy-add" disabled={!newName.trim() || !organization} onclick={() => void run(() => createPolicy({ organization_ref: organization, name: newName }).then(() => { newName = ""; }))}>
      {t("policies.add_policy")}
    </button>
  </p>

  {#if policies && policies.length > 0}
    <h2>{t("policies.add_version")}</h2>
    <form
      onsubmit={(event) => {
        event.preventDefault();
        void run(() =>
          addPolicyVersion(target, {
            title,
            document_ref: documentRef,
            effective_on: effectiveOn,
            audience,
            ...(audience !== "all" && audienceValue ? { audience_value: audienceValue } : {}),
            acknowledge_within_days: days,
          }).then(() => {
            title = "";
            documentRef = "";
          }),
        );
      }}
    >
      <label>
        {t("policies.policy")}
        <select data-testid="policy-target" bind:value={target} required>
          <option value="">—</option>
          {#each policies as p (p.pid)}<option value={p.pid}>{p.name}</option>{/each}
        </select>
      </label>
      <label>{t("policies.title_label")} <input data-testid="policy-title" bind:value={title} required /></label>
      <label>{t("policies.document_ref")} <input data-testid="policy-document" bind:value={documentRef} required /></label>
      <label>{t("policies.effective_on")} <input type="date" bind:value={effectiveOn} required /></label>
      <label>
        {t("policies.audience")}
        <select data-testid="policy-audience" bind:value={audience}>
          {#each POLICY_AUDIENCES as a (a)}<option value={a}>{tv(a)}</option>{/each}
        </select>
      </label>
      {#if audience !== "all"}
        <label>{t("policies.audience_value")} <input data-testid="policy-audience-value" bind:value={audienceValue} required /></label>
      {/if}
      <label>{t("policies.within_days")} <input type="number" min="1" bind:value={days} required /></label>
      <button type="submit" data-testid="policy-version-add">{t("policies.add_version")}</button>
    </form>
  {/if}
{/if}

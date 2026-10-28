<!--
  Equipment: what a worker holds and has held (a laptop, a phone, a pass, a
  key). The worker sees their own list; HR can also add assets, give one to
  the worker, and close an issue as returned or written off with a reason.
  No location, cost or value is held. HR's controls show only when the
  service lets the caller read the organization's assets.
-->
<script lang="ts">
  import { guardedLoad } from "#lib/loader.svelte.js";
  import { errorText } from "#lib/write.js";
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import {
    ASSET_KINDS,
    closeAssetIssue,
    createAsset,
    getWorkerEquipment,
    issueAsset,
    listAssets,
  } from "#lib/api/wpm.js";
  import type { Asset, WorkerEquipment } from "#lib/api/wpm.js";
  import { t, tf, tv } from "#lib/i18n.svelte.js";

  let { workerPid, organizationRef }: { workerPid: string; organizationRef: string } = $props();

  let held = $state<WorkerEquipment | null>(null);
  let assets = $state<Asset[] | null>(null); // null: not HR's to see
  let error = $state<string | null>(null);
  let pick = $state("");
  let newKind = $state<string>("laptop");
  let newTag = $state("");
  let newRole = $state("");
  let closing = $state<string | null>(null);
  let note = $state("");

  const load = guardedLoad(
    async () => {
      const mine = await getWorkerEquipment(workerPid);
      // A refusal is not an error here: the controls just do not show.
      const list: Asset[] | null = await listAssets(organizationRef)
        .then((r) => r.assets)
        .catch(() => null);
      return { mine, list };
    },
    ({ mine, list }) => {
      held = mine;
      assets = list;
    },
    (cause) => {
      error = errorText(cause);
    },
  );

  $effect(() => {
    void workerPid;
    void organizationRef;
    void load();
  });

  async function run(action: () => Promise<unknown>) {
    error = null;
    try {
      await action();
      closing = null;
      note = "";
      await load();
    } catch (cause) {
      error = errorText(cause);
    }
  }

  const free = $derived((assets ?? []).filter((a) => !a.out));
</script>

<section class="panel" data-testid="equipment">
  <h2>{t("equipment.title")}</h2>
  <p class="muted">{t("equipment.hint")}</p>
  {#if error}<ErrorNote testid="error" message={error} />{/if}
  {#if held && held.issues.length === 0}
    <p class="muted" data-testid="equipment-none">{t("equipment.none")}</p>
  {:else if held}
    <p data-testid="equipment-out">{tf("equipment.out", { count: held.out })}</p>
    <ul data-testid="equipment-list">
      {#each held.issues as i (i.issue_pid)}
        <li data-testid="equipment-row">
          <strong>{tv(i.kind)}</strong> {i.tag}
          <span class="muted">
            — {tf("equipment.issued", { day: i.issued_on })}{#if i.returned_on}
              · {tv(i.outcome)} {i.returned_on}{#if i.note} · {i.note}{/if}{/if}
          </span>
          {#if assets && !i.returned_on}
            {#if closing === i.issue_pid}
              <label>
                {t("equipment.note")}
                <input data-testid="equipment-note" bind:value={note} maxlength="300" />
              </label>
              <button type="button" data-testid="equipment-return" onclick={() => void run(() => closeAssetIssue(i.issue_pid, { outcome: "returned", ...(note ? { note } : {}) }))}>
                {tv("returned")}
              </button>
              <button type="button" data-testid="equipment-write-off" disabled={!note.trim()} onclick={() => void run(() => closeAssetIssue(i.issue_pid, { outcome: "written_off", note }))}>
                {tv("written_off")}
              </button>
            {:else}
              <button type="button" data-testid="equipment-close" onclick={() => (closing = i.issue_pid)}>
                {t("equipment.close")}
              </button>
            {/if}
          {/if}
        </li>
      {/each}
    </ul>
  {/if}
  {#if assets}
    <h3>{t("equipment.give")}</h3>
    <p>
      <select data-testid="equipment-pick" bind:value={pick} aria-label={t("equipment.give")}>
        <option value="">—</option>
        {#each free as a (a.pid)}<option value={a.pid}>{tv(a.kind)} {a.tag}</option>{/each}
      </select>
      <button type="button" data-testid="equipment-issue" disabled={!pick} onclick={() => void run(() => issueAsset(pick, workerPid).then(() => (pick = "")))}>
        {t("equipment.issue")}
      </button>
    </p>
    <h3>{t("equipment.add")}</h3>
    <p>
      <select data-testid="equipment-kind" bind:value={newKind} aria-label={t("equipment.kind")}>
        {#each ASSET_KINDS as k (k)}<option value={k}>{tv(k)}</option>{/each}
      </select>
      <input data-testid="equipment-tag" placeholder={t("equipment.tag")} aria-label={t("equipment.tag")} bind:value={newTag} maxlength="60" />
      <input data-testid="equipment-role" placeholder={t("equipment.owner_role")} aria-label={t("equipment.owner_role")} bind:value={newRole} maxlength="100" />
      <button type="button" data-testid="equipment-add" disabled={!newTag.trim() || !newRole.trim()} onclick={() => void run(() => createAsset({ organization_ref: organizationRef, kind: newKind, tag: newTag, owner_role: newRole }).then(() => { newTag = ""; newRole = ""; }))}>
        {t("equipment.add")}
      </button>
    </p>
  {/if}
</section>

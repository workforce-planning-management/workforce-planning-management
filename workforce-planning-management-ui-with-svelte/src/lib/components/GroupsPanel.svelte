<!--
  Groups — communities of practice, communities of interest, and other
  self-organised sets. A person can be in several at once. Joining or leaving is
  recorded with when; leaving keeps the past membership. HR editing someone
  else's record acts on their behalf.
-->
<script lang="ts">
  import { t, tf } from "#lib/i18n.svelte.js";
  import {
    GROUP_KINDS,
    createGroup,
    getWorker,
    joinGroup,
    leaveGroup,
    listGroups,
    workerGroups,
    type Group,
  } from "#lib/api/wpm.js";

  let { workerPid }: { workerPid: string } = $props();

  type Mine = Awaited<ReturnType<typeof workerGroups>>["groups"];
  let mine = $state<Mine>([]);
  let all = $state<Group[]>([]);
  let organization = $state<string | null>(null);
  let error = $state<string | null>(null);
  let choice = $state("");
  let name = $state("");
  let kind = $state<string>("practice");

  const message = (cause: unknown) => (cause instanceof Error ? cause.message : String(cause));
  const kindLabel = (k: string) => (k === "practice" ? t("comp.groupsPanel.kind_practice") : k === "interest" ? t("comp.groupsPanel.kind_interest") : t("comp.groupsPanel.kind_group"));
  const joinable = $derived(all.filter((g) => !mine.some((m) => m.group_pid === g.pid)));

  async function load() {
    try {
      const worker = await getWorker(workerPid);
      organization = worker.organization_ref;
      [mine, all] = await Promise.all([
        workerGroups(workerPid).then((r) => r.groups),
        listGroups(worker.organization_ref),
      ]);
    } catch (cause) {
      error = message(cause);
    }
  }

  $effect(() => {
    void workerPid;
    void load();
  });

  async function run(action: () => Promise<unknown>) {
    error = null;
    try {
      await action();
      await load();
    } catch (cause) {
      error = message(cause);
    }
  }
</script>

<section class="panel" data-testid="groups">
  <h2>{t("comp.groupsPanel.groups")}</h2>
  <p class="muted">{tf("comp.groupsPanel.you_can_join_groups_in_and", { organization: organization ?? t("comp.groupsPanel.your_organization") })}</p>
  {#if error}<p class="error" data-testid="error">{error}</p>{/if}
  <ul data-testid="my-groups">
    {#each mine as g (g.group_pid)}
      <li>
        {g.name} <span class="chip">{kindLabel(g.kind)}</span>
        {#if g.role === "lead"}<span class="chip ok">{t("comp.groupsPanel.lead")}</span>{/if}
        <span class="muted">{tf("comp.groupsPanel.since", { joined_at: g.joined_at.slice(0, 10) })}</span>
        <button type="button" onclick={() => void run(() => leaveGroup(g.group_pid, workerPid))}>{t("comp.groupsPanel.leave")}</button>
      </li>
    {:else}
      <li class="muted">{t("comp.groupsPanel.not_in_any_group_yet")}</li>
    {/each}
  </ul>
  <p>
    <select aria-label={t("comp.groupsPanel.group_to_join")} bind:value={choice}>
      <option value="">{t("comp.groupsPanel.join_a_group")}</option>
      {#each joinable as g (g.pid)}<option value={g.pid}>{g.name} ({g.members})</option>{/each}
    </select>
    <button type="button" disabled={!choice} onclick={() => void run(async () => { await joinGroup(choice, workerPid); choice = ""; })}>{t("comp.groupsPanel.join")}</button>
  </p>
  <form
    onsubmit={(event) => {
      event.preventDefault();
      void run(async () => {
        const made = await createGroup({ organization_ref: organization ?? "", name: name.trim(), kind });
        await joinGroup(made.pid, workerPid, "lead");
        name = "";
      });
    }}
  >
    <label>{t("comp.groupsPanel.start_a_group")} <input bind:value={name} required maxlength="120" /></label>
    <label>
      {t("comp.groupsPanel.kind")}
      <select bind:value={kind}>{#each GROUP_KINDS as k (k)}<option value={k}>{kindLabel(k)}</option>{/each}</select>
    </label>
    <button type="submit">{t("comp.groupsPanel.start")}</button>
  </form>
</section>

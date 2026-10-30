<!--
  Skills crosswalk: statements that a skill declared under one record counts
  as another, each with how sure it is and what it rests on. Anyone signed in
  reads; HR writes. Only exact and close mappings count in gap analysis; the
  others are shown as related and counted for nothing.
-->
<script lang="ts">
  import { arrow } from "#lib/locale-format.js";
  import { guardedLoad } from "#lib/loader.svelte.js";
  import { errorText } from "#lib/write.js";
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import {
    MAPPING_BASES,
    MAPPING_CONFIDENCES,
    createSkillMapping,
    deleteSkillMapping,
    listSkillMappings,
    listSkills,
  } from "#lib/api/wpm.js";
  import type { SkillMapping } from "#lib/api/wpm.js";
  import { t, tv } from "#lib/i18n.svelte.js";

  let mappings = $state<SkillMapping[]>([]);
  let skills = $state<Array<{ pid: string; name: string }>>([]);
  let error = $state<string | null>(null);
  let from = $state("");
  let to = $state("");
  let confidence = $state<string>("exact");
  let basis = $state<string>("authority");
  let source = $state("");

  const load = guardedLoad(
    () => Promise.all([listSkillMappings(), listSkills()]),
    ([answer, catalogue]) => {
      mappings = answer.mappings;
      skills = catalogue;
    },
    (cause) => {
      error = errorText(cause);
    },
  );

  $effect(() => {
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
</script>

<section class="panel" data-testid="crosswalk">
  <h2>{t("crosswalk.title")}</h2>
  <p class="muted">{t("crosswalk.hint")}</p>
  {#if error}<ErrorNote testid="crosswalk-error" message={error} />{/if}
  {#if mappings.length === 0}
    <p class="muted" data-testid="crosswalk-none">{t("crosswalk.none")}</p>
  {:else}
    <div class="data-grid-scroll">
      <table data-testid="crosswalk-table">
        <thead>
          <tr>
            <th>{t("crosswalk.from")}</th>
            <th>{t("crosswalk.to")}</th>
            <th>{t("crosswalk.confidence")}</th>
            <th>{t("crosswalk.basis")}</th>
            <th>{t("crosswalk.source")}</th>
            <th>{t("crosswalk.counts")}</th>
            <th></th>
          </tr>
        </thead>
        <tbody>
          {#each mappings as m (m.pid)}
            <tr data-testid="crosswalk-row">
              <td>{m.from_skill}</td>
              <td>{m.to_skill}</td>
              <td>{tv(m.confidence)}</td>
              <td>{tv(m.basis)}</td>
              <td>{m.source}</td>
              <td>{m.counts ? t("crosswalk.counted") : t("crosswalk.related")}</td>
              <td><button type="button" data-testid="crosswalk-remove" onclick={() => void run(() => deleteSkillMapping(m.pid))}>{t("crosswalk.remove")}</button></td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
  <h3>{t("crosswalk.add")}</h3>
  <p>
    <select data-testid="crosswalk-from" bind:value={from} aria-label={t("crosswalk.from")}>
      <option value="">—</option>
      {#each skills as s (s.pid)}<option value={s.pid}>{s.name}</option>{/each}
    </select>
    {arrow()}
    <select data-testid="crosswalk-to" bind:value={to} aria-label={t("crosswalk.to")}>
      <option value="">—</option>
      {#each skills as s (s.pid)}<option value={s.pid}>{s.name}</option>{/each}
    </select>
    <select data-testid="crosswalk-confidence" bind:value={confidence} aria-label={t("crosswalk.confidence")}>
      {#each MAPPING_CONFIDENCES as c (c)}<option value={c}>{tv(c)}</option>{/each}
    </select>
    <select data-testid="crosswalk-basis" bind:value={basis} aria-label={t("crosswalk.basis")}>
      {#each MAPPING_BASES as b (b)}<option value={b}>{tv(b)}</option>{/each}
    </select>
    <input data-testid="crosswalk-source" placeholder={t("crosswalk.source")} aria-label={t("crosswalk.source")} bind:value={source} />
    <button type="button" data-testid="crosswalk-add" disabled={!from || !to || !source.trim()} onclick={() => void run(() => createSkillMapping({ from_skill_pid: from, to_skill_pid: to, confidence, basis, source }).then(() => { source = ""; }))}>
      {t("crosswalk.add")}
    </button>
  </p>
</section>

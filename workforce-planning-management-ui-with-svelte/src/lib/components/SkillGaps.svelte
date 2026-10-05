<!--
  A person's skill gaps: what they need (their current role's requirements,
  their own targets, and — for them alone — aspirations) against what they
  have declared, ranked by importance × levels short. A skill they have not
  declared is shown as unknown ("Not declared"), never as a gap of some size.
  A development conversation starter, not a selection decision.
-->
<script lang="ts">
  import { workerSkillGaps } from "#lib/api/wpm.js";
  import type { SkillGap } from "#lib/api/types.js";
  import { t } from "#lib/i18n.svelte.js";

  let { workerPid }: { workerPid: string } = $props();

  let gaps = $state<SkillGap[] | null>(null);
  let error = $state<string | null>(null);

  $effect(() => {
    void workerPid;
    void (async () => {
      try {
        gaps = (await workerSkillGaps(workerPid)).gaps;
        error = null;
      } catch (cause) {
        error = cause instanceof Error ? cause.message : String(cause);
      }
    })();
  });

  const source = (s: SkillGap["sources"][number]) =>
    s === "role"
      ? t("skillgaps.sourceRole")
      : s === "target"
        ? t("skillgaps.sourceTarget")
        : t("skillgaps.sourceAspiration");

  const importance = (i: string) =>
    i === "critical"
      ? t("skillgaps.impCritical")
      : i === "important"
        ? t("skillgaps.impImportant")
        : t("skillgaps.impUseful");
</script>

<section class="panel" data-testid="skill-gaps">
  <h2>{t("skillgaps.title")}</h2>
  {#if error}
    <p class="error" data-testid="error">{error}</p>
  {:else if gaps === null}
    <p>{t("common.loading")}</p>
  {:else if gaps.length === 0}
    <p class="muted" data-testid="skill-gaps-none">{t("skillgaps.none")}</p>
  {:else}
    <table data-testid="skill-gaps-table">
      <thead>
        <tr>
          <th>{t("nav.skills")}</th>
          <th>{t("skillgaps.required")}</th>
          <th>{t("skillgaps.declared")}</th>
          <th>{t("skillgaps.short")}</th>
          <th>{t("skillgaps.priority")}</th>
          <th>{t("skillgaps.sources")}</th>
        </tr>
      </thead>
      <tbody>
        {#each gaps as g (g.skill_pid)}
          <tr data-status={g.status}>
            <td>
              <strong>{g.skill ?? g.skill_pid}</strong>
              <span class="muted">· {importance(g.importance)}</span>
            </td>
            <td>{g.required}</td>
            <td>{g.declared ?? "—"}</td>
            <td>
              {#if g.shortfall !== null}{g.shortfall}{:else}<span class="chip" title={t("skillgaps.unknownHint")}>{t("skillgaps.undeclared")}</span>{/if}
            </td>
            <td>{g.status === "below" ? g.priority : "—"}</td>
            <td>{g.sources.map(source).join(", ")}</td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}
</section>

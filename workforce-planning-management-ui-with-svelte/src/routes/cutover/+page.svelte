<!--
  Cutover readiness (`/cutover`, WPM-R70): for each planned go-live of an organization, the people
  side — trained staff, hypercare cover, key-person risk — each `ready`, `at risk` or `unknown`, and
  the worst of them. An unknown is never ready. The reasons are written by the service, in English.
  Whether to go live is a human decision outside this service. No person is named.
-->
<script lang="ts">
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import {
    addCutoverPool,
    addCutoverRole,
    createCutover,
    cutoverReadiness,
    listCutovers,
    listRoleProfiles,
    listSkillPools,
    removeCutover,
    type Cutover,
    type CutoverReadiness,
    type Readiness,
    type SkillPool,
  } from "#lib/api/wpm.js";
  import OrgPicker from "#lib/components/OrgPicker.svelte";
  import WriteResult from "#lib/components/WriteResult.svelte";
  import { t, tf, tp } from "#lib/i18n.svelte.js";
  import { failure, type Failure, errorText } from "#lib/write.js";
  import { stateLabel } from "#lib/stateLabel.js";

  let org = $state("");
  let cutovers = $state<Cutover[] | null>(null);
  let readiness = $state<Record<string, CutoverReadiness>>({});
  let error = $state<string | null>(null);

  let pools = $state<SkillPool[]>([]);
  let roleProfiles = $state<Array<{ pid: string; job_title: string }>>([]);
  let failed = $state<Failure | null>(null);
  let reload = $state(0);

  const message = (cause: unknown) => (errorText(cause));

  $effect(() => {
    const chosen = org;
    void reload;
    if (!chosen) return;
    let current = true;
    void (async () => {
      try {
        const list = await listCutovers(chosen);
        const loaded = await Promise.all(list.map((c) => cutoverReadiness(c.pid)));
        if (!current) return;
        cutovers = list;
        readiness = Object.fromEntries(loaded.map((r) => [r.cutover.pid, r]));
        error = null;
      } catch (cause) {
        if (current) error = message(cause);
      }
    })();
    return () => {
      current = false;
    };
  });

  // What the planning forms choose from; a caller who may not read them just gets empty lists.
  $effect(() => {
    const chosen = org;
    if (!chosen) return;
    let current = true;
    void listSkillPools(chosen)
      .then((p) => current && (pools = p))
      .catch(() => current && (pools = []));
    void listRoleProfiles()
      .then((p) => current && (roleProfiles = p))
      .catch(() => current && (roleProfiles = []));
    return () => {
      current = false;
    };
  });

  /** Run a planning write, then read everything again. */
  async function write(action: () => Promise<unknown>) {
    failed = null;
    try {
      await action();
      reload += 1;
    } catch (cause) {
      failed = failure(cause);
    }
  }

  // The new-cutover form.
  let programme = $state("");
  let name = $state("");
  let site = $state("");
  let start = $state("");
  let end = $state("");
  let aliases = $state("");
  let rota = $state("");

  const aliasList = () =>
    aliases
      .split(",")
      .map((a) => a.trim())
      .filter((a) => a.length > 0);

  // The add-role and add-pool forms, one set for whichever cutover they are used under.
  let roleProfile = $state("");
  let course = $state("");
  let headcount = $state(1);
  let poolChoice = $state("");

  /** A reason in the reader's language, from the service's data; its English text when the code is new. */
  function reasonText(
    check: CutoverReadiness["checks"][keyof CutoverReadiness["checks"]],
    i: number,
  ): { text: string; translated: boolean } {
    const d = check.details?.[i];
    const said = d ? tp(`cut.why.${d.code}`, d.params) : null;
    return said === null
      ? { text: check.reasons[i] ?? "", translated: false }
      : { text: said, translated: true };
  }

  const CHECKS = [
    ["trained_staff", "cut.trainedStaff"],
    ["hypercare_cover", "cut.hypercare"],
    ["key_person_risk", "cut.keyPerson"],
  ] as const;

  /** A mark as well as a colour, so the state is never colour alone. */
  const MARK: Record<Readiness, string> = { ready: "✓", at_risk: "✕", unknown: "?" };
</script>

<svelte:head><title>{t("titles.cutover")}</title></svelte:head>

<h1>{t("cut.title")}</h1>
<p class="muted">{t("cut.hint")}</p>

<div class="form"><OrgPicker bind:value={org} /></div>

{#if error}
  <ErrorNote testid="error" prefixed message={error} />
{:else if !org}
  <p class="muted">{t("common.noOrganization")}</p>
{:else if cutovers === null}
  <p>{t("common.loading")}</p>
{:else if cutovers.length === 0}
  <p class="muted" data-testid="no-cutovers">{t("cut.none")}</p>
{:else}
  {#if cutovers.some((c) => CHECKS.some(([key]) => readiness[c.pid]?.checks[key].reasons.some((_, i) => !reasonText(readiness[c.pid]!.checks[key], i).translated)))}
    <p class="muted small">{t("cut.reasonsEnglish")}</p>
  {/if}
  {#each cutovers as c (c.pid)}
    {@const r = readiness[c.pid]}
    <section class="panel" data-testid="cutover">
      <h2>
        {c.name}
        {#if r}
          <span class="state {r.readiness}" data-testid="overall" data-status={r.readiness}>
            <span aria-hidden="true">{MARK[r.readiness]}</span>
            {t("cut.overall")}: {stateLabel(r.readiness)}
          </span>
        {/if}
      </h2>
      <p class="muted">
        {tf("cut.site", { site: c.site })} · {tf("cut.window", {
          start: c.window_start,
          end: c.window_end,
        })}
      </p>
      {#if r}
        <ul class="checks">
          {#each CHECKS as [key, label] (key)}
            {@const check = r.checks[key]}
            <li data-testid="check-{key}" data-status={check.status}>
              <span class="state {check.status}">
                <span aria-hidden="true">{MARK[check.status]}</span>
                {t(label)}: {stateLabel(check.status)}
              </span>
              {#if check.reasons.length === 0}
                <span class="muted">{t("cut.noConcerns")}</span>
              {:else}
                <ul>
                  {#each check.reasons as _, i (i)}
                    <li>{reasonText(check, i).text}</li>
                  {/each}
                </ul>
              {/if}
            </li>
          {/each}
        </ul>
        <details>
          <summary>{t("cap.basis")}</summary>
          <p class="muted">{t("cut.basisText")}</p>
          <p class="muted small">{t("common.fromService")}: {r.basis}</p>
        </details>
      {/if}
      <details>
        <summary>{t("cut.addRole")}</summary>
        <form
          class="form"
          onsubmit={(e) => {
            e.preventDefault();
            void write(() =>
              addCutoverRole(c.pid, {
                role_profile_pid: roleProfile,
                course_ref: course,
                required_headcount: headcount,
              }),
            );
          }}
        >
          <label>
            {t("cut.roleProfile")}
            <select bind:value={roleProfile} required data-testid="role-profile">
              <option value="" disabled></option>
              {#each roleProfiles as p (p.pid)}<option value={p.pid}>{p.job_title}</option>{/each}
            </select>
          </label>
          <label>
            {t("cut.course")}
            <input bind:value={course} required data-testid="role-course" />
          </label>
          <label>
            {t("cut.headcount")}
            <input type="number" min="1" bind:value={headcount} required />
          </label>
          <button type="submit" data-testid="add-role">{t("common.add")}</button>
        </form>
      </details>
      <details>
        <summary>{t("cut.addPool")}</summary>
        <form
          class="form"
          onsubmit={(e) => {
            e.preventDefault();
            void write(() => addCutoverPool(c.pid, poolChoice));
          }}
        >
          <label>
            {t("cap.pool")}
            <select bind:value={poolChoice} required data-testid="pool-choice">
              <option value="" disabled></option>
              {#each pools as p (p.pid)}<option value={p.pid}>{p.name}</option>{/each}
            </select>
          </label>
          <button type="submit" data-testid="add-pool">{t("common.add")}</button>
        </form>
      </details>
      <button
        type="button"
        data-testid="remove-cutover"
        onclick={() => {
          if (confirm(t("cut.removeConfirm"))) void write(() => removeCutover(c.pid));
        }}
      >
        {t("cut.remove")}
      </button>
    </section>
  {/each}
{/if}

{#if org}
  <section class="panel" data-testid="plan-cutover">
    <h2>{t("cut.plan")}</h2>
    <form
      class="form"
      onsubmit={(e) => {
        e.preventDefault();
        void write(() =>
          createCutover({
            organization_ref: org,
            programme_ref: programme,
            name,
            site,
            window_start: start,
            window_end: end,
            rota_pid: rota.trim() === "" ? null : rota.trim(),
            site_aliases: aliasList(),
          }),
        );
      }}
    >
      <label>{t("common.programme")} <input bind:value={programme} required data-testid="cut-programme" /></label>
      <label>{t("common.name")} <input bind:value={name} required data-testid="cut-name" /></label>
      <label>{t("cut.siteLabel")} <input bind:value={site} required data-testid="cut-site" /></label>
      <label>{t("cut.windowStart")} <input type="date" bind:value={start} required data-testid="cut-start" /></label>
      <label>{t("cut.windowEnd")} <input type="date" bind:value={end} required data-testid="cut-end" /></label>
      <label>{t("cut.aliases")} <input bind:value={aliases} data-testid="cut-aliases" /></label>
      <label>{t("cut.rotaRef")} <input bind:value={rota} /></label>
      <button type="submit" data-testid="plan-create">{t("cut.create")}</button>
    </form>
    <WriteResult {failed} />
  </section>
{/if}

<style>
  .small {
    font-size: 0.82rem;
  }
  .checks {
    list-style: none;
    padding: 0;
  }
  .checks > li {
    margin-bottom: 0.6rem;
  }
  .checks ul {
    margin: 0.2rem 0 0;
    padding-inline-start: 1.1rem;
  }
  .state {
    display: inline-block;
    border: 1px solid var(--line);
    border-radius: 0.4rem;
    padding: 0.05rem 0.5rem;
    font-size: 0.85rem;
    font-weight: 600;
    color: var(--ink);
  }
  h2 .state {
    margin-inline-start: 0.5rem;
  }
  .ready {
    background: color-mix(in srgb, var(--state-available) 20%, var(--panel));
    border-color: var(--state-available);
  }
  .at_risk {
    background: color-mix(in srgb, var(--state-closed) 25%, var(--panel));
    border-color: var(--state-closed);
  }
  .unknown {
    border: 1px dashed var(--state-reserved);
    background: color-mix(in srgb, var(--state-reserved) 18%, var(--panel));
  }
</style>

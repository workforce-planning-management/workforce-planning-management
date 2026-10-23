<!--
  Equality monitoring (WPM-R125): voluntary, self-declared, off until a deployer records a lawful basis. Only the
  worker ever sees an answer; it is used only to count by group, never to decide anything about a person. Every
  category offers "prefer not to say", and an answer can be changed or taken back at any time. The categories and
  their values are the deployer's own wording and are shown as the service sends them.
-->
<script lang="ts">
  import { errorText } from "#lib/write.js";
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import {
    declareMyEquality,
    getMyEquality,
    withdrawMyEquality,
    type MyEquality,
  } from "#lib/api/wpm.js";
  import { t, tf, tv } from "#lib/i18n.svelte.js";

  let view = $state<MyEquality | null>(null);
  let error = $state<string | null>(null);
  let chosen = $state<Record<string, string>>({});

  const message = (cause: unknown) => (errorText(cause));

  async function load() {
    try {
      view = await getMyEquality();
      chosen = view.enabled ? { ...view.declarations } : {};
    } catch (cause) {
      error = message(cause);
    }
  }
  $effect(() => void load());

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

<section class="panel" data-testid="my-equality">
  <h2>{t("details.equality.title")}</h2>
  {#if error}<ErrorNote testid="equality-error" message={error} />{/if}
  {#if view && !view.enabled}
    <p class="muted" data-testid="equality-off">{t("details.equality.off")}</p>
  {:else if view?.enabled}
    <p class="muted" data-testid="equality-notice">{view.notice}</p>
    <p class="muted">{tf("details.equality.basis", { basis: view.lawful_basis, floor: view.small_group_floor })}</p>
    <form
      onsubmit={(event) => {
        event.preventDefault();
        void run(() => declareMyEquality(Object.fromEntries(Object.entries(chosen).filter(([, v]) => v))));
      }}
    >
      {#each Object.entries(view.categories) as [category, values] (category)}
        <label>
          {tv(category)}
          <select data-testid="equality-{category}" bind:value={chosen[category]}>
            <option value="">{t("details.equality.unanswered")}</option>
            {#each values as v (v)}<option value={v}>{tv(v)}</option>{/each}
          </select>
        </label>
        {#if view.declarations[category]}
          <button type="button" data-testid="equality-take-back-{category}" onclick={() => void run(() => withdrawMyEquality(category))}>
            {t("details.equality.take_back")}
          </button>
        {/if}
      {/each}
      <button type="submit" data-testid="equality-save">{t("common.save")}</button>
      {#if Object.keys(view.declarations).length > 0}
        <button type="button" data-testid="equality-take-back-all" onclick={() => void run(() => withdrawMyEquality())}>
          {t("details.equality.take_back_all")}
        </button>
      {/if}
    </form>
  {/if}
</section>

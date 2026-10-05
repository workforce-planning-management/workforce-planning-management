<!--
  My on-call: the stretches this person is on call across the on-call
  rotas they can read, over the next four weeks.
-->
<script lang="ts">
  import { workerOnCall } from "#lib/api/wpm.js";
  import { t, l } from "#lib/i18n.svelte.js";

  let { workerPid }: { workerPid: string } = $props();

  let stretches = $state<Awaited<ReturnType<typeof workerOnCall>> | null>(null);

  $effect(() => {
    void workerPid;
    void (async () => {
      try {
        stretches = await workerOnCall(workerPid);
      } catch {
        // Not shown rather than an error: a person without rota access
        // simply has no on-call panel.
        stretches = null;
      }
    })();
  });
</script>

{#if stretches}
  <section class="panel" data-testid="my-on-call">
    <h2>{t("rota.mine")}</h2>
    {#if stretches.length === 0}
      <p class="muted">{t("rota.mineNone")}</p>
    {:else}
      <ul>
        {#each stretches as s (s.rota_pid + s.from)}
          <li>
            <a href={l("/rota")}>{s.rota_name}</a>: {s.from} → {s.to}
          </li>
        {/each}
      </ul>
    {/if}
  </section>
{/if}

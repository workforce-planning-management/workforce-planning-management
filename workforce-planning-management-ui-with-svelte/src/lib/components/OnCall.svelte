<!--
  My on-call: the stretches this person is on call across the on-call
  rotas they can read, over the next four weeks.
-->
<script lang="ts">
  import { decideSwap, workerOnCall, workerSwapRequests } from "#lib/api/wpm.js";
  import type { SwapRequest } from "#lib/api/types.js";
  import { t, l } from "#lib/i18n.svelte.js";

  let { workerPid }: { workerPid: string } = $props();

  let stretches = $state<Awaited<ReturnType<typeof workerOnCall>> | null>(null);
  let incoming = $state<SwapRequest[]>([]);
  let outgoing = $state<SwapRequest[]>([]);
  let error = $state<string | null>(null);

  async function load() {
    try {
      const [mine, swaps] = await Promise.all([
        workerOnCall(workerPid),
        workerSwapRequests(workerPid),
      ]);
      stretches = mine;
      incoming = swaps.incoming;
      outgoing = swaps.outgoing;
    } catch {
      // Not shown rather than an error: a person without rota access
      // simply has no on-call panel.
      stretches = null;
    }
  }

  $effect(() => {
    void workerPid;
    void load();
  });

  async function decide(pid: string, decision: "accept" | "decline" | "cancel") {
    error = null;
    try {
      await decideSwap(pid, decision);
      await load();
    } catch (cause) {
      error = cause instanceof Error ? cause.message : String(cause);
    }
  }
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
    {#if error}<p class="error" data-testid="error">{error}</p>{/if}
    {#if incoming.length > 0}
      <h3>{t("rota.swapIncoming")}</h3>
      <ul data-testid="swap-incoming">
        {#each incoming as r (r.pid)}
          <li>
            <strong>{r.requester_name ?? r.requester_pid}</strong>
            {t("rota.asks")} — {r.rota_name}: {r.starts_on} → {r.ends_on}
            {#if r.note}<span class="muted">— {r.note}</span>{/if}
            <button type="button" onclick={() => void decide(r.pid, "accept")}>{t("rota.accept")}</button>
            <button type="button" onclick={() => void decide(r.pid, "decline")}>{t("rota.decline")}</button>
          </li>
        {/each}
      </ul>
    {/if}
    {#if outgoing.length > 0}
      <h3>{t("rota.swapOutgoing")}</h3>
      <ul data-testid="swap-outgoing">
        {#each outgoing as r (r.pid)}
          <li>
            {r.taker_name ?? r.taker_pid} — {r.rota_name}: {r.starts_on} → {r.ends_on}
            <button type="button" onclick={() => void decide(r.pid, "cancel")}>{t("rota.cancel")}</button>
          </li>
        {/each}
      </ul>
    {/if}
  </section>
{/if}

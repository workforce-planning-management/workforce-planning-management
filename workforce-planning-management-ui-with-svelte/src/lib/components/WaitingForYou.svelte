<!--
  What is waiting for you (WPM-R232): your unread notifications about a goal or a PADR, each linked to the page where it is done. They
  say what is waiting and nothing more (WPM-D23). With nothing waiting, the panel is not there.
-->
<script lang="ts">
  import { page } from "$app/state";
  import { guardedLoad } from "#lib/loader.svelte.js";
  import { errorText } from "#lib/write.js";
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import { listNotifications, markNotificationRead, type Notification } from "#lib/api/wpm.js";
  import type { MyOrganization } from "#lib/api/types.js";
  import { l, t } from "#lib/i18n.svelte.js";

  const me = $derived(
    ((page.data.organizations ?? []) as MyOrganization[]).find((m) => m.worker_pid !== null)?.worker_pid ?? null,
  );

  let rows = $state<Notification[]>([]);
  let error = $state<string | null>(null);

  function padrLink(row: Notification): string {
    return row.data["padr_pid"] ? `/me/padr/${String(row.data["padr_pid"])}` : "/me/padr";
  }

  const shown: Record<string, { text: () => string; href: (row: Notification) => string }> = {
    goal_waiting: { text: () => t("notif.goalWaiting"), href: () => "/me/goals" },
    padr_opened: { text: () => t("notif.padrOpened"), href: padrLink },
    padr_to_sign: { text: () => t("notif.padrToSign"), href: padrLink },
  };

  const load = guardedLoad(
    async () => (me ? await listNotifications(me) : []),
    (found) => {
      rows = found.filter((row) => row.read_at === null && row.kind in shown);
    },
    (cause) => {
      error = errorText(cause);
    },
  );

  $effect(() => {
    void me;
    void load();
  });

  async function dismiss(pid: string) {
    try {
      await markNotificationRead(pid);
      await load();
    } catch (cause) {
      error = errorText(cause);
    }
  }
</script>

{#if error}<ErrorNote testid="waiting-error" message={error} />{/if}
{#if rows.length > 0}
  <section class="panel" data-testid="waiting-for-you">
    <h2>{t("notif.waitingForYou")}</h2>
    <ul>
      {#each rows as row (row.pid)}
        <li data-testid="waiting-item">
          <a href={l(shown[row.kind]?.href(row) ?? "/me")}>{shown[row.kind]?.text()}</a>
          <button type="button" onclick={() => void dismiss(row.pid)}>{t("notif.markRead")}</button>
        </li>
      {/each}
    </ul>
  </section>
{/if}

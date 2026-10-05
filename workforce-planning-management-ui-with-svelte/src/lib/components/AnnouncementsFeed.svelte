<!--
  The announcement feed: live posts, pinned first then newest. Posts are plain
  text, shown with line breaks — never as markup. `limit` shows just the latest
  few (the dashboard); `manage` adds the editors' tools: scheduled and expired
  posts, retiring, and the post form (the service refuses anyone who is not an
  editor of the organization).
-->
<script lang="ts">
  import { page } from "$app/state";
  import {
    listAnnouncements,
    markAnnouncementRead,
    myAnnouncementReads,
    postAnnouncement,
    retireAnnouncement,
  } from "#lib/api/wpm.js";
  import type { Announcement } from "#lib/api/types.js";
  import { t, l } from "#lib/i18n.svelte.js";

  let { limit, manage = false }: { limit?: number; manage?: boolean } = $props();

  const organizationRefs = $derived((page.data.scope ?? []) as string[]);
  // The reader's own worker record, for read receipts.
  const workerPid = $derived(
    ((page.data.organizations ?? []) as Array<{ worker_pid: string | null }>).find((m) => m.worker_pid)
      ?.worker_pid ?? "",
  );
  let read = $state<Set<string>>(new Set());

  let posts = $state<Announcement[] | null>(null);
  let showAll = $state(false);
  let error = $state<string | null>(null);
  let title = $state("");
  let body = $state("");
  let pinned = $state(false);
  let publishOn = $state("");
  let expiresOn = $state("");
  let organization = $state("");
  let department = $state("");
  let links = $state([
    { label: "", url: "" },
    { label: "", url: "" },
    { label: "", url: "" },
  ]);

  const org = $derived(organization || organizationRefs[0] || "");
  const message = (cause: unknown) => (cause instanceof Error ? cause.message : String(cause));

  async function load() {
    try {
      posts = await listAnnouncements({ limit, includeAll: manage && showAll });
      if (workerPid) {
        try {
          read = new Set(await myAnnouncementReads(workerPid));
        } catch {
          // No read state is not an error: nothing is flagged unread.
          read = new Set();
        }
      }
    } catch {
      // The feed is an extra on the dashboard: a failure shows no feed
      // rather than an error over the page's real content.
      posts = manage ? posts : [];
    }
  }

  $effect(() => {
    void showAll;
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

  const unread = (a: Announcement) => workerPid !== "" && !read.has(a.pid);

  const safeLink = (url: string) => url.startsWith("https://");

  const statusLabel = (s: Announcement["status"]) =>
    s === "scheduled" ? t("announcements.scheduled") : s === "expired" ? t("announcements.expired") : "";
</script>

<section class="panel" data-testid="announcements">
  <h2>{limit ? t("announcements.latest") : t("nav.announcements")}</h2>
  {#if error}<p class="error" data-testid="error">{error}</p>{/if}
  {#if manage}
    <label>
      <input type="checkbox" bind:checked={showAll} data-testid="announcements-all" />
      {t("announcements.all")}
    </label>
  {/if}
  {#if posts === null}
    <p>{t("common.loading")}</p>
  {:else if posts.length === 0}
    <p class="muted" data-testid="announcements-none">{t("announcements.none")}</p>
  {:else}
    <ul class="feed" data-testid="announcement-list">
      {#each posts as a (a.pid)}
        <li>
          <h3>
            {#if a.pinned}<span class="chip">{t("announcements.pinned")}</span>{/if}
            {#if a.status !== "live"}<span class="chip">{statusLabel(a.status)}</span>{/if}
            {#if a.department}<span class="chip">{a.department}</span>{/if}
            {#if unread(a)}<span class="chip" data-testid="unread">{t("announcements.unread")}</span>{/if}
            {a.title}
          </h3>
          <p class="muted">{a.publish_on}{#if a.expires_on} → {a.expires_on}{/if}</p>
          <!-- Plain text with line breaks; Svelte escapes it, so a post can never carry markup. -->
          <p class="body">{a.body}</p>
          {#if a.links.length > 0}
            <ul class="links" data-testid="announcement-links">
              {#each a.links as link (link.url)}
                {#if safeLink(link.url)}
                  <li><a href={link.url} target="_blank" rel="noopener noreferrer">{link.label}</a></li>
                {/if}
              {/each}
            </ul>
          {/if}
          {#if unread(a)}
            <button
              type="button"
              data-testid="mark-read"
              onclick={() =>
                void run(async () => {
                  await markAnnouncementRead(a.pid, workerPid);
                })}
            >
              {t("announcements.markRead")}
            </button>
          {/if}
          {#if a.read_count !== undefined}
            <span class="muted" data-testid="read-count">{a.read_count} {t("announcements.readBy")}</span>
          {/if}
          {#if manage}
            <button type="button" onclick={() => void run(() => retireAnnouncement(a.pid))}>
              {t("announcements.retire")}
            </button>
          {/if}
        </li>
      {/each}
    </ul>
  {/if}
  {#if limit}
    <p><a href={l("/announcements")}>{t("announcements.seeAll")}</a></p>
  {/if}

  {#if manage}
    <details>
      <summary>{t("announcements.post")}</summary>
      <form
        data-testid="announcement-form"
        onsubmit={(event) => {
          event.preventDefault();
          void run(async () => {
            await postAnnouncement({
              organization_ref: org,
              title: title.trim(),
              body: body.trim(),
              pinned,
              ...(publishOn ? { publish_on: publishOn } : {}),
              ...(expiresOn ? { expires_on: expiresOn } : {}),
              ...(department.trim() ? { department: department.trim() } : {}),
              links: links
                .map((l) => ({ label: l.label.trim(), url: l.url.trim() }))
                .filter((l) => l.label || l.url),
            });
            title = body = publishOn = expiresOn = department = "";
            links = links.map(() => ({ label: "", url: "" }));
            pinned = false;
          });
        }}
      >
        <label>
          {t("announcements.organization")}
          <select bind:value={organization}>
            {#each organizationRefs as ref (ref)}<option value={ref}>{ref}</option>{/each}
          </select>
        </label>
        <label>{t("announcements.titleLabel")} <input bind:value={title} required maxlength="200" data-testid="announcement-title" /></label>
        <label>{t("announcements.body")} <textarea bind:value={body} required maxlength="5000" rows="4" data-testid="announcement-body"></textarea></label>
        <label><input type="checkbox" bind:checked={pinned} /> {t("announcements.pin")}</label>
        <label>{t("announcements.publishOn")} <input type="date" bind:value={publishOn} /></label>
        <label>{t("announcements.expiresOn")} <input type="date" bind:value={expiresOn} /></label>
        <label>{t("announcements.department")} <input bind:value={department} maxlength="100" data-testid="announcement-department" /></label>
        <fieldset>
          <legend>{t("announcements.links")}</legend>
          {#each links as link, i (i)}
            <label>{t("announcements.linkLabel")} <input bind:value={link.label} maxlength="100" /></label>
            <label>{t("announcements.linkUrl")} <input type="url" bind:value={link.url} maxlength="500" data-testid={`announcement-link-url-${i}`} /></label>
          {/each}
        </fieldset>
        <button type="submit" data-testid="announcement-post">{t("announcements.post")}</button>
      </form>
    </details>
  {/if}
</section>

<style>
  .feed {
    list-style: none;
    padding: 0;
  }
  .feed li {
    margin-block: 0.75rem;
  }
  .body {
    white-space: pre-line;
  }
  .links {
    list-style: none;
    padding: 0;
    display: flex;
    gap: 1rem;
    flex-wrap: wrap;
  }
</style>

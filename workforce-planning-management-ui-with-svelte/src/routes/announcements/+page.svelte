<!--
  Announcements (`/announcements`): the full feed, with the editors' tools
  (post, retire, scheduled and expired posts). Posts are plain text.
-->
<script lang="ts">
  import AnnouncementsFeed from "#lib/components/AnnouncementsFeed.svelte";
  import { t, tf } from "#lib/i18n.svelte.js";
</script>

<svelte:head><title>{tf("pages.announcements.page_title", { announcements: t("nav.announcements") })}</title></svelte:head>

<h1>{t("nav.announcements")}</h1>
<AnnouncementsFeed manage />

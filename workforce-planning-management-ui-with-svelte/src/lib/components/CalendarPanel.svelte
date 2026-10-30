<!--
  A calendar over `@lilydesignsystem/svelte-calendar`: week, four weeks or month, in the reader's
  language. Events carry a `title` that is always shown as text (never colour alone); `ranges`
  mark periods (a CPD requirement window). It only shows what it is given, so a day with nothing
  to show is empty, not "0". Weekday and month names come from Intl, not the catalogue.
-->
<script lang="ts">
  import CalendarView from "@lilydesignsystem/svelte-calendar";
  import type {
    CalendarEvent,
    CalendarEventField,
    CalendarRange,
    CalendarViewMode,
  } from "@lilydesignsystem/svelte-calendar";
  import { i18n, t, tf } from "#lib/i18n.svelte.js";
  import { formatDateRange } from "#lib/locale-format.js";
  import { browserLanguages, calendarLocale, todayIso, weekInfo } from "#lib/reader-calendar.js";

  let {
    label,
    events,
    ranges = [],
    fields,
    testid,
    initialDate,
  }: {
    label: string;
    events: CalendarEvent[];
    ranges?: CalendarRange[];
    /** Typed values an event carries in `values`, shown after its title in the agenda. */
    fields?: CalendarEventField[];
    testid?: string;
    /** ISO date the first view is built around; defaults to today. */
    initialDate?: string;
  } = $props();

  // Today in the reader's own time zone, not UTC; the week's first day and weekend from CLDR for the reader's region
  // (WPM-T378, WPM-D149). The Lily calendar reads the first day from the locale tag, so it gets the page's locale
  // with the CLDR `fw` keyword set; the weekend it takes as weekday numbers.
  const today = todayIso();
  const week = $derived(weekInfo(i18n.locale, browserLanguages()));
  const locale = $derived(calendarLocale(i18n.locale, week));
  // A phone has no room for a month grid: start on the agenda there.
  const narrow = (() => {
    try {
      return window.matchMedia("(max-width: 600px)").matches;
    } catch {
      return false;
    }
  })();
  let view = $state<CalendarViewMode>(narrow ? "agenda" : "month");
  // svelte-ignore state_referenced_locally
  let anchorDate = $state(initialDate ?? today);

  // 2023-01-01 was a Sunday, so day 1 + weekday is a UTC date with that weekday.
  const weekdayName = (weekday: number, style: "long" | "short") =>
    new Intl.DateTimeFormat(i18n.locale, { weekday: style, timeZone: "UTC" }).format(
      new Date(Date.UTC(2023, 0, 1 + weekday)),
    );
  const fullDate = (iso: string) =>
    new Intl.DateTimeFormat(i18n.locale, { dateStyle: "full", timeZone: "UTC" }).format(
      new Date(`${iso}T00:00:00Z`),
    );
  const shortDate = (iso: string) =>
    new Intl.DateTimeFormat(i18n.locale, { dateStyle: "medium", timeZone: "UTC" }).format(
      new Date(`${iso}T00:00:00Z`),
    );

  const labels = $derived({
    weekView: t("calendar.week"),
    fourWeekView: t("calendar.fourWeeks"),
    monthView: t("calendar.month"),
    agendaView: t("calendar.agenda"),
    agendaEmpty: t("calendar.agendaEmpty"),
    previous: () => t("calendar.previous"),
    next: () => t("calendar.next"),
    today: t("calendar.today"),
    periodAnnouncement: (_view: string, start: string, end: string) =>
      tf("calendar.announcement", { start: shortDate(start), end: shortDate(end) }),
    periodTitle: (v: string, start: string, end: string) =>
      v === "month" || v === "agenda"
        ? new Intl.DateTimeFormat(i18n.locale, { month: "long", year: "numeric", timeZone: "UTC" }).format(
            new Date(`${anchorDate}T00:00:00Z`),
          )
        : formatDateRange(start, end),
    weekday: (weekday: number) => weekdayName(weekday, "long"),
    weekdayAbbr: (weekday: number) => weekdayName(weekday, "short"),
    day: (iso: string, _weekday: number, outside: boolean) =>
      outside ? `${fullDate(iso)} ·` : fullDate(iso),
  });
</script>

<div class="cx-calendar" data-testid={testid}>
  <CalendarView
    {label}
    bind:view
    bind:anchorDate
    {events}
    {ranges}
    eventFields={fields}
    {today}
    {locale}
    weekendDays={week.weekend}
    maxEvents={3}
    {labels}
  >
    {#snippet day({ date, visibleEvents, hiddenCount, ranges: dayRanges })}
      {#if view !== "agenda"}<span class="cx-calendar-date">{Number(date.slice(8))}</span>{/if}
      {#each dayRanges as range (range.label ?? range.kind)}
        {#if range.label}
          <span class="cx-calendar-range">{range.label}</span>
        {/if}
      {/each}
      {#each visibleEvents as event (event.id)}
        <span class="cx-calendar-event">{event.title}</span>
      {/each}
      {#if hiddenCount > 0}
        <span class="cx-calendar-more">{tf("calendar.more", { count: hiddenCount })}</span>
      {/if}
    {/snippet}
  </CalendarView>
</div>

<style>
  .cx-calendar {
    margin: 0.5rem 0 1rem;
    overflow-x: auto;
  }
  .cx-calendar :global(.calendar-view-toolbar) {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem 1rem;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 0.5rem;
  }
  .cx-calendar :global(.calendar-view-nav) {
    display: flex;
    gap: 0.5rem;
    align-items: center;
  }
  .cx-calendar :global(.calendar-view-switcher-button[aria-pressed="true"]),
  .cx-calendar :global(.calendar-view-switcher-button[aria-current="true"]) {
    font-weight: 700;
    text-decoration: underline;
  }
  .cx-calendar :global(.calendar-view-table) {
    table-layout: fixed;
    width: 100%;
  }
  .cx-calendar :global(td) {
    vertical-align: top;
    height: 5.5rem;
    border: 1px solid var(--line);
    padding: 0.2rem 0.3rem;
    font-size: 0.8rem;
  }
  .cx-calendar :global(td[data-outside-period]) {
    opacity: 0.55;
  }
  .cx-calendar :global(td[data-weekend]) {
    background: var(--panel);
  }
  .cx-calendar :global(td:focus-visible) {
    outline: 2px solid currentColor;
    outline-offset: -2px;
  }
  .cx-calendar :global(.calendar-view-status) {
    position: absolute;
    width: 1px;
    height: 1px;
    overflow: hidden;
    clip-path: inset(50%);
  }
  /* Today is outlined and its date underlined, so it reads in both themes without a fill. */
  .cx-calendar :global(td[aria-current="date"]) {
    background: transparent;
    color: inherit;
    outline: 2px solid currentColor;
    outline-offset: -2px;
  }
  .cx-calendar :global(td[aria-current="date"] .cx-calendar-date) {
    text-decoration: underline;
  }
  @media (max-width: 600px) {
    .cx-calendar :global(td) {
      height: 4.5rem;
      padding: 0.1rem;
      font-size: 0.7rem;
    }
    .cx-calendar :global(th) {
      padding: 0.2rem 0;
      font-size: 0.6rem;
      letter-spacing: 0;
      text-align: center;
    }
  }
  .cx-calendar-date {
    display: block;
    font-weight: 600;
  }
  .cx-calendar-event,
  .cx-calendar-range,
  .cx-calendar-more {
    display: block;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .cx-calendar-event {
    border-inline-start: 3px solid currentColor;
    padding-inline-start: 0.25rem;
    margin-top: 0.1rem;
  }
  .cx-calendar-range {
    font-style: italic;
  }
  .cx-calendar-more {
    color: var(--muted);
  }
</style>

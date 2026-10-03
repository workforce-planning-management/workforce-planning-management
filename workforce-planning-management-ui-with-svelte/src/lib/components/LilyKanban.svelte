<!--
  A keyboard-accessible kanban board: `@lilydesignsystem/svelte-kanban-board`
  with this app's labels and minimal styling (the package ships no CSS).
  Cards move by pointer drag *or* a per-card "Move to…" menu (WCAG 2.5.7);
  the caller decides whether a move is legal and reloads `cards` — an
  illegal move is simply put back by the next render of the truth.
-->
<script lang="ts">
  import KanbanBoard from "@lilydesignsystem/svelte-kanban-board";
  import type { KanbanCard, KanbanColumn } from "@lilydesignsystem/svelte-kanban-board";

  let {
    label,
    columns,
    cards,
    onMove,
  }: {
    label: string;
    columns: KanbanColumn[];
    cards: KanbanCard[];
    onMove: (cardId: string, toColumnId: string) => void;
  } = $props();
</script>

<div class="lily-kanban">
  <KanbanBoard
    {label}
    {columns}
    {cards}
    {onMove}
    labels={{
      cardCount: (count) => `${count} card${count === 1 ? "" : "s"}`,
      overLimit: (count, limit) => `Over limit: ${count} of ${limit}`,
      moveButton: (card) => `Move ${card.title}`,
      moveMenuLabel: "Move to column",
      moveAnnouncement: (title, column) => `${title} moved to ${column}`,
    }}
  />
</div>

<style>
  .lily-kanban {
    overflow-x: auto;
    margin-block: 0.5rem 1rem;
  }
  .lily-kanban :global(.kanban-board) {
    border-collapse: separate;
    border-spacing: 0.5rem;
    width: 100%;
  }
  .lily-kanban :global([role="columnheader"]) {
    text-align: start;
    font-weight: 600;
    padding: 0.25rem 0.5rem;
  }
  .lily-kanban :global([role="gridcell"]) {
    vertical-align: top;
    min-width: 9rem;
    padding: 0.4rem 0.5rem;
    border: 1px solid var(--line, #d6d6d6);
    border-radius: 0.4rem;
    background: var(--panel, transparent);
  }
  .lily-kanban :global([role="gridcell"]:focus-visible) {
    outline: 2px solid currentColor;
    outline-offset: 2px;
  }
  .lily-kanban :global([data-over-limit]) {
    border-color: #b45309;
  }
  .lily-kanban :global(.kanban-board-wip-warning),
  .lily-kanban :global(.kanban-board-count) {
    font-size: 0.8em;
    opacity: 0.8;
  }
</style>

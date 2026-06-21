<!--
  EventLog Component

  Displays the event log during VDF computation, including:
  - Checkpoint save events
  - Lottery winner events
  - Error events
  - Informational events

  Features:
  - Type filtering (All / Info / Checkpoint / Winner / Error)
  - Relative time display ("2m ago") alongside timestamps
  - Auto-scrolls to the latest event
  - Animated entry effects
  - Maximum of 50 events displayed
-->

<script lang="ts">
  import { events } from '$stores/worker';
  import { formatRelativeTime } from '$utils';

  /** Active type filters (all selected by default) */
  let activeFilters: Set<string> = new Set(['info', 'checkpoint', 'winner', 'error']);

  /** Filter toggle options */
  const filterOptions = [
    { type: 'info', label: 'Info', color: '#6366f1' },
    { type: 'checkpoint', label: 'Checkpoint', color: '#3b82f6' },
    { type: 'winner', label: 'Winner', color: '#10b981' },
    { type: 'error', label: 'Error', color: '#ef4444' }
  ];

  /** Whether all filters are currently active */
  $: allActive = activeFilters.size === 4;

  /** Filtered event list */
  $: filteredEvents = allActive ? $events : $events.filter((e) => activeFilters.has(e.type));

  function toggleFilter(type: string) {
    if (activeFilters.has(type)) {
      if (activeFilters.size > 1) {
        activeFilters.delete(type);
        activeFilters = activeFilters; // trigger reactivity
      }
    } else {
      activeFilters.add(type);
      activeFilters = activeFilters;
    }
  }

  function showAll() {
    activeFilters = new Set(['info', 'checkpoint', 'winner', 'error']);
  }

  function formatTimestamp(timestamp: number): string {
    return new Date(timestamp).toLocaleTimeString();
  }

  function getEventIcon(type: string): string {
    switch (type) {
      case 'checkpoint':
        return '💾';
      case 'winner':
        return '🎉';
      case 'error':
        return '❌';
      case 'info':
        return 'ℹ️';
      default:
        return '📋';
    }
  }

  function getEventTypeClass(type: string): string {
    return `event-${type}`;
  }
</script>

<div class="event-log">
  <!-- Header with count -->
  <div class="event-header">
    <h3 class="event-title">Recent Events</h3>
    <span class="event-count">{$events.length}</span>
  </div>

  <!-- Type filters -->
  <div class="filter-bar">
    <button class="filter-pill" class:active={allActive} on:click={showAll}> All </button>
    {#each filterOptions as opt}
      <button
        class="filter-pill"
        class:active={activeFilters.has(opt.type)}
        style="--pill-color: {opt.color}"
        on:click={() => toggleFilter(opt.type)}
      >
        {opt.label}
      </button>
    {/each}
  </div>

  <!-- Event list -->
  <div class="events-list">
    {#each filteredEvents as event, index}
      <div
        class="event-item {getEventTypeClass(event.type)}"
        style="animation-delay: {Math.min(index * 40, 200)}ms"
      >
        <span class="event-icon">{getEventIcon(event.type)}</span>
        <div class="event-content">
          <span class="event-message">{event.message}</span>
          <div class="event-meta">
            <span class="event-time">{formatTimestamp(event.timestamp)}</span>
            <span class="event-relative">{formatRelativeTime(event.timestamp)}</span>
          </div>
        </div>
      </div>
    {:else}
      <div class="empty-state">
        <div class="empty-icon">📭</div>
        <span class="empty-title">No events recorded</span>
        <span class="empty-hint">Events will appear here when computation starts</span>
      </div>
    {/each}
  </div>
</div>

<style>
  .event-log {
    display: flex;
    flex-direction: column;
    gap: var(--space-md);
    height: 100%;
  }

  .event-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .event-title {
    margin: 0;
    font-size: 0.75rem;
    color: var(--color-text-muted);
    text-transform: uppercase;
    letter-spacing: 0.1em;
  }

  .event-count {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 24px;
    height: 24px;
    padding: 0 6px;
    font-size: 0.6875rem;
    font-weight: 600;
    background: rgba(0, 255, 136, 0.1);
    color: var(--color-accent-green);
    border-radius: var(--radius-full);
    font-family: var(--font-mono);
  }

  /* ── Filter bar ── */
  .filter-bar {
    display: flex;
    gap: 0.375rem;
    flex-wrap: wrap;
  }

  .filter-pill {
    padding: 0.25rem 0.625rem;
    font-size: 0.6875rem;
    font-weight: 500;
    font-family: var(--font-sans);
    color: var(--color-text-muted);
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-full);
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .filter-pill:hover {
    background: var(--color-surface-hover);
    color: var(--color-text-secondary);
  }

  .filter-pill.active {
    color: var(--pill-color, var(--color-accent-green));
    border-color: var(--pill-color, var(--color-accent-green));
    background: color-mix(in srgb, var(--pill-color, var(--color-accent-green)) 10%, transparent);
  }

  /* ── Event list ── */
  .events-list {
    display: flex;
    flex-direction: column;
    gap: 0.375rem;
    flex: 1;
    overflow-y: auto;
    padding-right: 0.375rem;
  }

  .event-item {
    display: flex;
    align-items: flex-start;
    gap: 0.625rem;
    padding: 0.75rem;
    background: rgba(255, 255, 255, 0.02);
    border-radius: var(--radius-sm);
    font-size: 0.8125rem;
    border: 1px solid var(--color-border);
    animation: slideInEvent 0.3s ease forwards;
    opacity: 0;
    transform: translateX(-8px);
    transition: all 0.2s ease;
  }

  @keyframes slideInEvent {
    to {
      opacity: 1;
      transform: translateX(0);
    }
  }

  .event-item:hover {
    background: var(--color-surface-hover);
    border-color: rgba(255, 255, 255, 0.08);
  }

  /* Event type borders */
  .event-checkpoint {
    border-left: 3px solid #3b82f6;
  }
  .event-winner {
    border-left: 3px solid #10b981;
    background: rgba(16, 185, 129, 0.04);
  }
  .event-error {
    border-left: 3px solid #ef4444;
    background: rgba(239, 68, 68, 0.04);
  }
  .event-info {
    border-left: 3px solid #6366f1;
  }

  .event-icon {
    font-size: 1rem;
    flex-shrink: 0;
    line-height: 1.3;
  }

  .event-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    min-width: 0;
  }

  .event-message {
    color: var(--color-text-primary);
    line-height: 1.4;
    word-break: break-word;
  }

  .event-meta {
    display: flex;
    gap: var(--space-sm);
    align-items: center;
  }

  .event-time {
    font-size: 0.6875rem;
    color: var(--color-text-muted);
    font-family: var(--font-mono);
  }

  .event-relative {
    font-size: 0.625rem;
    color: rgba(255, 255, 255, 0.15);
    font-family: var(--font-mono);
  }

  /* ── Empty state ── */
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    text-align: center;
    color: var(--color-text-muted);
    padding: 2.5rem 1.5rem;
  }

  .empty-icon {
    font-size: 1.75rem;
    opacity: 0.4;
  }

  .empty-title {
    font-size: 0.875rem;
    color: var(--color-text-secondary);
  }

  .empty-hint {
    font-size: 0.75rem;
    color: var(--color-text-muted);
    max-width: 200px;
  }

  /* ── Scrollbar ── */
  .events-list::-webkit-scrollbar {
    width: 4px;
  }

  .events-list::-webkit-scrollbar-track {
    background: transparent;
  }

  .events-list::-webkit-scrollbar-thumb {
    background: rgba(255, 255, 255, 0.08);
    border-radius: 2px;
  }

  .events-list::-webkit-scrollbar-thumb:hover {
    background: rgba(255, 255, 255, 0.15);
  }
</style>

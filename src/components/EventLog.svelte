<!--
  EventLog Component

  Displays the event log during VDF computation, including:
  - Checkpoint save events
  - Lottery winner events
  - Error events
  - Informational events

  Features:
  - Auto-scrolls to the latest event
  - Displays events in reverse chronological order
  - Uses distinct icons for different event types
  - Displays a maximum of 50 events
  - Animated entry effects

  Data source:
  Subscribes to the events Svelte Store to obtain the event list

  Usage example:
  ```svelte
  <EventLog />
  ```
-->

<script lang="ts">
  import { events } from '$stores/worker';

  /**
   * Format a timestamp
   *
   * Converts a Unix timestamp to a localised time string.
   *
   * @param timestamp - Unix timestamp in milliseconds
   * @returns Formatted time string
   *
   * @example
   * formatTimestamp(1234567890000) // "12:34:50"
   */
  function formatTimestamp(timestamp: number): string {
    return new Date(timestamp).toLocaleTimeString();
  }

  /**
   * Get the event icon
   *
   * Returns the corresponding emoji icon based on the event type.
   *
   * @param type - Event type
   * @returns Corresponding emoji icon
   *
   * @example
   * getEventIcon('winner') // "🎉"
   * getEventIcon('error') // "❌"
   */
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

  /**
   * Get event type class for styling
   *
   * Returns CSS class based on event type for color coding
   *
   * @param type - Event type
   * @returns CSS class name
   */
  function getEventTypeClass(type: string): string {
    switch (type) {
      case 'checkpoint':
        return 'event-checkpoint';
      case 'winner':
        return 'event-winner';
      case 'error':
        return 'event-error';
      case 'info':
        return 'event-info';
      default:
        return '';
    }
  }
</script>

<!-- Event log container -->
<div class="event-log">
  <!-- Heading -->
  <div class="event-header">
    <h3>Recent Events</h3>
    <span class="event-count">{$events.length}</span>
  </div>

  <!-- Event list -->
  <div class="events-list">
    {#each $events as event, index}
      <!-- Individual event item -->
      <div
        class="event-item {getEventTypeClass(event.type)}"
        style="animation-delay: {Math.min(index * 50, 300)}ms"
      >
        <!-- Event icon -->
        <span class="event-icon">{getEventIcon(event.type)}</span>

        <!-- Event content -->
        <div class="event-content">
          <!-- Event message -->
          <span class="event-message">{event.message}</span>

          <!-- Event timestamp -->
          <span class="event-time">{formatTimestamp(event.timestamp)}</span>
        </div>
      </div>
    {:else}
      <!-- Empty state placeholder -->
      <div class="empty-state">
        <div class="empty-icon">📭</div>
        <span>No events recorded</span>
      </div>
    {/each}
  </div>
</div>

<style>
  /* Event log container */
  .event-log {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    height: 100%;
  }

  /* Event header */
  .event-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  /* Heading styles */
  h3 {
    margin: 0;
    font-size: 0.875rem;
    color: #666;
    text-transform: uppercase;
    letter-spacing: 0.1em;
  }

  /* Event count badge */
  .event-count {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 24px;
    height: 24px;
    padding: 0 6px;
    font-size: 0.75rem;
    font-weight: 600;
    background: rgba(0, 255, 136, 0.1);
    color: #00ff88;
    border-radius: 12px;
  }

  /* Event list container */
  .events-list {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    flex: 1;
    max-height: 300px;
    overflow-y: auto;
    padding-right: 0.5rem;
  }

  /* Individual event item */
  .event-item {
    display: flex;
    align-items: flex-start;
    gap: 0.75rem;
    padding: 0.875rem;
    background: rgba(255, 255, 255, 0.02);
    border-radius: 10px;
    font-size: 0.875rem;
    border: 1px solid rgba(255, 255, 255, 0.05);
    animation: slideInEvent 0.3s ease forwards;
    opacity: 0;
    transform: translateX(-10px);
    transition: all 0.2s ease;
  }

  @keyframes slideInEvent {
    to {
      opacity: 1;
      transform: translateX(0);
    }
  }

  .event-item:hover {
    background: rgba(255, 255, 255, 0.04);
    border-color: rgba(255, 255, 255, 0.1);
  }

  /* Event type borders */
  .event-checkpoint {
    border-left: 3px solid #3b82f6;
  }

  .event-winner {
    border-left: 3px solid #10b981;
    background: rgba(16, 185, 129, 0.05);
  }

  .event-error {
    border-left: 3px solid #ef4444;
    background: rgba(239, 68, 68, 0.05);
  }

  .event-info {
    border-left: 3px solid #6366f1;
  }

  /* Event icon styles */
  .event-icon {
    font-size: 1.25rem;
    flex-shrink: 0;
  }

  /* Event content */
  .event-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    min-width: 0;
  }

  /* Event message styles */
  .event-message {
    color: #e6e6e6;
    line-height: 1.4;
    word-break: break-word;
  }

  /* Event timestamp styles */
  .event-time {
    font-size: 0.75rem;
    color: #555;
    font-family: 'Courier New', monospace;
  }

  /* Empty state placeholder styles */
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 0.75rem;
    text-align: center;
    color: #555;
    padding: 3rem 2rem;
  }

  .empty-icon {
    font-size: 2rem;
    opacity: 0.5;
  }

  /* Custom scrollbar */
  .events-list::-webkit-scrollbar {
    width: 4px;
  }

  .events-list::-webkit-scrollbar-track {
    background: transparent;
  }

  .events-list::-webkit-scrollbar-thumb {
    background: rgba(255, 255, 255, 0.1);
    border-radius: 2px;
  }

  .events-list::-webkit-scrollbar-thumb:hover {
    background: rgba(255, 255, 255, 0.2);
  }
</style>

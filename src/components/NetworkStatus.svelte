<!--
  NetworkStatus Component

  Displays the current state of the P2P networking layer:
  - Signaling server connection status
  - Number of connected peers
  - IPv6/IPv4 connection breakdown
  - Per-peer details (connection state, RTT, IPv6 status)
  - Recent network events
-->

<script lang="ts">
  import {
    networkState,
    networkEvents,
    signalingConnected,
    connectionQuality,
    type NetworkEvent
  } from '$stores/network';
  import { formatRelativeTime } from '$utils';

  /** Whether the peer details section is expanded. */
  let expanded = false;

  /** Format RTT for display. */
  function formatRtt(rtt: number): string {
    if (rtt < 0) return '—';
    if (rtt < 1) return '<1ms';
    return `${rtt}ms`;
  }

  /** Get the CSS class for a connection quality level. */
  function qualityClass(quality: string): string {
    switch (quality) {
      case 'good':
        return 'quality-good';
      case 'degraded':
        return 'quality-degraded';
      default:
        return 'quality-disconnected';
    }
  }

  /** Get the CSS class for a peer state. */
  function peerStateClass(state: string): string {
    switch (state) {
      case 'connected':
        return 'peer-connected';
      case 'connecting':
        return 'peer-connecting';
      case 'disconnected':
        return 'peer-disconnected';
      case 'failed':
        return 'peer-failed';
      default:
        return 'peer-new';
    }
  }

  /** Format a network event for display. */
  function formatEvent(event: NetworkEvent): string {
    const prefix = event.peerId ? `[${event.peerId.slice(0, 6)}] ` : '';
    return `${prefix}${event.message}`;
  }
</script>

<div class="network-status">
  <div class="status-header">
    <span class="status-title">Network</span>
    <span class="quality-badge {qualityClass($connectionQuality)}">
      {$connectionQuality}
    </span>
  </div>

  <div class="status-list">
    <!-- Signaling -->
    <div class="status-row">
      <span
        class="indicator"
        class:connected={$signalingConnected}
        class:disconnected={!$signalingConnected}
      ></span>
      <span class="status-label">Signaling</span>
      <span class="status-value" class:connected={$signalingConnected}>
        {$networkState.signalingState}
      </span>
    </div>

    <!-- Peer Count -->
    <div class="status-row">
      <span
        class="indicator"
        class:connected={$networkState.peerCount > 0}
        class:idle={$networkState.peerCount === 0}
      ></span>
      <span class="status-label">Peers</span>
      <span class="status-value">
        {$networkState.peerCount}
        {#if $networkState.ipv6PeerCount > 0}
          <span class="ipv6-badge">{$networkState.ipv6PeerCount} IPv6</span>
        {/if}
      </span>
    </div>

    <!-- RTT -->
    {#if $networkState.averageRtt > 0}
      <div class="status-row">
        <span class="indicator rtt"></span>
        <span class="status-label">Avg RTT</span>
        <span class="status-value">{formatRtt($networkState.averageRtt)}</span>
      </div>
    {/if}

    <!-- Local Peer ID -->
    {#if $networkState.localPeerId}
      <div class="status-row">
        <span class="indicator self"></span>
        <span class="status-label">Self</span>
        <span class="status-value mono">{$networkState.localPeerId}</span>
      </div>
    {/if}
  </div>

  <!-- Peer Details (expandable) -->
  {#if $networkState.peerCount > 0}
    <button class="expand-toggle" on:click={() => (expanded = !expanded)}>
      {expanded ? '▲ Hide Peers' : '▼ Show Peers'}
    </button>

    {#if expanded}
      <div class="peer-details">
        {#each [...$networkState.peers.entries()] as [peerId, peer]}
          <div class="peer-row">
            <span class="indicator {peerStateClass(peer.state)}"></span>
            <span class="peer-id mono">{peerId}</span>
            <span class="peer-meta">
              {#if peer.isIpv6}
                <span class="tag ipv6">IPv6</span>
              {:else}
                <span class="tag ipv4">IPv4</span>
              {/if}
              <span class="tag rtt">{formatRtt(peer.rtt)}</span>
            </span>
          </div>
        {/each}
      </div>
    {/if}
  {/if}

  <!-- Recent Events -->
  {#if $networkEvents.length > 0}
    <div class="events-section">
      <span class="events-title">Recent Events</span>
      <div class="events-list">
        {#each $networkEvents.slice(0, 5) as event}
          <div class="event-row">
            <span class="event-type {event.type}">{event.type}</span>
            <span class="event-message">{formatEvent(event)}</span>
            <span class="event-time">{formatRelativeTime(event.timestamp)}</span>
          </div>
        {/each}
      </div>
    </div>
  {/if}
</div>

<style>
  .network-status {
    display: flex;
    flex-direction: column;
    gap: var(--space-md);
  }

  .status-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .status-title {
    font-size: 0.75rem;
    color: var(--color-text-muted);
    text-transform: uppercase;
    letter-spacing: 0.1em;
  }

  .quality-badge {
    font-size: 0.625rem;
    font-family: var(--font-mono);
    padding: 0.125rem 0.5rem;
    border-radius: var(--radius-sm);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .quality-badge.quality-good {
    background: rgba(34, 197, 94, 0.15);
    color: var(--color-accent-green);
    border: 1px solid rgba(34, 197, 94, 0.3);
  }

  .quality-badge.quality-degraded {
    background: rgba(245, 158, 11, 0.15);
    color: var(--color-accent-amber);
    border: 1px solid rgba(245, 158, 11, 0.3);
  }

  .quality-badge.quality-disconnected {
    background: rgba(239, 68, 68, 0.15);
    color: var(--color-accent-red);
    border: 1px solid rgba(239, 68, 68, 0.3);
  }

  .status-list {
    display: flex;
    flex-direction: column;
    gap: 0.625rem;
  }

  .status-row {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    padding: 0.5rem 0.75rem;
    background: rgba(255, 255, 255, 0.02);
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border);
    transition: all 0.2s ease;
  }

  .status-row:hover {
    background: var(--color-surface-hover);
  }

  .indicator {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    flex-shrink: 0;
    transition: all 0.3s ease;
  }

  .indicator.connected {
    background: var(--color-accent-green);
    box-shadow: 0 0 6px var(--color-accent-green-glow);
  }

  .indicator.disconnected {
    background: var(--color-accent-red);
    box-shadow: 0 0 6px rgba(239, 68, 68, 0.4);
  }

  .indicator.idle {
    background: var(--color-text-muted);
  }

  .indicator.rtt {
    background: var(--color-accent-indigo);
  }

  .indicator.self {
    background: var(--color-accent-amber);
  }

  .indicator.peer-connected {
    background: var(--color-accent-green);
    box-shadow: 0 0 6px var(--color-accent-green-glow);
  }

  .indicator.peer-connecting {
    background: var(--color-accent-amber);
    animation: pulse 1s ease-in-out infinite;
  }

  .indicator.peer-disconnected {
    background: var(--color-accent-red);
  }

  .indicator.peer-failed {
    background: var(--color-accent-red);
    opacity: 0.6;
  }

  .indicator.peer-new {
    background: var(--color-text-muted);
  }

  @keyframes pulse {
    0%,
    100% {
      opacity: 1;
    }
    50% {
      opacity: 0.4;
    }
  }

  .status-label {
    font-size: 0.8125rem;
    color: var(--color-text-secondary);
    flex: 1;
  }

  .status-value {
    font-size: 0.75rem;
    font-family: var(--font-mono);
    color: var(--color-text-primary);
    font-weight: 500;
    display: flex;
    align-items: center;
    gap: 0.375rem;
  }

  .status-value.connected {
    color: var(--color-accent-green);
  }

  .status-value.mono {
    font-size: 0.6875rem;
  }

  .ipv6-badge {
    font-size: 0.5625rem;
    background: rgba(99, 102, 241, 0.15);
    color: var(--color-accent-indigo);
    padding: 0.0625rem 0.375rem;
    border-radius: 0.25rem;
    border: 1px solid rgba(99, 102, 241, 0.3);
  }

  .expand-toggle {
    font-size: 0.6875rem;
    color: var(--color-text-muted);
    background: none;
    border: none;
    cursor: pointer;
    padding: 0.25rem 0;
    text-align: center;
    transition: color 0.2s ease;
  }

  .expand-toggle:hover {
    color: var(--color-text-secondary);
  }

  .peer-details {
    display: flex;
    flex-direction: column;
    gap: 0.375rem;
    padding-left: 0.5rem;
  }

  .peer-row {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    padding: 0.375rem 0.5rem;
    background: rgba(255, 255, 255, 0.01);
    border-radius: var(--radius-sm);
    border: 1px solid rgba(255, 255, 255, 0.04);
  }

  .peer-id {
    font-size: 0.6875rem;
    color: var(--color-text-secondary);
    flex: 1;
  }

  .peer-meta {
    display: flex;
    gap: 0.25rem;
  }

  .tag {
    font-size: 0.5625rem;
    font-family: var(--font-mono);
    padding: 0.0625rem 0.375rem;
    border-radius: 0.25rem;
  }

  .tag.ipv6 {
    background: rgba(99, 102, 241, 0.15);
    color: var(--color-accent-indigo);
    border: 1px solid rgba(99, 102, 241, 0.3);
  }

  .tag.ipv4 {
    background: rgba(156, 163, 175, 0.15);
    color: var(--color-text-muted);
    border: 1px solid rgba(156, 163, 175, 0.3);
  }

  .tag.rtt {
    background: rgba(34, 197, 94, 0.1);
    color: var(--color-accent-green);
    border: 1px solid rgba(34, 197, 94, 0.2);
  }

  .events-section {
    border-top: 1px solid var(--color-border);
    padding-top: var(--space-sm);
  }

  .events-title {
    font-size: 0.6875rem;
    color: var(--color-text-muted);
    text-transform: uppercase;
    letter-spacing: 0.08em;
    display: block;
    margin-bottom: 0.5rem;
  }

  .events-list {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .event-row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.25rem 0.5rem;
    font-size: 0.6875rem;
  }

  .event-type {
    font-family: var(--font-mono);
    font-size: 0.5625rem;
    padding: 0.0625rem 0.25rem;
    border-radius: 0.25rem;
    text-transform: uppercase;
    flex-shrink: 0;
  }

  .event-type.peer-joined {
    background: rgba(34, 197, 94, 0.15);
    color: var(--color-accent-green);
  }

  .event-type.peer-left {
    background: rgba(239, 68, 68, 0.15);
    color: var(--color-accent-red);
  }

  .event-type.message {
    background: rgba(99, 102, 241, 0.15);
    color: var(--color-accent-indigo);
  }

  .event-type.signaling {
    background: rgba(245, 158, 11, 0.15);
    color: var(--color-accent-amber);
  }

  .event-type.error {
    background: rgba(239, 68, 68, 0.15);
    color: var(--color-accent-red);
  }

  .event-message {
    color: var(--color-text-secondary);
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .event-time {
    color: var(--color-text-muted);
    font-size: 0.5625rem;
    flex-shrink: 0;
  }
</style>

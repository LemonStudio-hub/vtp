<!--
  SystemHealth Component

  Displays the state of the background keep-alive subsystems:
  - Worker health status
  - Tab visibility state
  - Audio keep-alive status
  - Heartbeat monitoring
  - Wake state indicator
-->

<script lang="ts">
  import {
    isVisible,
    workerHealthy,
    missedHeartbeats,
    audioKeepAliveActive,
    isWaking
  } from '$stores/visibility';
</script>

<div class="system-health">
  <div class="health-header">
    <span class="health-title">System Health</span>
  </div>

  <div class="health-list">
    <!-- Worker status -->
    <div class="health-row">
      <span class="indicator" class:healthy={$workerHealthy} class:unhealthy={!$workerHealthy}
      ></span>
      <span class="health-label">Worker</span>
      <span class="health-value" class:healthy={$workerHealthy} class:unhealthy={!$workerHealthy}>
        {$workerHealthy ? 'Healthy' : 'Unhealthy'}
      </span>
    </div>

    <!-- Visibility -->
    <div class="health-row">
      <span class="indicator" class:healthy={$isVisible} class:hidden={!$isVisible}></span>
      <span class="health-label">Visibility</span>
      <span class="health-value">{$isVisible ? 'Visible' : 'Hidden'}</span>
    </div>

    <!-- Audio Keep-Alive -->
    <div class="health-row">
      <span
        class="indicator"
        class:healthy={$audioKeepAliveActive}
        class:inactive={!$audioKeepAliveActive}
      ></span>
      <span class="health-label">Audio Keep-Alive</span>
      <span class="health-value" class:muted={!$audioKeepAliveActive}>
        {$audioKeepAliveActive ? 'Active' : 'Inactive'}
      </span>
    </div>

    <!-- Heartbeats -->
    <div class="health-row">
      <span
        class="indicator"
        class:healthy={$missedHeartbeats === 0}
        class:unhealthy={$missedHeartbeats > 0}
      ></span>
      <span class="health-label">Heartbeats</span>
      <span class="health-value" class:warning={$missedHeartbeats > 0}>
        {$missedHeartbeats > 0 ? `${$missedHeartbeats} missed` : 'OK'}
      </span>
    </div>

    <!-- Wake State -->
    {#if $isWaking}
      <div class="health-row waking">
        <span class="indicator waking-pulse"></span>
        <span class="health-label">Wake</span>
        <span class="health-value waking-text">Recovering…</span>
      </div>
    {/if}
  </div>
</div>

<style>
  .system-health {
    display: flex;
    flex-direction: column;
    gap: var(--space-md);
  }

  .health-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .health-title {
    font-size: 0.75rem;
    color: var(--color-text-muted);
    text-transform: uppercase;
    letter-spacing: 0.1em;
  }

  .health-list {
    display: flex;
    flex-direction: column;
    gap: 0.625rem;
  }

  .health-row {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    padding: 0.5rem 0.75rem;
    background: rgba(255, 255, 255, 0.02);
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border);
    transition: all 0.2s ease;
  }

  .health-row:hover {
    background: var(--color-surface-hover);
  }

  .indicator {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    flex-shrink: 0;
    transition: all 0.3s ease;
  }

  .indicator.healthy {
    background: var(--color-accent-green);
    box-shadow: 0 0 6px var(--color-accent-green-glow);
  }

  .indicator.unhealthy {
    background: var(--color-accent-red);
    box-shadow: 0 0 6px rgba(239, 68, 68, 0.4);
  }

  .indicator.hidden {
    background: var(--color-accent-amber);
  }

  .indicator.inactive {
    background: var(--color-text-muted);
  }

  .indicator.waking-pulse {
    background: var(--color-accent-indigo);
    animation: pulse 1s ease-in-out infinite;
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

  .health-label {
    font-size: 0.8125rem;
    color: var(--color-text-secondary);
    flex: 1;
  }

  .health-value {
    font-size: 0.75rem;
    font-family: var(--font-mono);
    color: var(--color-text-primary);
    font-weight: 500;
  }

  .health-value.healthy {
    color: var(--color-accent-green);
  }

  .health-value.unhealthy {
    color: var(--color-accent-red);
  }

  .health-value.muted {
    color: var(--color-text-muted);
  }

  .health-value.warning {
    color: var(--color-accent-amber);
  }

  .health-value.waking-text {
    color: var(--color-accent-indigo);
  }

  .health-row.waking {
    border-color: rgba(99, 102, 241, 0.2);
    background: rgba(99, 102, 241, 0.05);
  }
</style>

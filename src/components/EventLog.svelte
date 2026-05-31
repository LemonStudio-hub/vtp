<!--
  EventLog 组件

  显示 VDF 计算过程中的事件日志，包括：
  - 检查点保存事件
  - 中签事件
  - 错误事件
  - 信息事件

  功能特点：
  - 自动滚动到最新事件
  - 按时间倒序显示
  - 不同类型事件使用不同图标
  - 最多显示 50 条事件

  数据来源：
  通过 Svelte Store 订阅 events 获取事件列表

  使用示例：
  ```svelte
  <EventLog />
  ```
-->

<script lang="ts">
  import { events } from '$stores/worker';

  /**
   * 格式化时间戳
   *
   * 将 Unix 时间戳转换为本地时间字符串。
   *
   * @param timestamp - Unix 时间戳（毫秒）
   * @returns 格式化后的时间字符串
   *
   * @example
   * formatTimestamp(1234567890000) // "12:34:50"
   */
  function formatTimestamp(timestamp: number): string {
    return new Date(timestamp).toLocaleTimeString();
  }

  /**
   * 获取事件图标
   *
   * 根据事件类型返回对应的 emoji 图标。
   *
   * @param type - 事件类型
   * @returns 对应的 emoji 图标
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
</script>

<!-- 事件日志容器 -->
<div class="event-log">
  <!-- 标题 -->
  <h3>最近事件</h3>

  <!-- 事件列表 -->
  <div class="events-list">
    {#each $events as event}
      <!-- 单个事件项 -->
      <div class="event-item">
        <!-- 事件图标 -->
        <span class="event-icon">{getEventIcon(event.type)}</span>

        <!-- 事件时间 -->
        <span class="event-time">{formatTimestamp(event.timestamp)}</span>

        <!-- 事件消息 -->
        <span class="event-message">{event.message}</span>
      </div>
    {:else}
      <!-- 空状态提示 -->
      <div class="empty-state">暂无事件记录</div>
    {/each}
  </div>
</div>

<style>
  /* 事件日志容器 */
  .event-log {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  /* 标题样式 */
  h3 {
    margin: 0;
    font-size: 1.125rem;
    color: #e6e6e6;
  }

  /* 事件列表容器 */
  .events-list {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    max-height: 200px;
    overflow-y: auto;
  }

  /* 单个事件项 */
  .event-item {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.75rem;
    background: #1a1a2e;
    border-radius: 8px;
    font-size: 0.875rem;
  }

  /* 事件图标样式 */
  .event-icon {
    font-size: 1.25rem;
  }

  /* 事件时间样式 */
  .event-time {
    color: #888;
    font-family: 'Courier New', monospace;
  }

  /* 事件消息样式 */
  .event-message {
    flex: 1;
    color: #e6e6e6;
  }

  /* 空状态提示样式 */
  .empty-state {
    text-align: center;
    color: #888;
    padding: 2rem;
  }
</style>

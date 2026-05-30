<!--
  StatsPanel 组件

  显示 VDF 计算的实时统计数据，包括：
  - 实时速度（步/秒）
  - 累计步数
  - 在线时长
  - 抽签次数
  - 运气指数

  数据来源：
  通过 Svelte Store 订阅 workerState 获取实时数据

  使用示例：
  ```svelte
  <StatsPanel />
  ```
-->

<script lang="ts">
  import { workerState } from '$stores/worker';

  /**
   * 格式化数字显示
   *
   * 将数字转换为本地化格式，添加千位分隔符。
   *
   * @param num - 要格式化的数字
   * @returns 格式化后的字符串
   *
   * @example
   * formatNumber(1234567) // "1,234,567"
   */
  function formatNumber(num: number): string {
    return num.toLocaleString();
  }

  /**
   * 格式化速度显示
   *
   * 将速度值转换为人类可读的格式，自动选择合适的单位。
   *
   * @param speed - 速度值（步/秒）
   * @returns 格式化后的字符串
   *
   * @example
   * formatSpeed(1500000) // "1.5M"
   * formatSpeed(1500) // "1.5K"
   * formatSpeed(500) // "500"
   */
  function formatSpeed(speed: number): string {
    if (speed >= 1000000) {
      return `${(speed / 1000000).toFixed(1)}M`;
    } else if (speed >= 1000) {
      return `${(speed / 1000).toFixed(1)}K`;
    }
    return speed.toFixed(0);
  }

  /**
   * 格式化时间显示
   *
   * 将秒数转换为 HH:MM:SS 格式。
   *
   * @param seconds - 秒数
   * @returns 格式化后的时间字符串
   *
   * @example
   * formatTime(3661) // "01:01:01"
   */
  function formatTime(seconds: number): string {
    const h = Math.floor(seconds / 3600);
    const m = Math.floor((seconds % 3600) / 60);
    const s = Math.floor(seconds % 60);
    return `${h.toString().padStart(2, '0')}:${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`;
  }
</script>

<!-- 统计面板网格布局 -->
<div class="stats-grid">
  <!-- 实时速度 -->
  <div class="stat-item">
    <label>实时速度</label>
    <span class="value">{formatSpeed($workerState.speed)} 步/秒</span>
  </div>

  <!-- 累计步数 -->
  <div class="stat-item">
    <label>累计步数</label>
    <span class="value">{formatNumber($workerState.currentStep)}</span>
  </div>

  <!-- 在线时长 -->
  <div class="stat-item">
    <label>在线时长</label>
    <span class="value">{formatTime($workerState.uptime)}</span>
  </div>

  <!-- 抽签次数 -->
  <div class="stat-item">
    <label>抽签次数</label>
    <span class="value">{formatNumber($workerState.winnerCount)}</span>
  </div>

  <!-- 运气指数 -->
  <div class="stat-item">
    <label>运气指数</label>
    <span class="value">
      {$workerState.luckPercentage.toFixed(0)}%
      <!-- 超过100%显示四叶草图标 -->
      {#if $workerState.luckPercentage > 100}
        🍀
      {/if}
    </span>
  </div>
</div>

<style>
  /* 统计网格布局 */
  .stats-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 1.5rem;
  }

  /* 统计项容器 */
  .stat-item {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  /* 标签样式 */
  label {
    font-size: 0.875rem;
    color: #888;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  /* 数值样式 */
  .value {
    font-size: 1.5rem;
    font-weight: bold;
    color: #00ff88;
    font-family: 'Courier New', monospace;
  }
</style>

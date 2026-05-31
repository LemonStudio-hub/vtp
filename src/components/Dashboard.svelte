<!--
  Dashboard 组件

  VTP 节点的主仪表盘界面，负责：
  1. 展示 VDF 计算进度
  2. 显示实时统计数据
  3. 提供控制按钮（开始/暂停/恢复）
  4. 显示事件日志

  组件结构：
  - IdentityBadge: 节点身份标识
  - VDFCanvas: VDF 进度可视化
  - StatsPanel: 实时统计面板
  - EventLog: 事件日志
  - 控制按钮区域

  使用示例：
  ```svelte
  <Dashboard />
  ```
-->

<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { workerStore, workerState } from '$stores/worker';
  import StatsPanel from './StatsPanel.svelte';
  import EventLog from './EventLog.svelte';
  import IdentityBadge from './IdentityBadge.svelte';
  import VDFCanvas from './VDFCanvas.svelte';

  /** Web Worker 实例引用 */
  let worker: Worker | null = null;

  /** Store 订阅取消函数 */
  let unsubscribe: () => void;

  /**
   * 组件挂载时订阅 Worker Store
   *
   * 获取 Worker 实例引用，用于发送控制命令
   */
  onMount(() => {
    unsubscribe = workerStore.subscribe((w) => {
      worker = w;
    });
  });

  /**
   * 组件销毁时取消订阅
   *
   * 防止内存泄漏
   */
  onDestroy(() => {
    if (unsubscribe) unsubscribe();
  });

  /**
   * 处理开始按钮点击
   *
   * 向 Worker 发送 start 命令，包含 VDF 配置参数：
   * - seed: 32 字节随机种子
   * - total: 总步数目标（100万步）
   * - k: VRF 抽签间隔（每1000步）
   * - tau: 32 字节阈值
   * - checkpointInterval: 检查点间隔（每10万步）
   */
  function handleStart() {
    if (worker) {
      worker.postMessage({
        type: 'start',
        seed: new Uint8Array(32),
        total: 1000000,
        k: 1000,
        tau: new Uint8Array(32),
        checkpointInterval: 100000
      });
    }
  }

  /**
   * 处理暂停按钮点击
   *
   * 向 Worker 发送 pause 命令
   */
  function handlePause() {
    if (worker) {
      worker.postMessage({ type: 'pause' });
    }
  }

  /**
   * 处理恢复按钮点击
   *
   * 向 Worker 发送 resume 命令
   */
  function handleResume() {
    if (worker) {
      worker.postMessage({ type: 'resume' });
    }
  }
</script>

<!-- 主仪表盘布局 -->
<div class="dashboard">
  <!-- 头部：身份标识和标题 -->
  <header>
    <IdentityBadge />
    <h1>VTP Node</h1>
  </header>

  <!-- VDF 进度可视化区域 -->
  <section class="vdf-section">
    <VDFCanvas />
  </section>

  <!-- 统计面板 -->
  <section class="stats-section">
    <StatsPanel />
  </section>

  <!-- 控制按钮区域 -->
  <section class="controls">
    <!-- 开始按钮：仅在未运行时可用 -->
    <button on:click={handleStart} disabled={$workerState.isRunning}> Start </button>

    <!-- 暂停按钮：仅在运行中且未暂停时可用 -->
    <button on:click={handlePause} disabled={!$workerState.isRunning || $workerState.isPaused}>
      Pause
    </button>

    <!-- 恢复按钮：仅在暂停状态时可用 -->
    <button on:click={handleResume} disabled={!$workerState.isPaused}> Resume </button>
  </section>

  <!-- 事件日志区域 -->
  <section class="events-section">
    <EventLog />
  </section>
</div>

<style>
  /* 仪表盘主容器 */
  .dashboard {
    display: grid;
    gap: 2rem;
  }

  /* 头部区域 */
  header {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding-bottom: 1rem;
    border-bottom: 1px solid #333;
  }

  /* 标题样式 */
  h1 {
    margin: 0;
    font-size: 2rem;
    color: #00ff88;
  }

  /* VDF 可视化区域 */
  .vdf-section {
    display: flex;
    justify-content: center;
  }

  /* 统计面板区域 */
  .stats-section {
    background: #16213e;
    border-radius: 12px;
    padding: 1.5rem;
  }

  /* 控制按钮容器 */
  .controls {
    display: flex;
    gap: 1rem;
    justify-content: center;
  }

  /* 通用按钮样式 */
  button {
    padding: 0.75rem 2rem;
    font-size: 1rem;
    border: none;
    border-radius: 8px;
    cursor: pointer;
    transition: all 0.2s;
  }

  /* 可用按钮样式 */
  button:not(:disabled) {
    background: #00ff88;
    color: #1a1a2e;
  }

  /* 按钮悬停效果 */
  button:not(:disabled):hover {
    background: #00cc6a;
  }

  /* 禁用按钮样式 */
  button:disabled {
    background: #333;
    color: #666;
    cursor: not-allowed;
  }

  /* 事件日志区域 */
  .events-section {
    background: #16213e;
    border-radius: 12px;
    padding: 1.5rem;
    max-height: 300px;
    overflow-y: auto;
  }
</style>

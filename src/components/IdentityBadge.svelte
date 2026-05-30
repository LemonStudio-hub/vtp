<!--
  IdentityBadge 组件

  显示节点的身份标识，包括：
  - 基于公钥生成的唯一图案
  - 节点 ID
  - 运行状态

  功能特点：
  - 根据公钥生成唯一的 Identicon 图案
  - 实时显示节点运行状态
  - 响应式更新

  算法说明：
  1. 对公钥进行简单哈希
  2. 使用哈希值生成 HSB 色彩
  3. 根据哈希位绘制 4x4 网格图案

  使用示例：
  ```svelte
  <IdentityBadge />
  ```
-->

<script lang="ts">
  import { workerState } from '$stores/worker';

  /** Canvas 元素引用 */
  let canvas: HTMLCanvasElement;

  /** Canvas 2D 渲染上下文 */
  let ctx: CanvasRenderingContext2D | null = null;

  /**
   * 响应式更新身份图案
   *
   * 当 canvas 元素和公钥都可用时，自动绘制身份图案。
   * 使用 Svelte 的响应式声明语法。
   */
  $: if (canvas && $workerState.publicKey) {
    drawIdentity();
  }

  /**
   * 绘制身份图案
   *
   * 根据公钥生成唯一的 Identicon 图案。
   *
   * 算法步骤：
   * 1. 对公钥进行简单哈希，得到数值
   * 2. 使用哈希值 % 360 作为基础色相
   * 3. 绘制背景色（基础色）
   * 4. 根据哈希的每一位决定是否绘制互补色方块
   * 5. 生成 4x4 的对称图案
   */
  function drawIdentity() {
    if (!canvas) return;

    ctx = canvas.getContext('2d');
    if (!ctx) return;

    // 设置 canvas 尺寸
    const size = 64;
    canvas.width = size;
    canvas.height = size;

    // 计算哈希值和基础色相
    const hash = simpleHash($workerState.publicKey);
    const hue = hash % 360;

    // 绘制背景（基础色）
    ctx.fillStyle = `hsl(${hue}, 70%, 50%)`;
    ctx.fillRect(0, 0, size, size);

    // 设置互补色
    ctx.fillStyle = `hsl(${(hue + 180) % 360}, 70%, 50%)`;

    // 绘制 4x4 网格图案
    // 根据哈希的每一位决定是否绘制方块
    for (let i = 0; i < 4; i++) {
      for (let j = 0; j < 4; j++) {
        // 检查哈希的对应位
        if ((hash >> (i * 4 + j)) & 1) {
          ctx.fillRect(i * 16, j * 16, 16, 16);
        }
      }
    }
  }

  /**
   * 简单哈希函数
   *
   * 对字节数组进行简单哈希，返回数值。
   * 用于生成身份图案的种子。
   *
   * @param data - 要哈希的字节数组，可以为 null
   * @returns 哈希值（正整数）
   *
   * 算法说明：
   * - 使用经典的 DJB2 哈希算法变体
   * - 只处理前 16 字节，避免过长输入
   * - 使用位运算确保结果为 32 位整数
   * - 返回绝对值确保非负
   */
  function simpleHash(data: Uint8Array | null): number {
    if (!data) return 0;

    let hash = 0;
    // 只处理前 16 字节
    for (let i = 0; i < Math.min(data.length, 16); i++) {
      // DJB2 哈希算法：hash = hash * 33 + byte
      hash = ((hash << 5) - hash + data[i]) | 0;
    }
    return Math.abs(hash);
  }
</script>

<!-- 身份标识容器 -->
<div class="identity-badge">
  <!-- 身份图案 Canvas -->
  <canvas bind:this={canvas} class="avatar"></canvas>

  <!-- 节点信息 -->
  <div class="node-info">
    <!-- 节点 ID -->
    <span class="node-id">Node {$workerState.nodeId || '---'}</span>

    <!-- 运行状态 -->
    <span class="status" class:running={$workerState.isRunning}>
      {$workerState.isRunning ? '运行中' : '待启动'}
    </span>
  </div>
</div>

<style>
  /* 身份标识容器 */
  .identity-badge {
    display: flex;
    align-items: center;
    gap: 1rem;
  }

  /* 身份图案样式 */
  .avatar {
    width: 64px;
    height: 64px;
    border-radius: 50%;
    border: 3px solid #333;
  }

  /* 节点信息容器 */
  .node-info {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  /* 节点 ID 样式 */
  .node-id {
    font-family: 'Courier New', monospace;
    font-size: 1rem;
    font-weight: bold;
    color: #e6e6e6;
  }

  /* 状态样式 */
  .status {
    font-size: 0.875rem;
    color: #888;
  }

  /* 运行状态样式 */
  .status.running {
    color: #00ff88;
  }
</style>

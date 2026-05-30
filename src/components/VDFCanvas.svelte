<!--
  VDFCanvas 组件

  可视化 VDF 计算进度的 Canvas 组件，包括：
  - 进度环：显示 VDF 计算完成百分比
  - 粒子效果：根据计算速度生成动态粒子
  - 百分比文字：居中显示完成百分比

  功能特点：
  - 实时响应计算状态
  - 粒子数量与计算速度成正比
  - 平滑的动画效果
  - 自动启动/停止动画

  性能优化：
  - 使用 requestAnimationFrame 实现平滑动画
  - 粒子数量限制，避免性能问题
  - 组件销毁时自动清理资源

  使用示例：
  ```svelte
  <VDFCanvas />
  ```
-->

<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { workerState } from '$stores/worker';

  /** Canvas 元素引用 */
  let canvas: HTMLCanvasElement;

  /** Canvas 2D 渲染上下文 */
  let ctx: CanvasRenderingContext2D | null = null;

  /** 动画帧 ID，用于取消动画 */
  let animationFrame: number;

  /** 粒子数组 */
  let particles: Particle[] = [];

  /**
   * 粒子接口定义
   *
   * 描述单个粒子的状态
   */
  interface Particle {
    /** X 坐标 */
    x: number;

    /** Y 坐标 */
    y: number;

    /** X 方向速度 */
    vx: number;

    /** Y 方向速度 */
    vy: number;

    /** 生命值，范围 [0, 1] */
    life: number;

    /** 最大生命值（帧数） */
    maxLife: number;

    /** 色相，范围 [0, 360] */
    hue: number;
  }

  /**
   * 响应式启动动画
   *
   * 当 canvas 可用且计算正在运行时，启动动画循环
   */
  $: if (canvas && $workerState.isRunning) {
    startAnimation();
  }

  /**
   * 响应式停止动画
   *
   * 当计算停止时，停止动画循环
   */
  $: if (!$workerState.isRunning && animationFrame) {
    stopAnimation();
  }

  /**
   * 组件挂载时初始化 Canvas
   *
   * 获取 2D 渲染上下文并设置 canvas 尺寸
   */
  onMount(() => {
    if (canvas) {
      ctx = canvas.getContext('2d');
      canvas.width = 400;
      canvas.height = 400;
    }
  });

  /**
   * 组件销毁时清理资源
   *
   * 取消动画帧，防止内存泄漏
   */
  onDestroy(() => {
    stopAnimation();
  });

  /**
   * 启动动画循环
   *
   * 取消现有动画帧并启动新的动画循环
   */
  function startAnimation() {
    if (animationFrame) cancelAnimationFrame(animationFrame);
    animate();
  }

  /**
   * 停止动画循环
   *
   * 取消当前动画帧
   */
  function stopAnimation() {
    if (animationFrame) {
      cancelAnimationFrame(animationFrame);
    }
  }

  /**
   * 动画主循环
   *
   * 每帧执行一次，负责：
   * 1. 清空画布（半透明效果，形成拖尾）
   * 2. 绘制进度环
   * 3. 更新和绘制粒子
   * 4. 生成新粒子
   * 5. 请求下一帧
   */
  function animate() {
    if (!ctx || !canvas) return;

    // 半透明背景，形成拖尾效果
    ctx.fillStyle = 'rgba(26, 26, 46, 0.1)';
    ctx.fillRect(0, 0, canvas.width, canvas.height);

    // 计算中心点和进度
    const centerX = canvas.width / 2;
    const centerY = canvas.height / 2;
    const progress = $workerState.currentStep / ($workerState.totalSteps || 1000000);

    // 绘制进度环
    drawProgressRing(ctx, centerX, centerY, progress);

    // 更新和绘制粒子
    updateParticles();
    drawParticles(ctx);

    // 根据速度生成新粒子
    if ($workerState.speed > 0) {
      spawnParticles(centerX, centerY, $workerState.speed);
    }

    // 请求下一帧
    animationFrame = requestAnimationFrame(animate);
  }

  /**
   * 绘制进度环
   *
   * 绘制两个同心圆弧：
   * - 灰色背景环：表示总进度
   * - 绿色前景环：表示当前进度
   * - 中心文字：显示百分比
   *
   * @param ctx - Canvas 2D 渲染上下文
   * @param x - 中心 X 坐标
   * @param y - 中心 Y 坐标
   * @param progress - 进度值，范围 [0, 1]
   */
  function drawProgressRing(
    ctx: CanvasRenderingContext2D,
    x: number,
    y: number,
    progress: number
  ) {
    const radius = 120;
    const lineWidth = 8;
    const startAngle = -Math.PI / 2; // 从顶部开始
    const endAngle = startAngle + (2 * Math.PI * progress);

    // 绘制背景环（灰色）
    ctx.beginPath();
    ctx.arc(x, y, radius, 0, 2 * Math.PI);
    ctx.strokeStyle = '#333';
    ctx.lineWidth = lineWidth;
    ctx.stroke();

    // 绘制前景环（绿色）
    ctx.beginPath();
    ctx.arc(x, y, radius, startAngle, endAngle);
    ctx.strokeStyle = '#00ff88';
    ctx.lineWidth = lineWidth;
    ctx.lineCap = 'round';
    ctx.stroke();

    // 绘制中心百分比文字
    ctx.fillStyle = '#e6e6e6';
    ctx.font = 'bold 24px Courier New';
    ctx.textAlign = 'center';
    ctx.textBaseline = 'middle';
    ctx.fillText(`${(progress * 100).toFixed(1)}%`, x, y);
  }

  /**
   * 生成新粒子
   *
   * 根据计算速度生成相应数量的粒子。
   * 粒子数量与速度成正比，最多 5 个。
   *
   * @param x - 生成中心 X 坐标
   * @param y - 生成中心 Y 坐标
   * @param speed - 计算速度（步/秒）
   */
  function spawnParticles(x: number, y: number, speed: number) {
    // 根据速度计算粒子数量，最多 5 个
    const count = Math.min(Math.floor(speed / 100000), 5);

    for (let i = 0; i < count; i++) {
      // 随机角度和速度
      const angle = Math.random() * Math.PI * 2;
      const speed = 0.5 + Math.random() * 2;

      particles.push({
        x,
        y,
        vx: Math.cos(angle) * speed,
        vy: Math.sin(angle) * speed,
        life: 1,
        maxLife: 60 + Math.random() * 60,
        hue: (Date.now() / 10) % 360
      });
    }
  }

  /**
   * 更新粒子状态
   *
   * 更新所有粒子的位置和生命值。
   * 移除生命值为 0 的粒子。
   */
  function updateParticles() {
    // 移除死亡粒子
    particles = particles.filter((p) => p.life > 0);

    // 更新粒子位置和生命值
    for (const p of particles) {
      p.x += p.vx;
      p.y += p.vy;
      p.life -= 1 / p.maxLife;
    }
  }

  /**
   * 绘制粒子
   *
   * 绘制所有存活的粒子。
   * 粒子大小和透明度与生命值成正比。
   *
   * @param ctx - Canvas 2D 渲染上下文
   */
  function drawParticles(ctx: CanvasRenderingContext2D) {
    for (const p of particles) {
      const alpha = p.life;
      const size = 2 + p.life * 3;

      ctx.beginPath();
      ctx.arc(p.x, p.y, size, 0, Math.PI * 2);
      ctx.fillStyle = `hsla(${p.hue}, 80%, 60%, ${alpha})`;
      ctx.fill();
    }
  }
</script>

<!-- VDF 可视化容器 -->
<div class="vdf-canvas-container">
  <canvas bind:this={canvas}></canvas>
</div>

<style>
  /* VDF 可视化容器 */
  .vdf-canvas-container {
    display: flex;
    justify-content: center;
    align-items: center;
    background: #16213e;
    border-radius: 12px;
    padding: 1rem;
  }

  /* Canvas 元素样式 */
  canvas {
    max-width: 100%;
    height: auto;
  }
</style>

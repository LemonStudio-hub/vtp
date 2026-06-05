<!--
  VDFCanvas Component

  A Canvas component that visualizes VDF computation progress, including:
  - Progress ring: displays the VDF computation completion percentage
  - Particle effect: generates dynamic particles based on computation speed
  - Percentage text: centered display of the completion percentage

  Features:
  - Real-time response to computation state
  - Particle count is proportional to computation speed
  - Smooth animation effects
  - Automatic animation start/stop

  Performance optimizations:
  - Uses requestAnimationFrame for smooth animation
  - Particle count limit to avoid performance issues
  - Automatic resource cleanup on component destroy

  Usage example:
  ```svelte
  <VDFCanvas />
  ```
-->

<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { workerState } from '$stores/worker';
  import { animationBudget } from '$stores/visibility';

  /** Reference to the Canvas element */
  let canvas: HTMLCanvasElement;

  /** Canvas 2D rendering context */
  let ctx: CanvasRenderingContext2D | null = null;

  /** Animation frame ID, used to cancel the animation */
  let animationFrame: number;

  /** Particle array */
  let particles: Particle[] = [];

  /** Orbit ring particles for ambient effect */
  let orbitParticles: OrbitParticle[] = [];

  /** Time counter for animations */
  let time = 0;

  /** Last frame timestamp for throttling */
  let lastFrameTime = 0;

  /** Current animation budget from visibility store */
  let currentBudget = 16;

  /**
   * Particle interface definition
   *
   * Describes the state of a single particle
   */
  interface Particle {
    /** X coordinate */
    x: number;

    /** Y coordinate */
    y: number;

    /** Velocity in the X direction */
    vx: number;

    /** Velocity in the Y direction */
    vy: number;

    /** Life value, range [0, 1] */
    life: number;

    /** Maximum life value (in frames) */
    maxLife: number;

    /** Hue value, range [0, 360] */
    hue: number;

    /** Size multiplier */
    size: number;
  }

  /**
   * Orbit particle interface
   *
   * Particles that orbit around the progress ring
   */
  interface OrbitParticle {
    /** Current angle in radians */
    angle: number;

    /** Distance from center */
    radius: number;

    /** Angular velocity */
    speed: number;

    /** Size */
    size: number;

    /** Hue */
    hue: number;

    /** Life value */
    life: number;
  }

  /**
   * Reactive animation start
   *
   * Starts the animation loop when the canvas is available
   * and computation is running
   */
  $: if (canvas && $workerState.isRunning) {
    startAnimation();
  }

  /**
   * Reactive animation stop
   *
   * Stops the animation loop when computation stops
   */
  $: if (!$workerState.isRunning && animationFrame) {
    stopAnimation();
  }

  /**
   * Subscribe to animation budget changes from visibility store.
   * When the tab is hidden, budget increases to ~1000ms (1fps)
   * to conserve CPU while still keeping the canvas responsive.
   */
  $: currentBudget = $animationBudget;

  /**
   * Initialize the Canvas when the component mounts
   *
   * Obtains the 2D rendering context and sets the canvas dimensions
   */
  onMount(() => {
    if (canvas) {
      ctx = canvas.getContext('2d');
      canvas.width = 450;
      canvas.height = 450;

      // Initialize orbit particles
      initOrbitParticles();

      // Start ambient animation even when not computing
      animate();
    }
  });

  /**
   * Clean up resources when the component is destroyed
   *
   * Cancels the animation frame to prevent memory leaks
   */
  onDestroy(() => {
    stopAnimation();
  });

  /**
   * Initialize orbit particles for ambient effect
   */
  function initOrbitParticles() {
    for (let i = 0; i < 20; i++) {
      orbitParticles.push({
        angle: Math.random() * Math.PI * 2,
        radius: 130 + Math.random() * 20,
        speed: 0.005 + Math.random() * 0.01,
        size: 1 + Math.random() * 2,
        hue: 140 + Math.random() * 40,
        life: Math.random()
      });
    }
  }

  /**
   * Start the animation loop
   *
   * Cancels any existing animation frame and starts a new animation loop
   */
  function startAnimation() {
    if (animationFrame) cancelAnimationFrame(animationFrame);
    animate();
  }

  /**
   * Stop the animation loop
   *
   * Cancels the current animation frame
   */
  function stopAnimation() {
    if (animationFrame) {
      cancelAnimationFrame(animationFrame);
    }
  }

  /**
   * Main animation loop
   *
   * Executes once per frame, responsible for:
   * 1. Clearing the canvas (semi-transparent effect for trailing)
   * 2. Drawing the progress ring
   * 3. Updating and drawing particles
   * 4. Spawning new particles
   * 5. Requesting the next frame
   *
   * Uses visibility-aware throttling: when the tab is hidden,
   * the frame rate drops to ~1fps to conserve CPU.
   */
  function animate() {
    if (!ctx || !canvas) return;

    const now = performance.now();
    const elapsed = now - lastFrameTime;

    // Throttle frame rate based on visibility
    if (elapsed < currentBudget) {
      animationFrame = requestAnimationFrame(animate);
      return;
    }

    lastFrameTime = now;
    const dt = elapsed / 1000; // Delta time in seconds
    time += Math.min(dt, 0.1); // Cap delta to avoid jumps after sleep

    // Clear canvas with slight trail effect
    ctx.fillStyle = 'rgba(10, 10, 26, 0.15)';
    ctx.fillRect(0, 0, canvas.width, canvas.height);

    // Calculate center point and progress
    const centerX = canvas.width / 2;
    const centerY = canvas.height / 2;
    const progress = $workerState.currentStep / ($workerState.totalSteps || 1000000);

    // Draw ambient glow
    drawAmbientGlow(ctx, centerX, centerY);

    // Draw orbit particles
    drawOrbitParticles(ctx, centerX, centerY);

    // Draw the progress ring
    drawProgressRing(ctx, centerX, centerY, progress);

    // Update and draw particles
    if ($workerState.isRunning) {
      updateParticles();
      drawParticles(ctx);

      // Spawn new particles based on speed
      if ($workerState.speed > 0) {
        spawnParticles(centerX, centerY, $workerState.speed);
      }
    }

    // Request the next frame
    animationFrame = requestAnimationFrame(animate);
  }

  /**
   * Draw ambient glow effect
   *
   * Creates a soft glowing background effect around the progress ring
   */
  function drawAmbientGlow(ctx: CanvasRenderingContext2D, x: number, y: number) {
    const glowRadius = 150 + Math.sin(time * 2) * 10;
    const gradient = ctx.createRadialGradient(x, y, 0, x, y, glowRadius);

    if ($workerState.isRunning) {
      gradient.addColorStop(0, 'rgba(0, 255, 136, 0.08)');
      gradient.addColorStop(0.5, 'rgba(0, 255, 136, 0.03)');
      gradient.addColorStop(1, 'rgba(0, 255, 136, 0)');
    } else {
      gradient.addColorStop(0, 'rgba(100, 100, 100, 0.05)');
      gradient.addColorStop(1, 'rgba(100, 100, 100, 0)');
    }

    ctx.fillStyle = gradient;
    ctx.fillRect(0, 0, ctx.canvas.width, ctx.canvas.height);
  }

  /**
   * Draw orbit particles
   *
   * Draws small particles orbiting around the progress ring
   */
  function drawOrbitParticles(ctx: CanvasRenderingContext2D, x: number, y: number) {
    for (const p of orbitParticles) {
      p.angle += p.speed;
      p.life = 0.3 + Math.sin(time * 2 + p.angle) * 0.3;

      const px = x + Math.cos(p.angle) * p.radius;
      const py = y + Math.sin(p.angle) * p.radius;

      ctx.beginPath();
      ctx.arc(px, py, p.size, 0, Math.PI * 2);
      ctx.fillStyle = `hsla(${p.hue}, 80%, 60%, ${p.life * 0.5})`;
      ctx.fill();
    }
  }

  /**
   * Draw the progress ring
   *
   * Draws two concentric arcs:
   * - Gray background ring: represents the total progress
   * - Green foreground ring: represents the current progress
   * - Center text: displays the percentage
   *
   * @param ctx - Canvas 2D rendering context
   * @param x - Center X coordinate
   * @param y - Center Y coordinate
   * @param progress - Progress value, range [0, 1]
   */
  function drawProgressRing(ctx: CanvasRenderingContext2D, x: number, y: number, progress: number) {
    const radius = 120;
    const lineWidth = 10;
    const startAngle = -Math.PI / 2; // Start from the top
    const endAngle = startAngle + 2 * Math.PI * progress;

    // Draw outer glow for the progress
    if (progress > 0) {
      ctx.save();
      ctx.shadowBlur = 20;
      ctx.shadowColor = 'rgba(0, 255, 136, 0.5)';
      ctx.beginPath();
      ctx.arc(x, y, radius, startAngle, endAngle);
      ctx.strokeStyle = 'rgba(0, 255, 136, 0.2)';
      ctx.lineWidth = lineWidth + 8;
      ctx.lineCap = 'round';
      ctx.stroke();
      ctx.restore();
    }

    // Draw the background ring (dark)
    ctx.beginPath();
    ctx.arc(x, y, radius, 0, 2 * Math.PI);
    ctx.strokeStyle = 'rgba(255, 255, 255, 0.08)';
    ctx.lineWidth = lineWidth;
    ctx.stroke();

    // Draw inner track
    ctx.beginPath();
    ctx.arc(x, y, radius - 15, 0, 2 * Math.PI);
    ctx.strokeStyle = 'rgba(255, 255, 255, 0.03)';
    ctx.lineWidth = 1;
    ctx.stroke();

    // Draw outer track
    ctx.beginPath();
    ctx.arc(x, y, radius + 15, 0, 2 * Math.PI);
    ctx.strokeStyle = 'rgba(255, 255, 255, 0.03)';
    ctx.lineWidth = 1;
    ctx.stroke();

    // Draw the foreground ring with gradient
    if (progress > 0) {
      const gradient = ctx.createConicGradient(startAngle, x, y);
      gradient.addColorStop(0, '#00ff88');
      gradient.addColorStop(progress, '#00cc6a');
      gradient.addColorStop(1, 'transparent');

      ctx.beginPath();
      ctx.arc(x, y, radius, startAngle, endAngle);
      ctx.strokeStyle = gradient;
      ctx.lineWidth = lineWidth;
      ctx.lineCap = 'round';
      ctx.stroke();

      // Draw progress end dot
      const endX = x + Math.cos(endAngle) * radius;
      const endY = y + Math.sin(endAngle) * radius;

      ctx.beginPath();
      ctx.arc(endX, endY, 6, 0, Math.PI * 2);
      ctx.fillStyle = '#00ff88';
      ctx.fill();

      // Outer glow for the dot
      ctx.beginPath();
      ctx.arc(endX, endY, 10, 0, Math.PI * 2);
      ctx.fillStyle = 'rgba(0, 255, 136, 0.3)';
      ctx.fill();
    }

    // Draw tick marks
    for (let i = 0; i < 60; i++) {
      const angle = (i / 60) * Math.PI * 2 - Math.PI / 2;
      const innerRadius = i % 5 === 0 ? radius - 20 : radius - 15;
      const outerRadius = radius - 12;

      ctx.beginPath();
      ctx.moveTo(x + Math.cos(angle) * innerRadius, y + Math.sin(angle) * innerRadius);
      ctx.lineTo(x + Math.cos(angle) * outerRadius, y + Math.sin(angle) * outerRadius);
      ctx.strokeStyle = i % 5 === 0 ? 'rgba(255, 255, 255, 0.15)' : 'rgba(255, 255, 255, 0.05)';
      ctx.lineWidth = i % 5 === 0 ? 2 : 1;
      ctx.stroke();
    }

    // Draw center percentage text
    const percentage = (progress * 100).toFixed(1);

    // Text shadow
    ctx.fillStyle = 'rgba(0, 0, 0, 0.5)';
    ctx.font = 'bold 36px "Courier New", monospace';
    ctx.textAlign = 'center';
    ctx.textBaseline = 'middle';
    ctx.fillText(`${percentage}%`, x + 1, y + 1);

    // Main text
    ctx.fillStyle = $workerState.isRunning ? '#00ff88' : '#666';
    ctx.fillText(`${percentage}%`, x, y);

    // Subtitle
    ctx.font = '12px "Courier New", monospace';
    ctx.fillStyle = 'rgba(255, 255, 255, 0.3)';
    ctx.fillText('VDF Progress', x, y + 25);
  }

  /**
   * Spawn new particles
   *
   * Generates a number of particles proportional to the computation speed.
   * The particle count is proportional to speed, with a maximum of 8.
   *
   * @param x - Spawn center X coordinate
   * @param y - Spawn center Y coordinate
   * @param speed - Computation speed (steps/second)
   */
  function spawnParticles(x: number, y: number, speed: number) {
    // Calculate particle count based on speed, capped at 8
    const count = Math.min(Math.floor(speed / 80000), 8);

    for (let i = 0; i < count; i++) {
      // Random angle and speed
      const angle = Math.random() * Math.PI * 2;
      const velocity = 0.5 + Math.random() * 3;

      particles.push({
        x: x + Math.cos(angle) * 120,
        y: y + Math.sin(angle) * 120,
        vx: Math.cos(angle) * velocity,
        vy: Math.sin(angle) * velocity,
        life: 1,
        maxLife: 40 + Math.random() * 60,
        hue: 140 + Math.random() * 40,
        size: 1 + Math.random() * 2
      });
    }
  }

  /**
   * Update particle states
   *
   * Updates the position and life value of all particles.
   * Removes particles whose life value has reached 0.
   */
  function updateParticles() {
    // Remove dead particles
    particles = particles.filter((p) => p.life > 0);

    // Update particle positions and life values
    for (const p of particles) {
      p.x += p.vx;
      p.y += p.vy;
      p.vx *= 0.98; // Slight drag
      p.vy *= 0.98;
      p.life -= 1 / p.maxLife;
    }
  }

  /**
   * Draw particles
   *
   * Draws all living particles.
   * Particle size and opacity are proportional to their life value.
   *
   * @param ctx - Canvas 2D rendering context
   */
  function drawParticles(ctx: CanvasRenderingContext2D) {
    for (const p of particles) {
      const alpha = p.life * 0.8;
      const size = p.size * p.life;

      // Outer glow
      ctx.beginPath();
      ctx.arc(p.x, p.y, size * 3, 0, Math.PI * 2);
      ctx.fillStyle = `hsla(${p.hue}, 80%, 60%, ${alpha * 0.2})`;
      ctx.fill();

      // Core
      ctx.beginPath();
      ctx.arc(p.x, p.y, size, 0, Math.PI * 2);
      ctx.fillStyle = `hsla(${p.hue}, 80%, 70%, ${alpha})`;
      ctx.fill();
    }
  }
</script>

<!-- VDF visualization container -->
<div class="vdf-canvas-container" class:running={$workerState.isRunning}>
  <canvas bind:this={canvas}></canvas>

  <!-- Status overlay -->
  <div class="status-overlay">
    {#if !$workerState.isRunning}
      <span class="status-text">Idle</span>
    {/if}
  </div>
</div>

<style>
  /* VDF visualization container */
  .vdf-canvas-container {
    position: relative;
    display: flex;
    justify-content: center;
    align-items: center;
    border-radius: 16px;
    padding: 1rem;
    transition: all 0.3s ease;
  }

  .vdf-canvas-container.running {
    box-shadow: 0 0 30px rgba(0, 255, 136, 0.1);
  }

  /* Canvas element styles */
  canvas {
    max-width: 100%;
    height: auto;
  }

  /* Status overlay */
  .status-overlay {
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    pointer-events: none;
  }

  .status-text {
    font-family: 'Courier New', monospace;
    font-size: 0.875rem;
    color: rgba(255, 255, 255, 0.2);
    letter-spacing: 0.1em;
    text-transform: uppercase;
  }
</style>

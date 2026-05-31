<!--
  PWAInstall 组件

  提供 PWA 应用安装功能，包括：
  - 监听浏览器安装提示事件
  - 显示自定义安装按钮
  - 处理用户安装选择

  功能特点：
  - 自动检测 PWA 安装条件
  - 自定义安装提示界面
  - 处理安装成功/失败事件
  - 响应式显示/隐藏

  浏览器兼容性：
  - Chrome 67+
  - Edge 79+
  - Firefox 不支持（需要手动添加到主屏幕）

  使用示例：
  ```svelte
  <PWAInstall />
  ```
-->

<script lang="ts">
  import { onMount } from 'svelte';

  /**
   * 延迟的安装提示事件
   *
   * 浏览器在满足 PWA 安装条件时触发 beforeinstallprompt 事件。
   * 我们阻止默认行为并保存事件引用，以便后续调用。
   */
  let deferredPrompt: any = null;

  /**
   * 是否显示安装按钮
   *
   * 当 deferredPrompt 可用时显示安装按钮
   */
  let showInstallButton = false;

  /**
   * 组件挂载时注册事件监听器
   *
   * 监听两个关键事件：
   * 1. beforeinstallprompt: 浏览器准备显示安装提示
   * 2. appinstalled: 应用安装成功
   */
  onMount(() => {
    /**
     * 监听 beforeinstallprompt 事件
     *
     * 当浏览器检测到 PWA 满足安装条件时触发。
     * 我们阻止默认的安装提示，改为显示自定义按钮。
     */
    window.addEventListener('beforeinstallprompt', (e) => {
      // 阻止默认的安装提示
      e.preventDefault();

      // 保存事件引用，用于后续调用
      deferredPrompt = e;

      // 显示自定义安装按钮
      showInstallButton = true;
    });

    /**
     * 监听 appinstalled 事件
     *
     * 当 PWA 安装成功时触发。
     * 清理状态并隐藏安装按钮。
     */
    window.addEventListener('appinstalled', () => {
      // 隐藏安装按钮
      showInstallButton = false;

      // 清理事件引用
      deferredPrompt = null;
    });
  });

  /**
   * 处理安装按钮点击
   *
   * 调用浏览器的安装提示，让用户选择是否安装。
   *
   * 工作流程：
   * 1. 检查是否有可用的安装提示
   * 2. 显示浏览器的安装对话框
   * 3. 根据用户选择更新状态
   */
  async function handleInstall() {
    if (!deferredPrompt) return;

    // 显示浏览器的安装对话框
    deferredPrompt.prompt();

    // 等待用户选择
    const { outcome } = await deferredPrompt.userChoice;

    // 根据用户选择更新状态
    if (outcome === 'accepted') {
      showInstallButton = false;
    }

    // 清理事件引用
    deferredPrompt = null;
  }
</script>

<!-- 条件渲染安装按钮 -->
{#if showInstallButton}
  <button class="install-button" on:click={handleInstall}> 📲 安装应用 </button>
{/if}

<style>
  /* 安装按钮样式 */
  .install-button {
    padding: 0.75rem 1.5rem;
    background: #00ff88;
    color: #1a1a2e;
    border: none;
    border-radius: 8px;
    font-size: 1rem;
    font-weight: bold;
    cursor: pointer;
    transition: all 0.2s;
  }

  /* 按钮悬停效果 */
  .install-button:hover {
    background: #00cc6a;
    transform: translateY(-2px);
  }

  /* 按钮点击效果 */
  .install-button:active {
    transform: translateY(0);
  }
</style>

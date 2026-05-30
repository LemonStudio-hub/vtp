/**
 * Worker 状态管理模块
 *
 * 使用 Svelte Store 管理 Web Worker 的状态和事件。
 * 提供全局状态管理，供各组件订阅和更新。
 *
 * 主要功能：
 * 1. Worker 实例管理
 * 2. 计算状态跟踪
 * 3. 事件日志管理
 * 4. 进度计算
 *
 * 使用示例：
 * ```typescript
 * import { workerState, addEvent } from '$stores/worker';
 *
 * // 订阅状态变化
 * workerState.subscribe(state => {
 *   console.log('Current step:', state.currentStep);
 * });
 *
 * // 添加事件
 * addEvent({ type: 'info', message: 'Computation started' });
 * ```
 */

import { writable, derived } from 'svelte/store';

/**
 * VTP 事件接口
 *
 * 描述 VDF 计算过程中的事件
 */
export interface VtpEvent {
  /** 事件类型 */
  type: 'info' | 'checkpoint' | 'winner' | 'error';

  /** 事件时间戳（Unix 毫秒） */
  timestamp: number;

  /** 事件消息 */
  message: string;
}

/**
 * Worker 状态接口
 *
 * 描述 Web Worker 的当前状态
 */
export interface WorkerState {
  /** 是否正在运行 */
  isRunning: boolean;

  /** 是否已暂停 */
  isPaused: boolean;

  /** 当前已完成的 VDF 步数 */
  currentStep: number;

  /** 总步数目标 */
  totalSteps: number;

  /** 当前计算速度（步/秒） */
  speed: number;

  /** 在线时长（秒） */
  uptime: number;

  /** 抽签次数 */
  winnerCount: number;

  /** 运气指数（百分比） */
  luckPercentage: number;

  /** 节点公钥 */
  publicKey: Uint8Array | null;

  /** 节点 ID */
  nodeId: string;
}

/**
 * Worker 实例 Store
 *
 * 存储当前 Web Worker 实例的引用。
 * 用于向 Worker 发送控制命令。
 */
export const workerStore = writable<Worker | null>(null);

/**
 * 事件日志 Store
 *
 * 存储 VDF 计算过程中的事件列表。
 * 最多保留 50 条事件，自动移除旧事件。
 */
export const events = writable<VtpEvent[]>([]);

/**
 * Worker 状态 Store
 *
 * 存储 Worker 的当前状态，包括：
 * - 运行状态
 * - 计算进度
 * - 性能指标
 * - 节点信息
 */
export const workerState = writable<WorkerState>({
  isRunning: false,
  isPaused: false,
  currentStep: 0,
  totalSteps: 0,
  speed: 0,
  uptime: 0,
  winnerCount: 0,
  luckPercentage: 100,
  publicKey: null,
  nodeId: '---'
});

/**
 * 计算进度 Store（派生）
 *
 * 从 workerState 派生的计算进度值，范围 [0, 1]。
 * 用于进度条和进度环显示。
 *
 * @example
 * ```typescript
 * import { progress } from '$stores/worker';
 *
 * progress.subscribe(value => {
 *   console.log('Progress:', (value * 100).toFixed(1) + '%');
 * });
 * ```
 */
export const progress = derived(workerState, ($state) => {
  if ($state.totalSteps === 0) return 0;
  return $state.currentStep / $state.totalSteps;
});

/**
 * 添加事件到事件日志
 *
 * 将新事件添加到事件列表的开头，并自动限制列表长度。
 *
 * @param event - 要添加的事件（不包含时间戳）
 *
 * @example
 * ```typescript
 * addEvent({ type: 'info', message: 'Computation started' });
 * addEvent({ type: 'winner', message: '🎉 Winner at step 12345' });
 * ```
 *
 * @注意
 * - 时间戳会自动添加
 * - 事件列表最多保留 50 条
 * - 新事件添加到列表开头
 */
export function addEvent(event: Omit<VtpEvent, 'timestamp'>) {
  events.update((current) => [
    { ...event, timestamp: Date.now() },
    ...current.slice(0, 49)
  ]);
}

/**
 * 重置 Worker 状态
 *
 * 将所有状态重置为初始值，用于：
 * - 重新开始计算
 * - 清除错误状态
 * - 初始化应用
 *
 * @example
 * ```typescript
 * resetWorkerState();
 * ```
 *
 * @注意
 * - 会清空所有事件日志
 * - 会重置所有计算状态
 * - 不会终止当前 Worker
 */
export function resetWorkerState() {
  workerState.set({
    isRunning: false,
    isPaused: false,
    currentStep: 0,
    totalSteps: 0,
    speed: 0,
    uptime: 0,
    winnerCount: 0,
    luckPercentage: 100,
    publicKey: null,
    nodeId: '---'
  });
  events.set([]);
}

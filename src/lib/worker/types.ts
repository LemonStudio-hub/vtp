/**
 * Worker 通信类型定义
 *
 * 定义了 Web Worker 与主线程之间的通信消息类型。
 * 使用 TypeScript 接口确保类型安全。
 *
 * 消息流向：
 * - 主线程 → Worker: WorkerMessage（控制命令）
 * - Worker → 主线程: WorkerResponse（状态更新）
 *
 * 使用示例：
 * ```typescript
 * import type { WorkerMessage, ProgressMessage } from './types';
 *
 * // 发送控制命令
 * const startMsg: WorkerMessage = {
 *   type: 'start',
 *   seed: new Uint8Array(32),
 *   total: 1000000,
 *   k: 1000,
 *   tau: new Uint8Array(32),
 *   checkpointInterval: 100000
 * };
 * worker.postMessage(startMsg);
 *
 * // 接收进度更新
 * worker.onmessage = (event: MessageEvent<WorkerResponse>) => {
 *   if (event.data.type === 'progress') {
 *     const msg = event.data as ProgressMessage;
 *     console.log(`Step: ${msg.step}, Speed: ${msg.speed}`);
 *   }
 * };
 * ```
 *
 * @module types
 */

/**
 * Worker 消息接口
 *
 * 主线程发送给 Worker 的控制消息。
 * 支持的命令类型：start/pause/resume/stop
 */
export interface WorkerMessage {
  /** 命令类型 */
  type: string;

  /** VDF 计算种子，至少 32 字节 */
  seed?: Uint8Array;

  /** VDF 总步数目标 */
  total?: number;

  /** VRF 抽签间隔 */
  k?: number;

  /** VRF 阈值，32 字节 */
  tau?: Uint8Array;

  /** 检查点间隔 */
  checkpointInterval?: number;

  /** 每批最大步数 */
  maxSteps?: number;
}

/**
 * 进度消息接口
 *
 * Worker 定期发送给主线程的进度报告。
 * 每秒发送一次，包含当前步数、速度和内存使用量。
 */
export interface ProgressMessage {
  /** 消息类型 */
  type: 'progress';

  /** 当前已完成的 VDF 步数 */
  step: number;

  /** 当前计算速度（步/秒） */
  speed: number;

  /** 当前内存使用量（字节） */
  memoryUsage: number;
}

/**
 * 中签消息接口
 *
 * 当发现中签时发送给主线程的消息。
 * 包含中签步数和 VRF 证明。
 */
export interface WinnerMessage {
  /** 消息类型 */
  type: 'winner';

  /** 中签步数 */
  step: number;

  /** VRF 证明 */
  proof: Uint8Array;
}

/**
 * 心跳消息接口
 *
 * 每 10 秒发送一次，保持与主线程的连接活跃。
 * 用于监控 Worker 存活状态。
 */
export interface HeartbeatMessage {
  /** 消息类型 */
  type: 'heartbeat';

  /** 消息时间戳（Unix 毫秒） */
  timestamp: number;

  /** 当前状态：running/paused */
  status: string;
}

/**
 * 错误消息接口
 *
 * 发生错误时发送给主线程的消息。
 * 包含错误代码、描述和是否可恢复。
 */
export interface ErrorMessage {
  /** 消息类型 */
  type: 'error';

  /** 错误代码，用于程序化处理 */
  code: string;

  /** 错误描述，用于显示 */
  message: string;

  /** 是否可恢复 */
  recoverable: boolean;
}

/**
 * 启动消息接口
 *
 * Worker 成功初始化后发送给主线程的消息。
 * 包含节点的 VRF 公钥。
 */
export interface StartedMessage {
  /** 消息类型 */
  type: 'started';

  /** 节点的 VRF 公钥 */
  publicKey: Uint8Array;
}

/**
 * 停止消息接口
 *
 * Worker 停止后发送给主线程的消息。
 */
export interface StoppedMessage {
  /** 消息类型 */
  type: 'stopped';
}

/**
 * 完成消息接口
 *
 * VDF 计算完成后发送给主线程的消息。
 * 包含最终步数。
 */
export interface FinishedMessage {
  /** 消息类型 */
  type: 'finished';

  /** 最终步数 */
  step: number;
}

/**
 * Worker 响应类型联合
 *
 * 所有可能的 Worker 响应消息类型。
 * 用于类型安全的消息处理。
 */
export type WorkerResponse =
  | ProgressMessage
  | WinnerMessage
  | HeartbeatMessage
  | ErrorMessage
  | StartedMessage
  | StoppedMessage
  | FinishedMessage;

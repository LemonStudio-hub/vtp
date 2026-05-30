/**
 * VTP Web Worker 主模块
 *
 * 负责在后台线程中执行 VDF 计算任务。
 * 通过 postMessage 与主线程进行通信。
 *
 * 主要功能：
 * 1. 接收主线程的控制命令（start/pause/resume/stop）
 * 2. 执行 VDF 批量计算
 * 3. 定期报告计算进度
 * 4. 发送心跳包保持连接
 * 5. 报告中签事件和错误
 *
 * 通信协议：
 * - 接收：WorkerMessage（包含命令和参数）
 * - 发送：ProgressMessage | WinnerMessage | HeartbeatMessage | ErrorMessage
 *
 * 性能考虑：
 * - 使用时间片机制避免阻塞浏览器
 * - 批量处理减少通信开销
 * - 定期内存监控防止泄漏
 */

import { Session } from '../vtp-core/pkg/vtp_core';
import type { BatchResult } from '../vtp-core/pkg/vtp_core';

/**
 * Worker 消息接口
 *
 * 主线程发送给 Worker 的控制消息
 */
interface WorkerMessage {
  /** 命令类型：start/pause/resume/stop */
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
 * Worker 定期发送给主线程的进度报告
 */
interface ProgressMessage {
  type: 'progress';

  /** 当前已完成的 VDF 步数 */
  step: number;

  /** 当前计算速度（步/秒） */
  speed: number;

  /** 当前内存使用量（字节） */
  memoryUsage: number;
}

/**
 * 批量计算结果类型
 *
 * 对应 Rust 中的 BatchResult 枚举
 */
type BatchResultType = 'Progress' | 'Winner' | 'Finished' | 'Error';

/**
 * 中签消息接口
 *
 * 当发现中签时发送给主线程的消息
 */
interface WinnerMessage {
  type: 'winner';

  /** 中签步数 */
  step: number;

  /** VRF 证明 */
  proof: Uint8Array;
}

/**
 * 心跳消息接口
 *
 * 定期发送给主线程保持连接活跃
 */
interface HeartbeatMessage {
  type: 'heartbeat';

  /** 消息时间戳 */
  timestamp: number;

  /** 当前状态：running/paused */
  status: string;
}

/**
 * 错误消息接口
 *
 * 发生错误时发送给主线程的消息
 */
interface ErrorMessage {
  type: 'error';

  /** 错误代码 */
  code: string;

  /** 错误描述 */
  message: string;

  /** 是否可恢复 */
  recoverable: boolean;
}

/** Worker 响应类型联合 */
type WorkerResponse = ProgressMessage | WinnerMessage | HeartbeatMessage | ErrorMessage;

// ==================== 状态变量 ====================

/** 当前 VDF 会话实例 */
let session: Session | null = null;

/** 是否正在运行 */
let isRunning = false;

/** 是否已暂停 */
let isPaused = false;

/** 计算开始时间 */
let startTime = 0;

/** 上次报告时间 */
let lastReportTime = 0;

/** 当前步数 */
let stepCount = 0;

/** 当前计算速度 */
let speed = 0;

/** 心跳定时器 */
let heartbeatInterval: ReturnType<typeof setInterval> | null = null;

// ==================== 消息处理 ====================

/**
 * 处理主线程发送的消息
 *
 * 根据消息类型分发到对应的处理函数。
 * 支持的命令：start/pause/resume/stop
 *
 * @param event - 消息事件，包含 WorkerMessage 数据
 */
self.onmessage = async (event: MessageEvent<WorkerMessage>) => {
  const { type, ...params } = event.data;

  switch (type) {
    case 'start':
      await handleStart(params);
      break;
    case 'pause':
      handlePause();
      break;
    case 'resume':
      handleResume();
      break;
    case 'stop':
      handleStop();
      break;
    default:
      sendError('UNKNOWN_COMMAND', `Unknown command: ${type}`, false);
  }
};

// ==================== 命令处理函数 ====================

/**
 * 处理 start 命令
 *
 * 初始化 VDF 会话并开始计算。
 *
 * 工作流程：
 * 1. 验证参数完整性
 * 2. 清理旧会话
 * 3. 创建新 Session 实例
 * 4. 初始化状态变量
 * 5. 启动心跳
 * 6. 开始主循环
 *
 * @param params - 包含 VDF 配置参数的 WorkerMessage
 */
async function handleStart(params: WorkerMessage) {
  try {
    const { seed, total, k, tau, checkpointInterval } = params;

    // 验证必需参数
    if (!seed || !total || !k || !tau || !checkpointInterval) {
      sendError('INVALID_PARAMS', 'Missing required parameters', false);
      return;
    }

    // 清理旧会话
    if (session) {
      session.free();
    }

    // 创建新会话
    session = new Session(seed, total, k, tau, checkpointInterval);
    isRunning = true;
    isPaused = false;
    startTime = Date.now();
    lastReportTime = startTime;
    stepCount = 0;

    // 通知主线程会话已启动
    self.postMessage({
      type: 'started',
      publicKey: session.public_key()
    });

    // 启动心跳和主循环
    startHeartbeat();
    runMainLoop();
  } catch (error) {
    sendError('INIT_FAILED', `Failed to initialize session: ${error}`, true);
  }
}

/**
 * 处理 pause 命令
 *
 * 暂停 VDF 计算。
 * 暂停后主循环会停止执行，但会话状态保留。
 */
function handlePause() {
  if (session && isRunning) {
    session.pause();
    isPaused = true;
    stopHeartbeat();
  }
}

/**
 * 处理 resume 命令
 *
 * 恢复 VDF 计算。
 * 从暂停点继续执行，重新启动心跳和主循环。
 */
function handleResume() {
  if (session && isRunning && isPaused) {
    session.resume();
    isPaused = false;
    startHeartbeat();
    runMainLoop();
  }
}

/**
 * 处理 stop 命令
 *
 * 停止 VDF 计算并清理资源。
 */
function handleStop() {
  isRunning = false;
  isPaused = false;
  stopHeartbeat();

  if (session) {
    session.free();
    session = null;
  }

  self.postMessage({ type: 'stopped' });
}

// ==================== 核心计算循环 ====================

/**
 * 主计算循环
 *
 * 持续执行 VDF 批量计算，直到会话完成或被暂停/停止。
 *
 * 性能优化策略：
 * 1. 批量处理：每次执行 BATCH_SIZE 步，减少函数调用开销
 * 2. 时间片控制：每轮计算不超过 TIME_SLICE_MS，避免阻塞浏览器
 * 3. 定期报告：每 REPORT_INTERVAL_MS 毫秒报告一次进度
 * 4. 错误恢复：捕获异常并延迟重试
 *
 * 常量说明：
 * - BATCH_SIZE = 1000：每批处理步数
 * - TIME_SLICE_MS = 50：时间片长度（毫秒）
 * - REPORT_INTERVAL_MS = 1000：进度报告间隔（毫秒）
 */
async function runMainLoop() {
  const BATCH_SIZE = 1000;
  const TIME_SLICE_MS = 50;
  const REPORT_INTERVAL_MS = 1000;

  while (isRunning && !isPaused) {
    const loopStart = performance.now();

    try {
      if (!session) break;

      // 执行批量 VDF 计算
      const result = session.run_batch(BATCH_SIZE) as BatchResultType;
      stepCount = session.state().step;

      // 计算并报告进度
      const now = Date.now();
      const elapsed = (now - lastReportTime) / 1000;

      if (elapsed >= REPORT_INTERVAL_MS / 1000) {
        // 计算当前速度（步/秒）
        speed = (stepCount - (stepCount - BATCH_SIZE)) / elapsed;

        const progressMsg: ProgressMessage = {
          type: 'progress',
          step: stepCount,
          speed: speed,
          memoryUsage: getMemoryUsage()
        };
        self.postMessage(progressMsg);

        lastReportTime = now;
      }

      // 处理中签事件
      if (result === 'Winner') {
        const proof = session.get_checkpoint_data();
        const winnerMsg: WinnerMessage = {
          type: 'winner',
          step: stepCount,
          proof: new Uint8Array(proof)
        };
        self.postMessage(winnerMsg);
      }

      // 处理完成事件
      if (result === 'Finished') {
        isRunning = false;
        self.postMessage({ type: 'finished', step: stepCount });
        break;
      }

      // 处理错误事件
      if (result === 'Error') {
        sendError('VDF_ERROR', 'VDF computation error occurred', true);
      }

      // 时间片控制：如果计算时间不足，等待剩余时间
      const elapsed_ms = performance.now() - loopStart;
      if (elapsed_ms < TIME_SLICE_MS) {
        await sleep(TIME_SLICE_MS - elapsed_ms);
      }
    } catch (error) {
      sendError('COMPUTATION_ERROR', `Error during computation: ${error}`, true);
      // 错误后延迟重试
      await sleep(1000);
    }
  }
}

// ==================== 辅助函数 ====================

/**
 * 发送错误消息给主线程
 *
 * @param code - 错误代码，用于程序化处理
 * @param message - 错误描述，用于显示
 * @param recoverable - 是否可恢复，影响前端处理策略
 */
function sendError(code: string, message: string, recoverable: boolean) {
  const errorMsg: ErrorMessage = {
    type: 'error',
    code,
    message,
    recoverable
  };
  self.postMessage(errorMsg);
}

/**
 * 获取当前内存使用量
 *
 * 使用 performance.memory API 获取 JavaScript 堆内存使用量。
 *
 * @returns 内存使用量（字节），如果不支持则返回 0
 *
 * 兼容性说明：
 * - performance.memory 仅在 Chrome/Edge 浏览器中可用
 * - Firefox 和 Safari 不支持此 API
 * - 使用类型断言避免 TypeScript 错误
 */
function getMemoryUsage(): number {
  // 检查是否支持 performance.memory API
  // 使用类型断言因为 TypeScript 默认不包含此属性
  const perf = performance as any;
  if (perf.memory && typeof perf.memory.usedJSHeapSize === 'number') {
    return perf.memory.usedJSHeapSize;
  }
  return 0;
}

/**
 * 启动心跳定时器
 *
 * 每 10 秒发送一次心跳消息，保持与主线程的连接活跃。
 * 心跳消息包含当前状态，用于主线程监控 Worker 存活。
 */
function startHeartbeat() {
  stopHeartbeat();
  heartbeatInterval = setInterval(() => {
    const heartbeatMsg: HeartbeatMessage = {
      type: 'heartbeat',
      timestamp: Date.now(),
      status: isPaused ? 'paused' : 'running'
    };
    self.postMessage(heartbeatMsg);
  }, 10000);
}

/**
 * 停止心跳定时器
 *
 * 清除心跳定时器并重置引用。
 */
function stopHeartbeat() {
  if (heartbeatInterval) {
    clearInterval(heartbeatInterval);
    heartbeatInterval = null;
  }
}

/**
 * 异步休眠函数
 *
 * 返回一个 Promise，在指定毫秒后 resolve。
 * 用于时间片控制和错误重试延迟。
 *
 * @param ms - 休眠时间（毫秒）
 * @returns Promise<void>
 */
function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

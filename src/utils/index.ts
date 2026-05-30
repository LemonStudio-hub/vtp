/**
 * 工具函数模块
 *
 * 提供常用的工具函数，包括：
 * - 数字格式化
 * - 字节格式化
 * - 时间格式化
 * - ID 生成
 * - 异步工具
 * - 函数防抖
 *
 * 使用示例：
 * ```typescript
 * import { formatBytes, formatTime, generateNodeId } from '$utils';
 *
 * console.log(formatBytes(1024)); // "1 KB"
 * console.log(formatTime(3661)); // "01:01:01"
 * console.log(generateNodeId()); // "a1b2c3d4"
 * ```
 */

/**
 * 格式化字节数
 *
 * 将字节数转换为人类可读的格式，自动选择合适的单位。
 *
 * @param bytes - 字节数
 * @returns 格式化后的字符串
 *
 * @example
 * ```typescript
 * formatBytes(0) // "0 B"
 * formatBytes(1024) // "1 KB"
 * formatBytes(1048576) // "1 MB"
 * formatBytes(1073741824) // "1 GB"
 * ```
 *
 * @注意
 * - 使用 1024 作为基数（二进制）
 * - 最大单位为 GB
 * - 保留两位小数
 */
export function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(2))} ${sizes[i]}`;
}

/**
 * 格式化数字
 *
 * 将数字转换为本地化格式，添加千位分隔符。
 *
 * @param num - 要格式化的数字
 * @returns 格式化后的字符串
 *
 * @example
 * ```typescript
 * formatNumber(1234567) // "1,234,567"
 * formatNumber(1234.567) // "1,234.567"
 * ```
 *
 * @注意
 * - 使用当前环境的本地化设置
 * - 保留原始精度
 */
export function formatNumber(num: number): string {
  return num.toLocaleString();
}

/**
 * 格式化速度
 *
 * 将速度值（步/秒）转换为人类可读的格式，自动选择合适的单位。
 *
 * @param speed - 速度值（步/秒）
 * @returns 格式化后的字符串
 *
 * @example
 * ```typescript
 * formatSpeed(1500000) // "1.5M"
 * formatSpeed(1500) // "1.5K"
 * formatSpeed(500) // "500"
 * ```
 *
 * @注意
 * - >= 1,000,000 使用 M（百万）
 * - >= 1,000 使用 K（千）
 * - 其他使用原始值
 * - 保留一位小数
 */
export function formatSpeed(speed: number): string {
  if (speed >= 1000000) {
    return `${(speed / 1000000).toFixed(1)}M`;
  } else if (speed >= 1000) {
    return `${(speed / 1000).toFixed(1)}K`;
  }
  return speed.toFixed(0);
}

/**
 * 格式化时间
 *
 * 将秒数转换为 HH:MM:SS 格式。
 *
 * @param seconds - 秒数
 * @returns 格式化后的时间字符串
 *
 * @example
 * ```typescript
 * formatTime(0) // "00:00:00"
 * formatTime(61) // "00:01:01"
 * formatTime(3661) // "01:01:01"
 * ```
 *
 * @注意
 * - 每个部分使用两位数字，不足补零
 * - 支持超过 24 小时的时间
 */
export function formatTime(seconds: number): string {
  const h = Math.floor(seconds / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  const s = Math.floor(seconds % 60);
  return `${h.toString().padStart(2, '0')}:${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`;
}

/**
 * 生成节点 ID
 *
 * 生成一个 8 位的十六进制节点 ID。
 *
 * @returns 8 位十六进制字符串
 *
 * @example
 * ```typescript
 * generateNodeId() // "a1b2c3d4"
 * generateNodeId() // "e5f6g7h8"
 * ```
 *
 * @注意
 * - 使用 Math.random()，不适合密码学用途
 * - 生成的 ID 可能重复（概率极低）
 */
export function generateNodeId(): string {
  const chars = '0123456789abcdef';
  let result = '';
  for (let i = 0; i < 8; i++) {
    result += chars[Math.floor(Math.random() * chars.length)];
  }
  return result;
}

/**
 * 异步休眠
 *
 * 返回一个 Promise，在指定毫秒后 resolve。
 *
 * @param ms - 休眠时间（毫秒）
 * @returns Promise<void>
 *
 * @example
 * ```typescript
 * async function example() {
 *   console.log('Start');
 *   await sleep(1000);
 *   console.log('End');
 * }
 * ```
 *
 * @注意
 * - 使用 setTimeout 实现
 * - 可以使用 await 等待
 */
export function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

/**
 * 函数防抖
 *
 * 创建一个防抖函数，在指定时间内多次调用只执行最后一次。
 *
 * @param func - 要防抖的函数
 * @param wait - 等待时间（毫秒）
 * @returns 防抖后的函数
 *
 * @example
 * ```typescript
 * const debouncedSearch = debounce((query: string) => {
 *   console.log('Searching:', query);
 * }, 300);
 *
 * // 快速输入时，只在停止输入 300ms 后执行一次
 * debouncedSearch('a');
 * debouncedSearch('ab');
 * debouncedSearch('abc');
 * ```
 *
 * @注意
 * - 使用 setTimeout 实现
 * - 每次调用会重置计时器
 * - 保留最后一次调用的参数
 */
export function debounce<T extends (...args: any[]) => any>(
  func: T,
  wait: number
): (...args: Parameters<T>) => void {
  let timeout: ReturnType<typeof setTimeout>;
  return (...args: Parameters<T>) => {
    clearTimeout(timeout);
    timeout = setTimeout(() => func(...args), wait);
  };
}

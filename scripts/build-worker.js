/**
 * Worker 构建脚本
 *
 * 使用 esbuild 将 TypeScript Worker 源码编译为浏览器可用的 JavaScript。
 *
 * 功能：
 * 1. 将 TypeScript 编译为 JavaScript
 * 2. 打包所有依赖
 * 3. 生成 source map
 * 4. 生产环境自动压缩
 *
 * 使用方法：
 * ```bash
 * npm run worker:build
 * ```
 *
 * 构建输出：
 * - static/worker.js - 编译后的 Worker 代码
 * - static/worker.js.map - Source map 文件
 *
 * 依赖：
 * - esbuild: 高性能 JavaScript/TypeScript 打包工具
 *
 * @module build-worker
 */

import { build } from 'esbuild';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';

// 获取当前文件路径（ES Module 环境）
const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

/**
 * 构建 Worker
 *
 * 使用 esbuild 编译和打包 Worker 源码。
 *
 * 配置说明：
 * - entryPoints: 入口文件路径
 * - bundle: 打包所有依赖到一个文件
 * - outfile: 输出文件路径
 * - format: 输出格式为 ES Module
 * - target: 目标环境为现代浏览器
 * - platform: 目标平台为浏览器
 * - sourcemap: 生成 source map 便于调试
 * - minify: 生产环境自动压缩
 *
 * @throws {Error} 构建失败时抛出错误
 */
async function buildWorker() {
  try {
    await build({
      // 入口文件：Worker TypeScript 源码
      entryPoints: [join(__dirname, '../src/lib/worker/index.ts')],

      // 打包所有依赖到单个文件
      bundle: true,

      // 输出文件路径
      outfile: join(__dirname, '../static/worker.js'),

      // 输出格式：ES Module
      format: 'esm',

      // 目标环境：现代浏览器
      target: 'esnext',

      // 目标平台：浏览器
      platform: 'browser',

      // 生成 source map 便于调试
      sourcemap: true,

      // 生产环境自动压缩
      minify: process.env.NODE_ENV === 'production'
    });

    console.log('Worker built successfully');
  } catch (error) {
    console.error('Worker build failed:', error);
    process.exit(1);
  }
}

// 执行构建
buildWorker();

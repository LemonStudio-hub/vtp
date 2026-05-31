//! 错误处理模块
//!
//! 定义了 VTP 库的错误类型和错误处理器。
//! 提供统一的错误处理机制，支持错误重试和恢复。
//!
//! # 错误类型
//! - `InvalidInput`: 输入参数无效
//! - `InvalidState`: 内部状态无效
//! - `ComputationFailed`: 计算失败
//! - `CheckpointFailed`: 检查点保存/加载失败
//! - `SessionFinished`: 会话已完成
//! - `SessionNotStarted`: 会话未开始
//!
//! # 错误处理策略
//! 使用 ErrorHandler 结构体管理错误状态，支持：
//! - 错误计数
//! - 最大重试次数
//! - 错误恢复判断

use serde::{Deserialize, Serialize};
use std::fmt;
use wasm_bindgen::prelude::*;

/// VTP 错误枚举
///
/// 定义了 VTP 库中可能出现的所有错误类型。
/// 每个变体代表一种特定的错误情况。
///
/// # 使用场景
/// - 在 Session::run_batch 中返回错误状态
/// - 在 ErrorHandler 中记录错误
/// - 前端根据错误类型显示不同的错误信息
#[wasm_bindgen]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum VtpError {
    /// 输入参数无效
    ///
    /// 当传入的参数不符合要求时返回，例如：
    /// - 种子长度不足 32 字节
    /// - 步数为 0
    InvalidInput,

    /// 内部状态无效
    ///
    /// 当内部状态不一致时返回，例如：
    /// - VDF 迭代器状态损坏
    /// - 会话状态异常
    InvalidState,

    /// 计算失败
    ///
    /// 当 VDF 或 VRF 计算过程中发生错误时返回
    ComputationFailed,

    /// 检查点保存/加载失败
    ///
    /// 当 IndexedDB 操作失败时返回
    CheckpointFailed,

    /// 会话已完成
    ///
    /// 当尝试在已完成的会话上执行操作时返回
    SessionFinished,

    /// 会话未开始
    ///
    /// 当尝试在未开始的会话上执行操作时返回
    SessionNotStarted,
}

/// 实现 Display trait，用于错误信息的格式化输出
///
/// 提供人类可读的错误描述，用于日志和前端显示
impl fmt::Display for VtpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VtpError::InvalidInput => write!(f, "Invalid input provided"),
            VtpError::InvalidState => write!(f, "Invalid state encountered"),
            VtpError::ComputationFailed => write!(f, "Computation failed"),
            VtpError::CheckpointFailed => write!(f, "Checkpoint save/load failed"),
            VtpError::SessionFinished => write!(f, "Session has already finished"),
            VtpError::SessionNotStarted => write!(f, "Session has not started yet"),
        }
    }
}

/// 实现 Error trait，使 VtpError 可以作为标准错误类型使用
impl std::error::Error for VtpError {}

/// 错误处理器
///
/// 管理错误状态，支持错误计数和重试机制。
///
/// # 字段说明
/// - `last_error`: 最近一次发生的错误
/// - `error_count`: 累计错误次数
/// - `max_retries`: 最大重试次数
///
/// # 使用示例
/// ```rust
/// use vtp_core::error::{ErrorHandler, VtpError};
///
/// let mut handler = ErrorHandler::new(3);
///
/// // 处理错误
/// let can_continue = handler.handle_error(VtpError::ComputationFailed);
/// if !can_continue {
///     println!("Too many errors, stopping...");
/// }
///
/// // 重置错误计数
/// handler.reset();
/// ```
#[derive(Debug, Clone)]
pub struct ErrorHandler {
    /// 最近一次发生的错误
    pub last_error: Option<VtpError>,

    /// 累计错误次数
    pub error_count: u32,

    /// 最大重试次数
    pub max_retries: u32,
}

/// 默认实现
///
/// 创建一个最大重试次数为 3 的 ErrorHandler
impl Default for ErrorHandler {
    fn default() -> Self {
        Self { last_error: None, error_count: 0, max_retries: 3 }
    }
}

impl ErrorHandler {
    /// 创建新的错误处理器
    ///
    /// # 参数
    /// - `max_retries`: 最大重试次数
    ///
    /// # 返回值
    /// 返回初始化的 ErrorHandler 实例
    pub fn new(max_retries: u32) -> Self {
        Self { last_error: None, error_count: 0, max_retries }
    }

    /// 处理错误
    ///
    /// 记录错误并更新错误计数。
    ///
    /// # 参数
    /// - `error`: 发生的错误
    ///
    /// # 返回值
    /// - `true`: 可以继续执行（未超过最大重试次数）
    /// - `false`: 应该停止执行（超过最大重试次数）
    ///
    /// # 注意
    /// 即使返回 true，也应该考虑适当的延迟后重试
    pub fn handle_error(&mut self, error: VtpError) -> bool {
        self.last_error = Some(error);
        self.error_count += 1;

        if self.error_count >= self.max_retries {
            return false;
        }

        true
    }

    /// 重置错误状态
    ///
    /// 清除错误记录和计数，通常在错误恢复后调用
    pub fn reset(&mut self) {
        self.last_error = None;
        self.error_count = 0;
    }

    /// 检查是否可以重试
    ///
    /// # 返回值
    /// - `true`: 可以重试（错误次数未超过限制）
    /// - `false`: 不应重试（已达到最大重试次数）
    pub fn can_retry(&self) -> bool {
        self.error_count < self.max_retries
    }
}

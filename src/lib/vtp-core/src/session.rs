//! 会话状态机模块
//!
//! 管理 VDF 挑战的完整生命周期，包括：
//! - VDF 计算迭代
//! - VRF 抽签检查
//! - 检查点管理
//! - 错误处理
//!
//! # 设计思路
//! Session 结构体封装了一次完整的 VDF 挑战。它协调 VDF 迭代器和
//! VRF 证明生成，在适当的时机进行抽签检查。
//!
//! # 工作流程
//! 1. 创建 Session，初始化 VDF 迭代器和 VRF 密钥对
//! 2. 调用 `run_batch` 执行批量 VDF 计算
//! 3. 在检查点步骤自动进行 VRF 抽签
//! 4. 根据返回的 BatchResult 处理不同情况

use crate::error::{ErrorHandler, VtpError};
use crate::vdf::VdfIterator;
use crate::vrf;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

/// 中签结果结构体
///
/// 包含中签步数和 VRF 证明
#[wasm_bindgen]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WinnerResult {
    /// 中签步数
    pub step: u64,

    /// VRF 证明
    proof: Vec<u8>,
}

#[wasm_bindgen]
impl WinnerResult {
    /// 获取 VRF 证明
    #[wasm_bindgen(getter)]
    pub fn proof(&self) -> Vec<u8> {
        self.proof.clone()
    }
}

/// 批量计算结果枚举
///
/// 表示一次批量 VDF 计算的结果状态。
/// 每次调用 `run_batch` 都会返回一个 BatchResult。
///
/// # 变体说明
/// - `Progress`: 计算进行中，返回当前步数
/// - `Winner`: 发现中签，返回 WinnerResult
/// - `Finished`: VDF 计算已完成
/// - `Error`: 发生错误
#[wasm_bindgen]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BatchResult {
    /// 计算进行中，包含当前步数
    Progress(u64),

    /// 发现中签，包含 WinnerResult
    Winner(WinnerResult),

    /// VDF 计算已完成
    Finished = "finished",

    /// 发生错误
    Error(VtpError),
}

/// 会话状态结构体
///
/// 包含会话的当前状态信息，用于前端显示和监控。
#[wasm_bindgen]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    /// 当前已完成的 VDF 步数
    pub step: u64,

    /// 总步数目标
    pub total: u64,

    /// 会话是否激活（未完成）
    pub is_active: bool,

    /// 会话是否暂停
    pub is_paused: bool,

    /// 错误发生次数
    pub error_count: u32,
}

/// VDF 挑战会话
///
/// 管理一次完整的 VDF 挑战，包括 VDF 计算、VRF 抽签和检查点。
///
/// # 字段说明
/// - `vdf`: VDF 迭代器，负责顺序哈希计算
/// - `keypair`: VRF 密钥对，用于生成和验证抽签证明
/// - `k`: 抽签间隔，每 k 步进行一次 VRF 抽签
/// - `tau`: 阈值，用于判断是否中签
/// - `checkpoint_interval`: 检查点间隔
/// - `error_handler`: 错误处理器
/// - `is_paused`: 暂停标志
///
/// # 使用示例
/// ```rust
/// use vtp_core::session::{Session, BatchResult};
///
/// let seed = [0u8; 32];
/// let tau = [0u8; 32];
/// let mut session = Session::new(&seed, 100, 0, &tau, 50);
///
/// loop {
///     match session.run_batch(50) {
///         BatchResult::Progress(step) => { /* 计算进行中 */ },
///         BatchResult::Winner(result) => { /* 发现中签 */ },
///         BatchResult::Finished => break,
///         BatchResult::Error(err) => { /* 发生错误 */ },
///     }
/// }
/// ```
#[wasm_bindgen]
pub struct Session {
    /// VDF 迭代器
    vdf: VdfIterator,

    /// VRF 密钥对
    keypair: vrf::VrfKeypair,

    /// 抽签间隔
    k: u64,

    /// 阈值（当前未使用，保留用于未来扩展）
    #[allow(dead_code)]
    tau: Vec<u8>,

    /// 检查点间隔
    checkpoint_interval: u64,

    /// 错误处理器
    error_handler: ErrorHandler,

    /// 暂停标志
    is_paused: bool,
}

#[wasm_bindgen]
impl Session {
    /// 创建新的会话
    ///
    /// # 参数
    /// - `seed`: VDF 计算的初始种子，至少 32 字节
    /// - `total`: VDF 总步数目标
    /// - `k`: 抽签间隔
    /// - `tau`: 阈值，32 字节
    /// - `checkpoint_interval`: 检查点间隔
    ///
    /// # 返回值
    /// 返回初始化的 Session 实例
    ///
    /// # Panics
    /// 如果 seed 或 tau 少于 32 字节，会 panic
    #[wasm_bindgen(constructor)]
    pub fn new(
        seed: &[u8],
        total: u64,
        k: u64,
        tau: &[u8],
        checkpoint_interval: u64,
    ) -> Self {
        let vdf = VdfIterator::new(seed, total);
        let keypair = vrf::generate_keypair();
        let error_handler = ErrorHandler::default();

        Self {
            vdf,
            keypair,
            k,
            tau: tau.to_vec(),
            checkpoint_interval,
            error_handler,
            is_paused: false,
        }
    }

    /// 获取当前会话状态
    ///
    /// # 返回值
    /// 返回 SessionState 结构体，包含当前步数、总步数等信息
    #[wasm_bindgen(getter)]
    pub fn state(&self) -> SessionState {
        SessionState {
            step: self.vdf.step(),
            total: self.vdf.total(),
            is_active: !self.vdf.is_finished(),
            is_paused: self.is_paused,
            error_count: self.error_handler.error_count,
        }
    }

    /// 获取 VRF 公钥
    ///
    /// # 返回值
    /// 返回 32 字节的公钥向量
    #[wasm_bindgen(getter)]
    pub fn public_key(&self) -> Vec<u8> {
        self.keypair.public_key()
    }

    /// 暂停会话
    ///
    /// 暂停后，`run_batch` 将返回当前进度而不执行新的计算。
    pub fn pause(&mut self) {
        self.is_paused = true;
    }

    /// 恢复会话
    ///
    /// 恢复后，`run_batch` 将继续执行 VDF 计算。
    pub fn resume(&mut self) {
        self.is_paused = false;
    }

    /// 检查会话是否暂停
    ///
    /// # 返回值
    /// - `true`: 会话已暂停
    /// - `false`: 会话正在运行
    pub fn is_paused(&self) -> bool {
        self.is_paused
    }

    /// 执行批量 VDF 计算
    ///
    /// 执行最多 `max_steps` 步 VDF 计算，并在检查点进行 VRF 抽签。
    ///
    /// # 参数
    /// - `max_steps`: 最大执行步数
    ///
    /// # 返回值
    /// 返回 BatchResult 枚举，表示计算结果
    ///
    /// # 工作流程
    /// 1. 检查会话是否已完成或暂停
    /// 2. 执行 VDF 批量计算
    /// 3. 在检查点步骤生成 VRF 证明
    /// 4. 判断是否中签
    /// 5. 返回相应的结果
    pub fn run_batch(&mut self, max_steps: u64) -> BatchResult {
        if self.vdf.is_finished() {
            return BatchResult::Error(VtpError::SessionFinished);
        }

        if self.is_paused {
            return BatchResult::Progress(self.vdf.step());
        }

        let _steps = self.vdf.run_batch(max_steps);
        let current_step = self.vdf.step();

        if self.is_checkpoint_step(current_step) {
            let message = current_step.to_be_bytes();
            let proof = vrf::prove(&self.keypair.secret_key(), &message);

            if self.should_trigger_vrf(current_step) {
                let winner = WinnerResult {
                    step: current_step,
                    proof,
                };
                return BatchResult::Winner(winner);
            }
        }

        if self.vdf.is_finished() {
            BatchResult::Finished
        } else {
            BatchResult::Progress(current_step)
        }
    }

    /// 检查是否为检查点步骤
    ///
    /// # 参数
    /// - `step`: 当前步数
    ///
    /// # 返回值
    /// - `true`: 是检查点步骤
    /// - `false`: 不是检查点步骤
    fn is_checkpoint_step(&self, step: u64) -> bool {
        step % self.checkpoint_interval == 0
    }

    /// 检查是否应该触发 VRF 抽签
    ///
    /// # 参数
    /// - `step`: 当前步数
    ///
    /// # 返回值
    /// - `true`: 应该触发 VRF
    /// - `false`: 不应该触发
    fn should_trigger_vrf(&self, step: u64) -> bool {
        if self.k == 0 {
            return false;
        }

        step % self.k == 0
    }

    /// 获取检查点数据
    ///
    /// 生成用于持久化的检查点数据，包含当前步数和 VDF 状态。
    ///
    /// # 返回值
    /// 返回序列化的检查点数据
    ///
    /// # 数据格式
    /// [8 字节步数 (大端序)] [32 字节 VDF 状态]
    pub fn get_checkpoint_data(&self) -> Vec<u8> {
        let state = self.vdf.get_state();
        let step = self.vdf.step();

        let mut data = Vec::new();
        data.extend_from_slice(&step.to_be_bytes());
        data.extend_from_slice(&state);
        data
    }

    /// 验证中签证明
    ///
    /// 验证给定的 VRF 证明是否有效。
    ///
    /// # 参数
    /// - `step`: 中签步数
    /// - `proof`: VRF 证明
    ///
    /// # 返回值
    /// - `true`: 证明有效
    /// - `false`: 证明无效
    pub fn verify_winner(&self, step: u64, proof: &[u8]) -> bool {
        let message = step.to_be_bytes();
        vrf::verify(&self.keypair.public_key(), &message, proof)
    }
}

//! VDF (Verifiable Delay Function) 模块
//!
//! 实现了基于连续 SHA256 迭代的可验证延迟函数。
//! VDF 是一种需要按顺序执行计算的函数，无法通过并行化来加速。
//!
//! # 算法说明
//! 每一步 VDF 计算都是对前一步结果的 SHA256 哈希：
//! state[i+1] = SHA256(state[i])
//!
//! # 性能目标
//! - 在 WebAssembly 环境中，单步耗时 ≤ 500ns
//! - 对应 ≥ 2M 步/秒的吞吐量

use sha2::{Digest, Sha256};
use wasm_bindgen::prelude::*;

/// VDF 迭代器，用于管理 VDF 计算的状态和进度
///
/// 该结构体封装了 VDF 计算的完整状态，包括：
/// - 当前计算状态（32 字节哈希值）
/// - 已完成的步数
/// - 总步数目标
///
/// # 使用场景
/// - 长时间运行的 VDF 计算任务
/// - 需要暂停/恢复的计算
/// - 批量处理优化
///
/// # 示例
/// ```rust
/// use vtp_core::vdf::VdfIterator;
/// let seed = [0u8; 32];
/// let mut iterator = VdfIterator::new(&seed, 1000000);
///
/// // 执行 1000 步
/// let steps = iterator.run_batch(1000);
///
/// // 检查进度
/// println!("Progress: {}/{}", iterator.step(), iterator.total());
/// ```
#[wasm_bindgen]
pub struct VdfIterator {
    /// 当前 VDF 状态，32 字节的哈希值
    state: [u8; 32],

    /// 已完成的 VDF 步数
    step: u64,

    /// 总步数目标
    total: u64,
}

#[wasm_bindgen]
impl VdfIterator {
    /// 创建新的 VDF 迭代器
    ///
    /// # 参数
    /// - `seed`: 初始种子，至少 32 字节
    /// - `total`: 总步数目标
    ///
    /// # 返回值
    /// 返回初始化的 VdfIterator 实例
    ///
    /// # Panics
    /// 如果 seed 少于 32 字节，会 panic
    #[wasm_bindgen(constructor)]
    pub fn new(seed: &[u8], total: u64) -> Self {
        let mut state = [0u8; 32];
        state.copy_from_slice(&seed[..32]);

        Self { state, step: 0, total }
    }

    /// 获取当前已完成的步数
    ///
    /// # 返回值
    /// 返回当前已完成的 VDF 迭代次数
    pub fn step(&self) -> u64 {
        self.step
    }

    /// 获取总步数目标
    ///
    /// # 返回值
    /// 返回 VDF 计算的总步数目标
    pub fn total(&self) -> u64 {
        self.total
    }

    /// 检查 VDF 计算是否已完成
    ///
    /// # 返回值
    /// - `true`: 已完成所有步数
    /// - `false`: 还有剩余步数
    pub fn is_finished(&self) -> bool {
        self.step >= self.total
    }

    /// 获取当前 VDF 状态
    ///
    /// # 返回值
    /// 返回 32 字节的当前状态向量
    ///
    /// # 注意
    /// 每次调用都会创建一个新的 Vec，频繁调用可能影响性能
    pub fn get_state(&self) -> Vec<u8> {
        self.state.to_vec()
    }

    /// 执行单步 VDF 计算
    ///
    /// # 返回值
    /// - `true`: 成功执行一步
    /// - `false`: 已完成所有步数，无法继续
    ///
    /// # 性能考虑
    /// 对于批量处理，建议使用 `run_batch` 方法以获得更好的性能
    pub fn next(&mut self) -> bool {
        if self.is_finished() {
            return false;
        }

        self.state = vdf_step(&self.state);
        self.step += 1;
        true
    }

    /// 批量执行 VDF 计算
    ///
    /// 执行最多 `max_steps` 步 VDF 计算，或者直到完成所有步数。
    /// 该方法优化了循环开销，适合长时间运行的计算任务。
    ///
    /// # 参数
    /// - `max_steps`: 最大执行步数
    ///
    /// # 返回值
    /// 返回实际执行的步数
    ///
    /// # 性能说明
    /// - 使用 `saturating_sub` 防止溢出
    /// - 使用 `min` 确保不超过剩余步数
    /// - 循环内直接操作数组，避免额外的函数调用开销
    pub fn run_batch(&mut self, max_steps: u64) -> u64 {
        let remaining = self.total.saturating_sub(self.step);
        let steps = max_steps.min(remaining);

        for _ in 0..steps {
            self.state = vdf_step(&self.state);
            self.step += 1;
        }

        steps
    }
}

/// 执行单步 VDF 计算
///
/// 对输入状态执行一次 SHA256 哈希，返回新的状态。
/// 这是 VDF 计算的核心原语。
///
/// # 参数
/// - `state`: 32 字节的输入状态
///
/// # 返回值
/// 返回 32 字节的输出状态
///
/// # 算法
/// output = SHA256(input)
///
/// # 性能
/// - 单次 SHA256 计算耗时约 200-500ns（取决于硬件）
/// - 在 WebAssembly 中可能略慢
pub fn vdf_step(state: &[u8; 32]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(state);
    let result = hasher.finalize();

    let mut output = [0u8; 32];
    output.copy_from_slice(&result);
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    /// 测试单步 VDF 计算
    ///
    /// 验证：
    /// 1. 输出与输入不同
    /// 2. 输出长度为 32 字节
    #[wasm_bindgen_test]
    fn test_vdf_step() {
        let state = [0u8; 32];
        let next_state = vdf_step(&state);

        assert_ne!(state, next_state);
        assert_eq!(next_state.len(), 32);
    }

    /// 测试 VDF 迭代器的基本功能
    ///
    /// 验证：
    /// 1. 初始状态正确
    /// 2. 单步迭代正常工作
    /// 3. 完成后正确返回 false
    #[wasm_bindgen_test]
    fn test_vdf_iterator() {
        let seed = [0u8; 32];
        let mut iter = VdfIterator::new(&seed, 100);

        assert_eq!(iter.step(), 0);
        assert_eq!(iter.total(), 100);
        assert!(!iter.is_finished());

        for i in 1..=100 {
            assert!(iter.next());
            assert_eq!(iter.step(), i);
        }

        assert!(iter.is_finished());
        assert!(!iter.next());
    }

    /// 测试批量 VDF 计算
    ///
    /// 验证：
    /// 1. 批量执行正确
    /// 2. 不超过总步数
    /// 3. 进度正确更新
    #[wasm_bindgen_test]
    fn test_vdf_batch() {
        let seed = [0u8; 32];
        let mut iter = VdfIterator::new(&seed, 1000);

        let steps = iter.run_batch(100);
        assert_eq!(steps, 100);
        assert_eq!(iter.step(), 100);

        let steps = iter.run_batch(1000);
        assert_eq!(steps, 900);
        assert_eq!(iter.step(), 1000);
        assert!(iter.is_finished());
    }

    /// 测试 VDF 的确定性
    ///
    /// 验证相同种子产生相同结果，确保计算的确定性
    #[wasm_bindgen_test]
    fn test_deterministic() {
        let seed = [0u8; 32];

        let mut iter1 = VdfIterator::new(&seed, 100);
        iter1.run_batch(100);

        let mut iter2 = VdfIterator::new(&seed, 100);
        iter2.run_batch(100);

        assert_eq!(iter1.get_state(), iter2.get_state());
    }
}

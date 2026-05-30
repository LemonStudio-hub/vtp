//! VTP Core 库 - Verifiable Time Proof 核心实现
//!
//! 本库提供了 VTP 协议的核心功能，包括：
//! - VDF (Verifiable Delay Function) 可验证延迟函数
//! - VRF (Verifiable Random Function) 可验证随机函数
//! - 会话状态机管理
//!
//! 该库设计为编译到 WebAssembly，在浏览器环境中运行。

pub mod error;
pub mod session;
pub mod utils;
pub mod vdf;
pub mod vrf;

use wasm_bindgen::prelude::*;

/// 初始化 panic hook，用于在 WebAssembly 中捕获和显示 panic 信息
///
/// 这个函数在模块加载时自动调用，设置了 `console_error_panic_hook`
/// 以便在发生 panic 时将错误信息输出到浏览器控制台。
///
/// # 注意
/// - 仅在启用 `console_error_panic_hook` feature 时生效
/// - 该函数使用 `set_once()` 确保只初始化一次
#[wasm_bindgen(start)]
pub fn init_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

/// 执行单个 VDF 计算的便捷函数
///
/// 该函数接受一个种子和步数，执行指定次数的 VDF 迭代，
/// 返回最终的状态。主要用于性能基准测试和快速验证。
///
/// # 参数
/// - `seed`: 32 字节的种子，作为 VDF 计算的初始状态
/// - `steps`: 要执行的 VDF 迭代次数
///
/// # 返回值
/// 返回 32 字节的最终状态
///
/// # 注意
/// - 种子必须至少 32 字节，否则会 panic
/// - 该函数是同步的，长时间运行会阻塞调用线程
///
/// # 示例
/// ```rust
/// use vtp_core::run_single_vdf;
/// let seed = [0u8; 32];
/// let result = run_single_vdf(&seed, 1000);
/// assert_eq!(result.len(), 32);
/// ```
#[wasm_bindgen]
pub fn run_single_vdf(seed: &[u8], steps: u32) -> Vec<u8> {
    let mut state = [0u8; 32];
    state.copy_from_slice(&seed[..32]);

    for _ in 0..steps {
        state = vdf::vdf_step(&state);
    }

    state.to_vec()
}

/// 获取 VDF 单步计算的复杂度值
///
/// 返回值表示单次 VDF 迭代的相对计算成本。
/// 当前实现返回 1，表示单次 SHA256 迭代。
///
/// # 返回值
/// 返回 VDF 单步计算复杂度值
#[wasm_bindgen]
pub fn vdf_step_count() -> u32 {
    1
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    /// 测试 run_single_vdf 函数的基本功能
    ///
    /// 验证：
    /// 1. 函数能正常执行
    /// 2. 返回结果长度为 32 字节
    #[wasm_bindgen_test]
    fn test_run_single_vdf() {
        let seed = [0u8; 32];
        let result = run_single_vdf(&seed, 100);
        assert_eq!(result.len(), 32);
    }
}

use crate::trace::models::{GroundedAnswer, GroundedQaContext};

/// Grounded Q&A Provider 错误
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GroundedQaError {
    /// 问题为空
    EmptyQuestion,
    /// 当前上下文无法回答且不允许伪造 citation
    UnknownAnswerRequired,
    /// Provider 生成失败
    GenerationFailed(String),
}

impl std::fmt::Display for GroundedQaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GroundedQaError::EmptyQuestion => write!(f, "问题不能为空"),
            GroundedQaError::UnknownAnswerRequired => {
                write!(f, "当前证据不足以回答问题，必须返回 unknown")
            }
            GroundedQaError::GenerationFailed(msg) => write!(f, "生成失败: {}", msg),
        }
    }
}

impl std::error::Error for GroundedQaError {}

/// Grounded Q&A Provider trait。
///
/// 接收 `GroundedQaContext`，返回 `GroundedAnswer` 或 `GroundedQaError`。
/// 所有实现必须遵守：非 unknown 回答必须包含至少一个有效 citation，unknown 回答不得伪造 citation。
pub trait GroundedQaProvider: Send + Sync {
    fn generate_answer(
        &self,
        context: &GroundedQaContext,
    ) -> Result<GroundedAnswer, GroundedQaError>;
}

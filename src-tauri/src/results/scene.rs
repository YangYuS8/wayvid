use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SceneCompatibilityIssue {
    pub code: String,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SceneRuntimeSummary {
    pub layer_count: usize,
    pub effect_count: usize,
    pub pass_count: usize,
    pub animated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SceneAssessment {
    /// Asset validation is independent from GPU and desktop acceptance.
    pub runnable: bool,
    pub summary: Option<SceneRuntimeSummary>,
    pub issues: Vec<SceneCompatibilityIssue>,
}

impl SceneAssessment {
    pub fn blocked(code: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            runnable: false,
            summary: None,
            issues: vec![SceneCompatibilityIssue {
                code: code.into(),
                detail: detail.into(),
            }],
        }
    }
}

use serde::{Deserialize, Serialize};

use crate::models::CompatibilityBadge;
use crate::policies::shared::compatibility_policy::{CompatibilityDecision, CompatibilityReason};

pub use crate::policies::shared::compatibility_policy::CompatibilityNextStep;

pub type CompatibilityAssessment = CompatibilityDecision;
pub type CompatibilityReasonCode = CompatibilityReason;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompatibilitySummary {
    pub badge: CompatibilityBadge,
    pub reason_code: CompatibilityReasonCode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompatibilityExplanation {
    pub badge: CompatibilityBadge,
    pub reason_code: CompatibilityReasonCode,
    pub headline: String,
    pub detail: String,
    pub next_step: CompatibilityNextStep,
}

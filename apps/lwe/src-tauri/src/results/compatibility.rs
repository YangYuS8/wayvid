use serde::{Deserialize, Serialize};

use crate::models::CompatibilityBadge;
use crate::policies::shared::compatibility_policy::{CompatibilityLevel, CompatibilityReason};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompatibilityNextStep {
    None,
    OpenInSteam,
    ResyncWorkshopItem,
    WaitForFutureSupport,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompatibilityAssessment {
    pub level: CompatibilityLevel,
    pub reason: CompatibilityReason,
    pub next_step: CompatibilityNextStep,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompatibilitySummary {
    pub badge: CompatibilityBadge,
    pub reason_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompatibilityExplanation {
    pub badge: CompatibilityBadge,
    pub reason_code: String,
    pub headline: String,
    pub detail: String,
    pub next_step: CompatibilityNextStep,
}

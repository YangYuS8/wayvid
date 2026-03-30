use crate::models::{CompatibilityBadge, CompatibilityExplanationModel, CompatibilitySummaryModel};
use crate::policies::shared::compatibility_policy::{CompatibilityLevel, CompatibilityReason};
use crate::results::compatibility::CompatibilityAssessment;

fn badge(level: CompatibilityLevel) -> CompatibilityBadge {
    match level {
        CompatibilityLevel::FullySupported => CompatibilityBadge::FullySupported,
        CompatibilityLevel::PartiallySupported => CompatibilityBadge::PartiallySupported,
        CompatibilityLevel::Unsupported => CompatibilityBadge::Unsupported,
    }
}

fn reason_code(reason: CompatibilityReason) -> String {
    match reason {
        CompatibilityReason::ReadyForLibrary => "ready_for_library".to_string(),
        CompatibilityReason::MissingProjectMetadata => "missing_project_metadata".to_string(),
        CompatibilityReason::MissingPrimaryAsset => "missing_primary_asset".to_string(),
        CompatibilityReason::UnsupportedWebItem => "unsupported_web_item".to_string(),
        CompatibilityReason::UnsupportedProjectType => "unsupported_project_type".to_string(),
    }
}

pub fn compatibility_summary(assessment: &CompatibilityAssessment) -> CompatibilitySummaryModel {
    CompatibilitySummaryModel {
        badge: badge(assessment.level),
        reason_code: reason_code(assessment.reason),
    }
}

pub fn compatibility_explanation(
    assessment: &CompatibilityAssessment,
) -> CompatibilityExplanationModel {
    let (headline, detail) = match assessment.reason {
        CompatibilityReason::ReadyForLibrary => (
            "Ready to use".to_string(),
            "This item is synchronized locally and available for Library and desktop use."
                .to_string(),
        ),
        CompatibilityReason::MissingProjectMetadata => (
            "Missing project metadata".to_string(),
            "LWE found the local Workshop folder, but the project metadata is missing or unreadable."
                .to_string(),
        ),
        CompatibilityReason::MissingPrimaryAsset => (
            "Missing primary asset".to_string(),
            "The project metadata exists, but the primary video or scene asset is missing from the local Workshop item."
                .to_string(),
        ),
        CompatibilityReason::UnsupportedWebItem => (
            "Web item not in first release".to_string(),
            "Web Workshop items are recognized, but LWE first-release support is currently limited to video and scene wallpapers."
                .to_string(),
        ),
        CompatibilityReason::UnsupportedProjectType => (
            "Project type not supported".to_string(),
            "This Workshop item uses a project type outside the current first-release import surface."
                .to_string(),
        ),
    };

    CompatibilityExplanationModel {
        badge: badge(assessment.level),
        reason_code: reason_code(assessment.reason),
        headline,
        detail,
        next_step: assessment.next_step,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::policies::shared::compatibility_policy::{CompatibilityLevel, CompatibilityReason};
    use crate::results::compatibility::{CompatibilityAssessment, CompatibilityNextStep};

    #[test]
    fn compatibility_assembly_turns_assessment_into_summary_and_explanation() {
        let assessment = CompatibilityAssessment {
            level: CompatibilityLevel::Unsupported,
            reason: CompatibilityReason::UnsupportedWebItem,
            next_step: CompatibilityNextStep::WaitForFutureSupport,
        };

        let summary = compatibility_summary(&assessment);
        let explanation = compatibility_explanation(&assessment);

        assert_eq!(summary.reason_code, "unsupported_web_item");
        assert_eq!(
            explanation.next_step,
            CompatibilityNextStep::WaitForFutureSupport
        );
    }
}

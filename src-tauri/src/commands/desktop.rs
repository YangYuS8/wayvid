use crate::action_outcome::ActionOutcome;
use crate::assembly::action_outcome::assemble_desktop_apply_outcome;
use crate::assembly::desktop_page::assemble_desktop_page;
use crate::commands::background::run_desktop_blocking;
use crate::models::DesktopPageSnapshot;
use crate::services::desktop_service::DesktopService;

#[tauri::command]
pub async fn load_desktop_page() -> Result<DesktopPageSnapshot, String> {
    run_desktop_blocking(|| DesktopService::load_page().map(assemble_desktop_page)).await
}

#[tauri::command]
pub async fn apply_library_item_to_monitor(
    monitor_id: String,
    item_id: String,
) -> Result<ActionOutcome<()>, String> {
    run_desktop_blocking(move || {
        Ok(assemble_desktop_apply_outcome(
            DesktopService::apply_to_monitor(&monitor_id, &item_id)?,
        ))
    })
    .await
}

#[tauri::command]
pub async fn clear_library_item_from_monitor(
    monitor_id: String,
) -> Result<ActionOutcome<()>, String> {
    run_desktop_blocking(move || {
        Ok(assemble_desktop_apply_outcome(
            DesktopService::clear_monitor(&monitor_id)?,
        ))
    })
    .await
}

#[tauri::command]
pub async fn load_scene_properties(
    monitor_id: String,
    item_id: String,
) -> Result<crate::models::ScenePropertiesSnapshot, String> {
    run_desktop_blocking(move || {
        crate::services::scene_properties_service::ScenePropertiesService::load(
            &monitor_id,
            &item_id,
        )
    })
    .await
}

#[tauri::command]
pub async fn choose_scene_image(
    app: tauri::AppHandle,
    item_id: String,
    property_name: String,
) -> Result<Option<crate::services::scene_resource_service::ImportedSceneImage>, String> {
    run_desktop_blocking(move || {
        crate::services::scene_resource_service::SceneResourceService::choose(
            &app,
            &item_id,
            &property_name,
        )
    })
    .await
}

#[tauri::command]
pub async fn update_scene_properties(
    monitor_id: String,
    item_id: String,
    values: crate::results::session_persistence::ScenePropertyValues,
) -> Result<ActionOutcome<()>, String> {
    run_desktop_blocking(move || {
        crate::services::scene_properties_service::ScenePropertiesService::validate_values_shape(
            &values,
        )?;
        Ok(assemble_desktop_apply_outcome(
            DesktopService::update_scene_properties(&monitor_id, &item_id, &values)?,
        ))
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::results::desktop::DesktopApplyResult;

    #[test]
    fn desktop_apply_flow_command_assembly_returns_failure_outcome_without_real_desktop() {
        let outcome =
            assemble_desktop_apply_outcome(DesktopApplyResult::MonitorDiscoveryUnavailable {
                reason: "niri is unavailable".to_string(),
            });

        assert!(matches!(
            outcome,
            ActionOutcome {
                ok: false,
                message: Some(_),
                ..
            }
        ));
    }
}

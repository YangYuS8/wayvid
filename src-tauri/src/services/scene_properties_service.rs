use std::collections::BTreeMap;

use crate::models::ScenePropertiesSnapshot;
use crate::services::{
    desktop_persistence_service::DesktopPersistenceService, library_service::LibraryService,
    scene_service::SceneService,
};
use lwe_engine::scene::{apply_property_overrides, property_definitions};
use lwe_library::WorkshopProjectType;
use lwe_scene::SceneAssets;

pub struct ScenePropertiesService;

impl ScenePropertiesService {
    pub fn load(monitor_id: &str, item_id: &str) -> Result<ScenePropertiesSnapshot, String> {
        let entry = LibraryService::inspect_item(item_id)?;
        if entry.entry.project_type != WorkshopProjectType::Scene {
            return Err("Properties are available only for Scene wallpapers".into());
        }
        let values = DesktopPersistenceService::load_scene_properties(monitor_id, item_id)?;
        let root = SceneService::assets_root(&entry.entry.project_dir)?;
        let assets = SceneAssets::open(&entry.entry.project_dir, root.as_deref())
            .map_err(|error| error.to_string())?;
        let bytes = assets
            .read("project.json", 4 * 1024 * 1024)
            .map_err(|error| error.to_string())?;
        let mut project = serde_json::from_slice(&bytes)
            .map_err(|_| "Invalid Scene project metadata".to_string())?;
        if !values.is_empty() {
            apply_property_overrides(&mut project, &values).map_err(|error| error.to_string())?;
        }
        let properties = property_definitions(&project).map_err(|error| error.to_string())?;
        Ok(ScenePropertiesSnapshot {
            item_id: item_id.into(),
            monitor_id: monitor_id.into(),
            properties,
        })
    }

    pub fn validate_values_shape(
        values: &BTreeMap<String, serde_json::Value>,
    ) -> Result<(), String> {
        if values.len() > 256
            || values
                .keys()
                .any(|name| name.is_empty() || name.len() > 256)
        {
            return Err("Scene property selection exceeds the supported limits".into());
        }
        Ok(())
    }
}

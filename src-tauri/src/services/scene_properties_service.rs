use std::collections::BTreeMap;

use crate::models::ScenePropertiesSnapshot;
use crate::services::{
    desktop_persistence_service::DesktopPersistenceService, library_service::LibraryService,
    scene_service::SceneService,
};
use lwe_engine::scene::{
    PropertyValue, apply_property_overrides, property_definitions, property_groups,
};
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
        let project = serde_json::from_slice(&bytes)
            .map_err(|_| "Invalid Scene project metadata".to_string())?;
        snapshot_from_project(project, &values, monitor_id, item_id)
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

fn snapshot_from_project(
    mut project: serde_json::Value,
    values: &BTreeMap<String, serde_json::Value>,
    monitor_id: &str,
    item_id: &str,
) -> Result<ScenePropertiesSnapshot, String> {
    let (scalars, images) =
        super::scene_resource_service::SceneResourceService::split_choices(&project, values)?;
    if !scalars.is_empty() {
        apply_property_overrides(&mut project, &scalars).map_err(|error| error.to_string())?;
    }
    let mut properties = property_definitions(&project).map_err(|error| error.to_string())?;
    for property in &mut properties {
        if images.contains_key(&property.name) {
            property.value =
                PropertyValue::Texture(values[&property.name].as_str().map(str::to_owned));
        }
    }
    let groups = property_groups(&project).map_err(|error| error.to_string())?;
    Ok(ScenePropertiesSnapshot {
        item_id: item_id.into(),
        monitor_id: monitor_id.into(),
        properties,
        groups,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn snapshots_serialize_sections_separately_from_saved_value_controls() {
        let project = json!({"general":{"properties":{
            "before":{"type":"bool","value":true,"order":100},
            "glitch1":{"type":"group","index":1,"order":101,"text":"ELEMENTS","value":""},
            "enabled":{"type":"bool","value":false,"order":102},
            "crt":{"type":"group","index":7,"order":107,"text":"CRT","value":""},
            "intensity":{"type":"slider","value":0.25,"min":0,"max":1,"order":108}
        }}});
        let values = BTreeMap::from([("enabled".into(), json!(true))]);
        let snapshot = snapshot_from_project(project.clone(), &values, "DISPLAY", "scene").unwrap();
        let encoded = serde_json::to_value(snapshot).unwrap();
        assert_eq!(encoded["itemId"], "scene");
        assert_eq!(encoded["monitorId"], "DISPLAY");
        assert_eq!(encoded["properties"].as_array().unwrap().len(), 3);
        assert_eq!(encoded["properties"][1]["name"], "enabled");
        assert_eq!(encoded["properties"][1]["value"], true);
        assert_eq!(
            encoded["groups"],
            json!([
                {"name":"glitch1","label":"ELEMENTS","order":101.0,"members":["enabled"]},
                {"name":"crt","label":"CRT","order":107.0,"members":["intensity"]}
            ])
        );
        assert!(
            encoded["groups"]
                .as_array()
                .unwrap()
                .iter()
                .all(|group| group.get("value").is_none())
        );
        assert!(
            snapshot_from_project(
                project,
                &BTreeMap::from([("glitch1".into(), json!(""))]),
                "DISPLAY",
                "scene"
            )
            .is_err()
        );
    }

    #[test]
    fn ungrouped_and_empty_snapshots_keep_an_explicit_empty_groups_array() {
        for project in [
            json!({}),
            json!({"general":{"properties":{"enabled":{"type":"bool","value":true}}}}),
        ] {
            let encoded = serde_json::to_value(
                snapshot_from_project(project, &BTreeMap::new(), "DISPLAY", "scene").unwrap(),
            )
            .unwrap();
            assert_eq!(encoded["groups"], json!([]));
        }
        let project = json!({"general":{"properties":{
            "empty":{"type":"group","text":""},
            "tail":{"type":"group","text":""}
        }}});
        let encoded = serde_json::to_value(
            snapshot_from_project(project, &BTreeMap::new(), "DISPLAY", "scene").unwrap(),
        )
        .unwrap();
        assert_eq!(encoded["properties"], json!([]));
        assert_eq!(
            encoded["groups"],
            json!([
                {"name":"empty","label":"","order":null,"members":[]},
                {"name":"tail","label":"","order":null,"members":[]}
            ])
        );
    }
}

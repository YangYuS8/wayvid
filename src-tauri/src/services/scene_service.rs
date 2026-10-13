use crate::results::scene::{SceneAssessment, SceneRuntimeSummary};
use lwe_engine::scene::{PreparedScene, unsupported_feature_code};
use lwe_library::SteamLibrary;
use lwe_scene::SceneAssetError;
use std::collections::{BTreeMap, hash_map::DefaultHasher};
use std::fs;
use std::hash::{Hash, Hasher};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

const ASSESSOR_SCHEMA: u32 = 10;
const MAX_CACHED_ASSESSMENTS: usize = 64;
const MAX_FINGERPRINT_ENTRIES: usize = 32_768;
const MAX_FINGERPRINT_DEPTH: usize = 16;
const MAX_TEXT_FINGERPRINT_BYTES: u64 = 8 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct CacheKey {
    project_dir: PathBuf,
    assets_root: Option<PathBuf>,
    schema: u32,
}

struct CachedAssessment {
    fingerprint: u64,
    last_access: u64,
    assessment: SceneAssessment,
}

#[derive(Default)]
struct AssessmentCache {
    clock: u64,
    entries: BTreeMap<CacheKey, CachedAssessment>,
}

fn assessment_cache() -> &'static Mutex<AssessmentCache> {
    static CACHE: OnceLock<Mutex<AssessmentCache>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(AssessmentCache::default()))
}

pub struct SceneService;

impl SceneService {
    /// Pure CPU preparation. Call on the existing background command path.
    /// An apply always prepares again: cached catalog assessments never
    /// substitute for validation of the resources about to be rendered.
    pub fn prepare_project(project_dir: &Path) -> Result<PreparedScene, String> {
        Self::prepare_project_with_properties(project_dir, &BTreeMap::new())
    }

    pub fn prepare_project_with_properties(
        project_dir: &Path,
        values: &BTreeMap<String, serde_json::Value>,
    ) -> Result<PreparedScene, String> {
        let assets_root = Self::assets_root(project_dir)?;
        PreparedScene::prepare_with_properties(project_dir, assets_root.as_deref(), values)
            .map_err(|error| format!("{error:#}"))
    }

    pub fn assess_project(project_dir: &Path) -> SceneAssessment {
        match Self::assets_root(project_dir) {
            Ok(assets_root) => {
                Self::assess_project_with_assets(project_dir, assets_root.as_deref())
            }
            Err(error) => SceneAssessment::blocked("resource_access", error),
        }
    }

    pub fn assess_project_with_assets(
        project_dir: &Path,
        assets_root: Option<&Path>,
    ) -> SceneAssessment {
        let key = CacheKey {
            project_dir: project_dir.to_path_buf(),
            assets_root: assets_root.map(Path::to_path_buf),
            schema: ASSESSOR_SCHEMA,
        };
        let fingerprint = resource_fingerprint(project_dir, assets_root).ok();
        if let Some(fingerprint) = fingerprint
            && let Ok(mut cache) = assessment_cache().lock()
        {
            cache.clock = cache.clock.wrapping_add(1);
            let now = cache.clock;
            if let Some(cached) = cache.entries.get_mut(&key)
                && cached.fingerprint == fingerprint
            {
                cached.last_access = now;
                return cached.assessment.clone();
            }
        }

        let assessment = match PreparedScene::prepare(project_dir, assets_root) {
            Ok(prepared) => {
                let summary = prepared.summary();
                SceneAssessment {
                    runnable: true,
                    summary: Some(SceneRuntimeSummary {
                        layer_count: summary.layer_count,
                        effect_count: summary.effect_count,
                        pass_count: summary.pass_count,
                        animated: summary.animated,
                    }),
                    issues: Vec::new(),
                }
            }
            Err(error) => {
                let detail = format!("{error:#}");
                let code = match unsupported_feature_code(&error) {
                    Some("scenescript") => "unsupported_script",
                    Some("property_binding") => "unsupported_properties",
                    Some("particle") => "unsupported_particles",
                    Some("audio") => "unsupported_audio",
                    Some("text") => "unsupported_text",
                    Some("model") => "unsupported_model",
                    Some("puppet") => "unsupported_puppet",
                    Some("hdr") => "unsupported_hdr",
                    Some("camera") => "unsupported_camera",
                    Some("hierarchy") => "unsupported_hierarchy",
                    Some("shader") => "unsupported_shader",
                    Some(_) => "unsupported_scene",
                    None => match error.downcast_ref::<SceneAssetError>() {
                        Some(SceneAssetError::NotFound { .. }) => "missing_resource",
                        Some(SceneAssetError::UnsafePath { .. }) => "unsafe_resource",
                        Some(SceneAssetError::InvalidPackage { .. }) => "invalid_package",
                        Some(SceneAssetError::InvalidTexture { .. }) => "invalid_texture",
                        Some(SceneAssetError::LimitExceeded { .. }) => "resource_limit",
                        Some(SceneAssetError::Io { .. }) => "resource_access",
                        _ => issue_code(&detail),
                    },
                };
                SceneAssessment::blocked(code, detail)
            }
        };

        if let Some(fingerprint) = fingerprint
            && let Ok(mut cache) = assessment_cache().lock()
        {
            cache.clock = cache.clock.wrapping_add(1);
            let last_access = cache.clock;
            if cache.entries.len() >= MAX_CACHED_ASSESSMENTS
                && !cache.entries.contains_key(&key)
                && let Some(oldest) = cache
                    .entries
                    .iter()
                    .min_by_key(|(_, value)| value.last_access)
                    .map(|(key, _)| key.clone())
            {
                cache.entries.remove(&oldest);
            }
            cache.entries.insert(
                key,
                CachedAssessment {
                    fingerprint,
                    last_access,
                    assessment: assessment.clone(),
                },
            );
        }
        assessment
    }

    pub fn assets_root(project_dir: &Path) -> Result<Option<PathBuf>, String> {
        let mut candidates = Vec::new();
        for ancestor in project_dir.ancestors() {
            if ancestor.file_name().is_some_and(|name| name == "steamapps") {
                candidates.push(ancestor.join("common/wallpaper_engine/assets"));
            }
        }
        if let Some(path) = existing_assets_root(&candidates)? {
            return Ok(Some(path));
        }
        let steam = SteamLibrary::discover_optional()
            .map_err(|error| format!("Cannot discover Wallpaper Engine assets: {error}"))?;
        if let Some(steam) = steam {
            candidates.extend(
                steam
                    .all_libraries()
                    .into_iter()
                    .map(|library| library.join("steamapps/common/wallpaper_engine/assets")),
            );
        }
        existing_assets_root(&candidates)
    }
}

fn existing_assets_root(candidates: &[PathBuf]) -> Result<Option<PathBuf>, String> {
    for path in candidates {
        match fs::metadata(path) {
            Ok(metadata) if metadata.is_dir() => return Ok(Some(path.clone())),
            Ok(_) => {
                return Err(format!(
                    "Wallpaper Engine assets path is not a directory: {}",
                    path.display()
                ));
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => {
                return Err(format!(
                    "Cannot inspect Wallpaper Engine assets {}: {error}",
                    path.display()
                ));
            }
        }
    }
    Ok(None)
}

// These labels classify a failed authoritative preparation for presentation.
// They never decide whether the renderer can execute a feature.
fn issue_code(detail: &str) -> &'static str {
    let detail = detail.to_ascii_lowercase();
    if !detail.contains("unsupported") {
        return "validation_failed";
    }
    for (needle, code) in [
        ("texs", "unsupported_animated_texture"),
        ("animated texture", "unsupported_animated_texture"),
        ("video", "unsupported_video_texture"),
        ("scenescript", "unsupported_script"),
        ("script", "unsupported_script"),
        ("particle", "unsupported_particles"),
        ("sound", "unsupported_audio"),
        ("audio", "unsupported_audio"),
        ("text", "unsupported_text"),
        ("puppet", "unsupported_puppet"),
        ("skinning", "unsupported_puppet"),
        ("3d", "unsupported_model"),
        ("model", "unsupported_model"),
        ("hdr", "unsupported_hdr"),
        ("bloom", "unsupported_hdr"),
        ("camera", "unsupported_camera"),
        ("parallax", "unsupported_camera"),
        ("hierarchy", "unsupported_hierarchy"),
        ("shader", "unsupported_shader"),
        ("effect", "unsupported_shader"),
    ] {
        if if needle.contains(' ') {
            detail.contains(needle)
        } else {
            detail
                .split(|character: char| !character.is_ascii_alphanumeric())
                .any(|word| word == needle)
        } {
            return code;
        }
    }
    "unsupported_scene"
}

fn resource_fingerprint(project_dir: &Path, assets_root: Option<&Path>) -> Result<u64, String> {
    let mut hasher = DefaultHasher::new();
    ASSESSOR_SCHEMA.hash(&mut hasher);
    let mut entries = 0;
    fingerprint_tree(project_dir, &mut hasher, &mut entries, 0)?;
    if let Some(assets_root) = assets_root {
        fingerprint_tree(assets_root, &mut hasher, &mut entries, 0)?;
    }
    Ok(hasher.finish())
}

fn fingerprint_tree(
    path: &Path,
    hasher: &mut DefaultHasher,
    entries: &mut usize,
    depth: usize,
) -> Result<(), String> {
    *entries += 1;
    if *entries > MAX_FINGERPRINT_ENTRIES || depth > MAX_FINGERPRINT_DEPTH {
        return Err("Scene assessment fingerprint exceeds traversal limit".to_string());
    }
    path.hash(hasher);
    let metadata = fs::symlink_metadata(path).map_err(|error| error.to_string())?;
    metadata.len().hash(hasher);
    metadata.modified().ok().hash(hasher);
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        metadata.dev().hash(hasher);
        metadata.ino().hash(hasher);
        metadata.ctime().hash(hasher);
        metadata.ctime_nsec().hash(hasher);
    }
    if metadata.file_type().is_symlink() {
        fs::read_link(path)
            .map_err(|error| error.to_string())?
            .hash(hasher);
        if let Ok(target) = fs::metadata(path) {
            target.len().hash(hasher);
            target.modified().ok().hash(hasher);
        }
    } else if metadata.is_dir() {
        let mut children = fs::read_dir(path)
            .map_err(|error| error.to_string())?
            .map(|entry| entry.map(|entry| entry.path()))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| error.to_string())?;
        children.sort();
        for child in children {
            fingerprint_tree(&child, hasher, entries, depth + 1)?;
        }
    } else if metadata.is_file()
        && metadata.len() <= MAX_TEXT_FINGERPRINT_BYTES
        && path
            .extension()
            .and_then(|value| value.to_str())
            .is_some_and(|extension| {
                matches!(extension, "json" | "vert" | "frag" | "geom" | "glsl")
            })
    {
        let mut bytes = Vec::new();
        fs::File::open(path)
            .map_err(|error| error.to_string())?
            .take(MAX_TEXT_FINGERPRINT_BYTES + 1)
            .read_to_end(&mut bytes)
            .map_err(|error| error.to_string())?;
        bytes.hash(hasher);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scene_fixture() -> tempfile::TempDir {
        let temp = tempfile::tempdir().unwrap();
        fs::create_dir_all(temp.path().join("models")).unwrap();
        fs::create_dir_all(temp.path().join("materials")).unwrap();
        fs::write(
            temp.path().join("project.json"),
            r#"{"type":"scene","file":"scene.json"}"#,
        )
        .unwrap();
        fs::write(temp.path().join("scene.json"), r#"{"general":{"orthogonalprojection":{"width":4,"height":4}},"objects":[{"image":"models/picture.json","origin":"2 2 0","size":"4 4"}]}"#).unwrap();
        fs::write(
            temp.path().join("models/picture.json"),
            r#"{"material":"materials/picture.json"}"#,
        )
        .unwrap();
        fs::write(temp.path().join("materials/picture.json"), r#"{"passes":[{"shader":"genericimage4","textures":["picture"],"blending":"translucent"}]}"#).unwrap();
        let mut texture = b"TEXV0005\0TEXI0001\0".to_vec();
        for value in [0_u32, 0, 1, 1, 1, 1, 0] {
            texture.extend(value.to_le_bytes());
        }
        texture.extend(b"TEXB0001\0");
        for value in [1_u32, 1, 1, 1, 4] {
            texture.extend(value.to_le_bytes());
        }
        texture.extend([255, 0, 0, 255]);
        fs::write(temp.path().join("materials/picture.tex"), texture).unwrap();
        temp
    }

    #[test]
    fn invalid_scene_is_blocked_without_losing_resource_error() {
        let temp = tempfile::tempdir().unwrap();
        fs::write(
            temp.path().join("project.json"),
            r#"{"type":"scene","file":"scene.json"}"#,
        )
        .unwrap();
        fs::write(temp.path().join("scene.json"), b"{invalid}").unwrap();
        let assessment = SceneService::assess_project_with_assets(temp.path(), None);
        assert!(!assessment.runnable);
        assert!(!assessment.issues[0].detail.is_empty());
        assert!(assessment.summary.is_none());
    }

    #[test]
    fn eligible_2d_scene_uses_renderer_validation_and_cache_invalidation() {
        let temp = scene_fixture();
        let assessment = SceneService::assess_project_with_assets(temp.path(), None);
        assert!(assessment.runnable, "{:?}", assessment.issues);
        assert_eq!(assessment.summary.as_ref().unwrap().layer_count, 1);
        assert_eq!(assessment.summary.as_ref().unwrap().pass_count, 1);
        assert!(assessment.issues.is_empty());
        assert_eq!(
            SceneService::assess_project_with_assets(temp.path(), None),
            assessment
        );

        fs::remove_file(temp.path().join("materials/picture.tex")).unwrap();
        let assessment = SceneService::assess_project_with_assets(temp.path(), None);
        assert!(!assessment.runnable);
        assert_eq!(assessment.issues[0].code, "missing_resource");
    }

    #[test]
    fn unknown_layer_and_script_are_blocked_with_feature_codes() {
        for (object, code) in [
            (
                r#"{"particle":"particles/rain.json"}"#,
                "unsupported_particles",
            ),
            (r#"{"future":"unknown"}"#, "unsupported_scene"),
            (
                r#"{"image":"models/picture.json","parent":1}"#,
                "unsupported_hierarchy",
            ),
            (
                r#"{"image":"models/picture.json","alpha":{"script":"export function update(){return 1;}"}}"#,
                "unsupported_script",
            ),
            (
                r#"{"image":"models/picture.json","visible":{"value":true,"script":"export function update(){return engine.unsupportedHostCall();}"}}"#,
                "unsupported_script",
            ),
            (
                r#"{"image":"models/picture.json","alpha":{"user":"opacity","value":0.5}}"#,
                "unsupported_properties",
            ),
        ] {
            let temp = scene_fixture();
            fs::create_dir_all(temp.path().join("particles")).unwrap();
            fs::write(
                temp.path().join("particles/rain.json"),
                br#"{"unknown_behavior":true}"#,
            )
            .unwrap();
            fs::write(temp.path().join("scene.json"), format!(r#"{{"general":{{"orthogonalprojection":{{"width":4,"height":4}}}},"objects":[{object}]}}"#)).unwrap();
            let assessment = SceneService::assess_project_with_assets(temp.path(), None);
            assert!(!assessment.runnable);
            assert_eq!(assessment.issues[0].code, code);
        }
    }

    #[test]
    fn fingerprint_changes_after_asset_replacement_and_removal() {
        let temp = tempfile::tempdir().unwrap();
        let asset = temp.path().join("scene.json");
        fs::write(&asset, b"{}").unwrap();
        let first = resource_fingerprint(temp.path(), None).unwrap();
        fs::write(&asset, b"{\"objects\": []}").unwrap();
        let second = resource_fingerprint(temp.path(), None).unwrap();
        fs::remove_file(asset).unwrap();
        let third = resource_fingerprint(temp.path(), None).unwrap();
        assert_ne!(first, second);
        assert_ne!(second, third);
    }

    #[test]
    fn installed_assets_are_found_from_the_items_steam_library() {
        let temp = tempfile::tempdir().unwrap();
        let assets = temp.path().join("steamapps/common/wallpaper_engine/assets");
        fs::create_dir_all(&assets).unwrap();
        let project = temp.path().join("steamapps/workshop/content/431960/1");
        assert_eq!(SceneService::assets_root(&project).unwrap(), Some(assets));
    }

    #[test]
    fn unsupported_feature_labels_do_not_hide_unknown_features() {
        assert_eq!(
            issue_code("unsupported_scene: audio-reactive shader"),
            "unsupported_audio"
        );
        assert_eq!(
            issue_code("unsupported scene resource feature: TEXS animation"),
            "unsupported_animated_texture"
        );
        assert_eq!(
            issue_code("unsupported_scene: future object"),
            "unsupported_scene"
        );
        assert_eq!(issue_code("invalid scene package"), "validation_failed");
    }
}

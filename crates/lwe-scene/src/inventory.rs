//! Bounded dependency discovery. This is an observation of declarations and
//! resource framing, not a renderer eligibility or correctness assessment.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::{
    ResourceOrigin, Result, SceneAssetError, SceneAssets, Texture, TextureInspectionBudget, limit,
};

/// All limits cover the complete traversal, not an individual dependency only.
/// A lower caller limit produces a partial report with an explicit finding.
#[derive(Debug, Clone, Serialize)]
pub struct InventoryLimits {
    pub max_resources: usize,
    pub max_documents: usize,
    pub max_document_bytes: usize,
    pub max_total_document_bytes: usize,
    pub max_shader_bytes: usize,
    pub max_total_shader_bytes: usize,
    pub max_nodes: usize,
    pub max_depth: usize,
    pub max_observations: usize,
    /// Dynamic identifiers/locations. Fixed schema strings and bounded hashes
    /// are separately bounded by resource/document/observation counts.
    pub max_report_string_bytes: usize,
    pub max_texture_mips: usize,
    pub max_animation_frames: usize,
}

impl Default for InventoryLimits {
    fn default() -> Self {
        Self {
            max_resources: 8192,
            max_documents: 2048,
            max_document_bytes: 8 * 1024 * 1024,
            max_total_document_bytes: 32 * 1024 * 1024,
            max_shader_bytes: 1024 * 1024,
            max_total_shader_bytes: 16 * 1024 * 1024,
            max_nodes: 1_000_000,
            max_depth: 64,
            max_observations: 50_000,
            max_report_string_bytes: 16 * 1024 * 1024,
            max_texture_mips: 16_384,
            max_animation_frames: 16_384,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct FeatureLocation {
    pub resource: String,
    /// Structural location: known field names are normalized, unknown names
    /// hashed. This is not a lossless pointer back into private JSON.
    pub pointer: String,
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct FeatureInventory {
    pub reachable: usize,
    pub packaged_only: usize,
    pub project_metadata: usize,
    pub locations: Vec<FeatureLocation>,
}

#[derive(Debug, Clone, Serialize)]
pub struct InventoryFinding {
    pub code: String,
    pub resource: String,
    pub pointer: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UninterpretedField {
    pub resource: String,
    pub pointer: String,
    pub shape: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResourceDependency {
    pub source: String,
    pub pointer: String,
    pub kind: String,
    /// Only a validated asset identifier, never script/shader/text contents.
    pub target: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct TextureDeclaration {
    pub format: u32,
    pub flags: u32,
    pub body_version: u32,
    pub storage_width: u32,
    pub storage_height: u32,
    pub image_width: u32,
    pub image_height: u32,
    pub images: u32,
    pub mips: usize,
    pub animation_frames: usize,
    pub video_container_signature: bool,
    pub nonzero_extra: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResourceInventory {
    pub path: String,
    pub origin: Option<ResourceOrigin>,
    pub bytes: Option<u64>,
    pub role: String,
    pub reachable: bool,
    pub packaged: bool,
    pub status: String,
    /// Available for parsed JSON/shaders. Texture/media contents are not hashed
    /// individually by this traversal (the example hashes the containing PKG).
    pub sha256: Option<String>,
    pub texture: Option<TextureDeclaration>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ObjectInventory {
    pub index: usize,
    pub id: Option<i64>,
    pub kinds: Vec<String>,
    pub parent_id: Option<i64>,
    pub parent_declared: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct GeneralDeclaration {
    pub field: String,
    /// A feature boolean only; project property values and strings are omitted.
    pub default_boolean: Option<bool>,
    pub property_binding: bool,
    pub script_binding: bool,
    pub shape: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ParticleComponentDeclaration {
    pub resource: String,
    pub pointer: String,
    pub category: String,
    pub type_identifier: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CameraDeclaration {
    pub orthogonal_dimensions: Option<[u32; 2]>,
    pub projection_auto_declared: bool,
    pub projection_auto_default: Option<bool>,
    pub paths_shape: String,
    pub active_paths_empty: Option<bool>,
    pub saved_xy_nonzero: bool,
    pub saved_eye_center_xy_equal: Option<bool>,
    pub saved_up_default: Option<bool>,
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct InventoryCoverage {
    pub documents: usize,
    pub document_bytes: usize,
    pub shader_bytes: usize,
    pub nodes: usize,
    pub texture_mips: usize,
    pub animation_frames: usize,
    pub framing_mips_budget_used: usize,
    pub framing_animation_frames_budget_used: usize,
    pub observations: usize,
    pub report_string_bytes: usize,
    pub package_entries: usize,
    /// Means bounded discovery finished, not that all semantics are understood.
    pub traversal_complete: bool,
}

/// A complete set of discovered blockers can contain several independent
/// features. It deliberately has no `runnable` or `supported` field.
#[derive(Debug, Clone, Serialize)]
pub struct SceneInventory {
    pub schema_version: u32,
    pub assessment_scope: String,
    pub scene_path: String,
    pub package_version: Option<String>,
    pub limits: InventoryLimits,
    pub coverage: InventoryCoverage,
    pub features: BTreeMap<String, FeatureInventory>,
    pub general: Vec<GeneralDeclaration>,
    pub camera: Option<CameraDeclaration>,
    pub objects: Vec<ObjectInventory>,
    pub particle_components: Vec<ParticleComponentDeclaration>,
    pub resources: Vec<ResourceInventory>,
    pub dependencies: Vec<ResourceDependency>,
    pub findings: Vec<InventoryFinding>,
    pub uninterpreted_fields: Vec<UninterpretedField>,
    pub boundaries: Vec<String>,
}

impl SceneInventory {
    /// Traverse the scene and its static references, project metadata and all
    /// packaged JSON/TEX/shaders. Unused packaged assets remain distinguished
    /// from the reachable graph. Loose files are followed only by references;
    /// loose directory enumeration and script execution are not performed.
    pub fn collect(
        assets: &SceneAssets,
        scene_path: &str,
        limits: InventoryLimits,
    ) -> Result<Self> {
        crate::package::validate_name(scene_path)?;
        limits.validate()?;
        let mut builder = Builder::new(assets, scene_path, limits);
        builder.add_resource(scene_path, false);
        match assets.resource("project.json") {
            Ok(_) => {
                builder.add_resource("project.json", false);
            }
            Err(SceneAssetError::NotFound { .. }) => {}
            Err(error) => builder.finding(error_code(&error), "project.json", None),
        }
        for entry in assets.package_entries() {
            builder.report.coverage.package_entries += 1;
            builder.add_resource(&entry.name, true);
        }
        while let Some(path) = builder.queue.pop_front() {
            builder.inspect(&path);
        }
        match assets.package_is_unchanged() {
            Ok(true) => {}
            Ok(false) => builder.partial("package_changed", scene_path, None),
            Err(error) => builder.partial(error_code(&error), scene_path, None),
        }
        builder.finish();
        Ok(builder.report)
    }
}

impl InventoryLimits {
    fn validate(&self) -> Result<()> {
        let hard = Self::default();
        for (name, value, maximum) in [
            ("resources", self.max_resources, hard.max_resources),
            ("documents", self.max_documents, hard.max_documents),
            (
                "document bytes",
                self.max_document_bytes,
                hard.max_document_bytes,
            ),
            (
                "total document bytes",
                self.max_total_document_bytes,
                hard.max_total_document_bytes,
            ),
            ("shader bytes", self.max_shader_bytes, hard.max_shader_bytes),
            (
                "total shader bytes",
                self.max_total_shader_bytes,
                hard.max_total_shader_bytes,
            ),
            ("nodes", self.max_nodes, hard.max_nodes),
            ("depth", self.max_depth, hard.max_depth),
            ("observations", self.max_observations, hard.max_observations),
            (
                "report string bytes",
                self.max_report_string_bytes,
                hard.max_report_string_bytes,
            ),
            ("texture mips", self.max_texture_mips, hard.max_texture_mips),
            (
                "animation frames",
                self.max_animation_frames,
                hard.max_animation_frames,
            ),
        ] {
            if value > maximum {
                return Err(limit(
                    format!("inventory {name}"),
                    value as u64,
                    maximum as u64,
                ));
            }
        }
        Ok(())
    }
}

struct Builder<'a> {
    assets: &'a SceneAssets,
    report: SceneInventory,
    indices: BTreeMap<String, usize>,
    queue: VecDeque<String>,
    stopped_observations: bool,
    texture_budget: TextureInspectionBudget,
}

impl<'a> Builder<'a> {
    fn new(assets: &'a SceneAssets, scene: &str, limits: InventoryLimits) -> Self {
        Self {
            assets,
            texture_budget: TextureInspectionBudget { remaining_mips: limits.max_texture_mips, remaining_animation_frames: limits.max_animation_frames },
            report: SceneInventory {
                schema_version: 1,
                assessment_scope: "static_dependency_inventory; no runtime qualification".into(),
                scene_path: scene.into(),
                package_version: assets.package_version().map(str::to_owned),
                limits,
                coverage: InventoryCoverage { traversal_complete: true, ..Default::default() },
                features: BTreeMap::new(), general: Vec::new(), camera: None, objects: Vec::new(), particle_components: Vec::new(),
                resources: Vec::new(), dependencies: Vec::new(), findings: Vec::new(),
                uninterpreted_fields: Vec::new(),
                boundaries: [
                    "Declarations and framing do not establish decode, shader, native visual or desktop runtime support.",
                    "Packaged-only declarations are not automatically required by the active scene.",
                    "JS payloads are neither exported nor executed; dynamically created references and input/API needs remain unknown.",
                    "Property bindings are distinct from JS; property expression/override semantics are not evaluated.",
                    "Particle components, child links and puppet references are observed, not simulated; puppet does not prove generic 3D.",
                    "Parent sentinels, inherited transforms, camera and HDR/bloom rendering semantics are not validated.",
                    "Shader discovery handles includes and sampler JSON defaults only; conditionals, macros and arbitrary code-generated assets remain unknown.",
                    "Audio/cursor tokens in shaders and included helpers do not prove active branches or required runtime input.",
                    "Texture inspection validates bounded framing only, skipping payloads; no image/LZ4/video decoding or codec validation is performed.",
                    "Unknown JSON fields are recorded by shape with hashed field names; private scalar values and text contents are omitted.",
                    "Loose directories are not enumerated; only static references and project metadata are followed. JSON schema semantics are not exhaustive.",
                ].map(str::to_owned).to_vec(),
            },
            indices: BTreeMap::new(), queue: VecDeque::new(), stopped_observations: false,
        }
    }

    fn observation(&mut self, strings: &[&str]) -> bool {
        let bytes: usize = strings.iter().map(|value| value.len()).sum();
        if self.report.coverage.observations >= self.report.limits.max_observations
            || bytes
                > self
                    .report
                    .limits
                    .max_report_string_bytes
                    .saturating_sub(self.report.coverage.report_string_bytes)
        {
            self.report.coverage.traversal_complete = false;
            if !self.stopped_observations {
                self.stopped_observations = true;
                // One reserved terminal finding even when the caller sets zero.
                self.report.findings.push(InventoryFinding {
                    code: "observation_limit".into(),
                    resource: self.report.scene_path.clone(),
                    pointer: None,
                });
            }
            return false;
        }
        self.report.coverage.observations += 1;
        self.report.coverage.report_string_bytes += bytes;
        true
    }

    fn finding(&mut self, code: &str, resource: &str, pointer: Option<&str>) {
        if self.observation(&[code, resource, pointer.unwrap_or("")]) {
            self.report.findings.push(InventoryFinding {
                code: code.into(),
                resource: resource.into(),
                pointer: pointer.map(str::to_owned),
            });
        }
    }

    fn partial(&mut self, code: &str, resource: &str, pointer: Option<&str>) {
        self.report.coverage.traversal_complete = false;
        self.finding(code, resource, pointer);
    }

    fn feature(&mut self, kind: &str, resource: &str, pointer: &str) {
        if self.observation(&[kind, resource, pointer]) {
            self.report
                .features
                .entry(kind.into())
                .or_default()
                .locations
                .push(FeatureLocation {
                    resource: resource.into(),
                    pointer: pointer.into(),
                });
        }
    }

    fn add_resource(&mut self, path: &str, packaged: bool) -> bool {
        if let Some(index) = self.indices.get(path).copied() {
            self.report.resources[index].packaged |= packaged;
            return true;
        }
        if self.indices.len() >= self.report.limits.max_resources {
            self.partial("resource_limit", &self.report.scene_path.clone(), None);
            return false;
        }
        if !self.observation(&[path, path, role(path)]) {
            return false;
        }
        let index = self.report.resources.len();
        self.indices.insert(path.into(), index);
        let (origin, bytes, status) = match self.assets.resource(path) {
            Ok(resource) => (Some(resource.origin()), Some(resource.len()), "present"),
            Err(error) => {
                self.partial(error_code(&error), path, None);
                (None, None, error_code(&error))
            }
        };
        let role = role(path);
        if status == "present" && matches!(role, "json" | "texture" | "shader") {
            self.queue.push_back(path.into());
        }
        let resource_status =
            if status == "present" && !matches!(role, "json" | "texture" | "shader") {
                "metadata_only"
            } else {
                status
            };
        self.report.resources.push(ResourceInventory {
            path: path.into(),
            origin,
            bytes,
            role: role.into(),
            reachable: false,
            packaged,
            status: resource_status.into(),
            sha256: None,
            texture: None,
        });
        true
    }

    fn dependency(&mut self, source: &str, pointer: &str, kind: &str, target: &str) {
        if self.stopped_observations {
            return;
        }
        if crate::package::validate_name(target).is_err() {
            self.unresolved(source, pointer, kind, "unsafe_reference");
            return;
        }
        let added = self.add_resource(target, false);
        let status = if added {
            let resource = &self.report.resources[self.indices[target]];
            if resource.origin.is_some() {
                "present".into()
            } else {
                resource.status.clone()
            }
        } else {
            "resource_limit".into()
        };
        if self.observation(&[source, pointer, kind, target, &status]) {
            self.report.dependencies.push(ResourceDependency {
                source: source.into(),
                pointer: pointer.into(),
                kind: kind.into(),
                target: Some(target.into()),
                status,
            });
        }
    }

    fn unresolved(&mut self, source: &str, pointer: &str, kind: &str, status: &str) {
        if self.observation(&[source, pointer, kind, status]) {
            self.report.dependencies.push(ResourceDependency {
                source: source.into(),
                pointer: pointer.into(),
                kind: kind.into(),
                target: None,
                status: status.into(),
            });
        }
    }

    fn inspect(&mut self, path: &str) {
        let index = self.indices[path];
        let resource = match self.assets.resource(path) {
            Ok(resource) => resource,
            Err(error) => {
                self.report.resources[index].status = error_code(&error).into();
                self.finding(error_code(&error), path, None);
                return;
            }
        };
        let size = resource.len();
        self.report.resources[index].origin = Some(resource.origin());
        self.report.resources[index].bytes = Some(size);
        match role(path) {
            "json" => {
                if self.report.coverage.documents >= self.report.limits.max_documents
                    || size > self.report.limits.max_document_bytes as u64
                    || size
                        > self
                            .report
                            .limits
                            .max_total_document_bytes
                            .saturating_sub(self.report.coverage.document_bytes)
                            as u64
                {
                    self.report.resources[index].status = "document_limit".into();
                    self.partial("document_limit", path, None);
                    return;
                }
                self.report.coverage.documents += 1;
                self.report.coverage.document_bytes += size as usize;
                match resource.read_range(0, size as usize, self.report.limits.max_document_bytes) {
                    Ok(bytes) => {
                        self.report.resources[index].sha256 = Some(hash(&bytes));
                        match serde_json::from_slice::<Value>(&bytes) {
                            Ok(value) => {
                                self.report.resources[index].status = "json_observed".into();
                                if path == self.report.scene_path {
                                    self.scene(&value);
                                }
                                self.walk(path, "", "", &value, 0, path != "project.json");
                            }
                            Err(_) => {
                                self.report.resources[index].status = "invalid_json".into();
                                self.partial("invalid_json", path, None);
                            }
                        }
                    }
                    Err(error) => self.partial(error_code(&error), path, None),
                }
            }
            "texture" => {
                if self.texture_budget.remaining_mips == 0 {
                    self.report.resources[index].status = "texture_mip_limit".into();
                    self.partial("texture_mip_limit", path, None);
                    return;
                }
                let result = Texture::inspect_resource_bounded(&resource, &mut self.texture_budget);
                self.report.coverage.framing_mips_budget_used =
                    self.report.limits.max_texture_mips - self.texture_budget.remaining_mips;
                self.report.coverage.framing_animation_frames_budget_used =
                    self.report.limits.max_animation_frames
                        - self.texture_budget.remaining_animation_frames;
                match result {
                    Ok(metadata) => {
                        self.report.coverage.texture_mips += metadata.mips.len();
                        self.report.coverage.animation_frames += metadata
                            .animation
                            .as_ref()
                            .map_or(0, |animation| animation.frames.len());
                        if metadata.animation.is_some() {
                            self.feature("texture_animation", path, "");
                        }
                        if metadata.video {
                            self.feature("video_texture", path, "");
                        }
                        self.report.resources[index].status = "texture_framing_observed".into();
                        self.report.resources[index].texture = Some(TextureDeclaration {
                            format: metadata.format,
                            flags: metadata.flags,
                            body_version: metadata.body_version,
                            storage_width: metadata.storage_width,
                            storage_height: metadata.storage_height,
                            image_width: metadata.image_width,
                            image_height: metadata.image_height,
                            images: metadata.images,
                            mips: metadata.mips.len(),
                            animation_frames: metadata
                                .animation
                                .map_or(0, |animation| animation.frames.len()),
                            video_container_signature: metadata.video,
                            nonzero_extra: metadata.body_extra.is_some_and(|value| value != 0)
                                || metadata.image_info_extra != 0,
                        });
                    }
                    Err(error) => {
                        self.report.resources[index].status = error_code(&error).into();
                        self.partial(error_code(&error), path, None);
                    }
                }
            }
            "shader" => {
                if size > self.report.limits.max_shader_bytes as u64
                    || size
                        > self
                            .report
                            .limits
                            .max_total_shader_bytes
                            .saturating_sub(self.report.coverage.shader_bytes)
                            as u64
                {
                    self.report.resources[index].status = "shader_limit".into();
                    self.partial("shader_limit", path, None);
                    return;
                }
                self.report.coverage.shader_bytes += size as usize;
                match resource.read_range(0, size as usize, self.report.limits.max_shader_bytes) {
                    Ok(bytes) => {
                        self.report.resources[index].sha256 = Some(hash(&bytes));
                        match std::str::from_utf8(&bytes) {
                            Ok(source) => {
                                self.shader(path, source);
                                self.report.resources[index].status =
                                    "shader_references_observed".into();
                            }
                            Err(_) => self.partial("non_utf8_shader", path, None),
                        }
                    }
                    Err(error) => self.partial(error_code(&error), path, None),
                }
            }
            _ => {}
        }
    }

    fn scene(&mut self, value: &Value) {
        if let Some(general) = value.get("general").and_then(Value::as_object) {
            for field in ["hdr", "bloom", "cameraparallax", "camerashake", "zoom"] {
                if let Some(value) = general.get(field) {
                    let default = value.get("value").unwrap_or(value);
                    self.report.general.push(GeneralDeclaration {
                        field: field.into(),
                        default_boolean: default.as_bool(),
                        property_binding: value.get("user").is_some(),
                        script_binding: value.get("script").is_some()
                            || value.get("scenescript").is_some(),
                        shape: shape(value).into(),
                    });
                    if default.as_bool() == Some(true)
                        || value.is_object()
                        || (field == "zoom" && default.as_f64().is_some_and(|number| number != 1.0))
                    {
                        self.feature(
                            field,
                            &self.report.scene_path.clone(),
                            &format!("/general/{field}"),
                        );
                    }
                }
            }
        }
        if let Some(camera) = value.get("camera") {
            let path = self.report.scene_path.clone();
            self.feature("camera", &path, "/camera");
            let projection = value
                .get("general")
                .and_then(|general| general.get("orthogonalprojection"))
                .or_else(|| camera.get("orthogonalprojection"));
            let eye = camera.get("eye").and_then(vector);
            let center = camera.get("center").and_then(vector);
            let paths = camera.get("paths");
            let active = paths
                .and_then(|paths| paths.get("active"))
                .or_else(|| paths.filter(|paths| paths.is_array()));
            self.report.camera = Some(CameraDeclaration {
                orthogonal_dimensions: projection.and_then(|projection| {
                    Some([
                        u32::try_from(projection.get("width")?.as_u64()?).ok()?,
                        u32::try_from(projection.get("height")?.as_u64()?).ok()?,
                    ])
                }),
                projection_auto_declared: projection
                    .is_some_and(|projection| projection.get("auto").is_some()),
                projection_auto_default: projection
                    .and_then(|projection| projection.get("auto"))
                    .and_then(Value::as_bool),
                paths_shape: paths.map_or("missing", shape).into(),
                active_paths_empty: active.and_then(Value::as_array).map(Vec::is_empty),
                saved_xy_nonzero: [eye, center]
                    .into_iter()
                    .flatten()
                    .any(|coordinates| coordinates[0] != 0.0 || coordinates[1] != 0.0),
                saved_eye_center_xy_equal: eye
                    .zip(center)
                    .map(|(eye, center)| eye[..2] == center[..2]),
                saved_up_default: camera
                    .get("up")
                    .and_then(vector)
                    .map(|up| up == [0.0, 1.0, 0.0]),
            });
            for field in ["eye", "center"] {
                if let Some(coordinates) = camera.get(field).and_then(vector) {
                    if coordinates[0] != 0.0 || coordinates[1] != 0.0 {
                        self.feature("saved_camera_xy", &path, &format!("/camera/{field}"));
                    }
                } else if camera.get(field).is_some() {
                    self.finding(
                        "unknown_camera_vector",
                        &path,
                        Some(&format!("/camera/{field}")),
                    );
                }
            }
            if value
                .get("general")
                .and_then(|general| general.get("orthogonalprojection"))
                .is_none()
                && camera.get("orthogonalprojection").is_none()
            {
                self.feature("unknown_projection", &path, "/camera");
            }
        }
        let Some(objects) = value.get("objects").and_then(Value::as_array) else {
            self.finding(
                "unknown_scene_objects",
                &self.report.scene_path.clone(),
                Some("/objects"),
            );
            return;
        };
        for (index, object) in objects.iter().enumerate() {
            if !self.observation(&[]) {
                break;
            }
            let kinds: Vec<String> = [
                "image", "particle", "text", "sound", "model", "light", "camera", "video", "web",
            ]
            .into_iter()
            .filter(|kind| object.get(*kind).is_some())
            .map(str::to_owned)
            .collect();
            if kinds.is_empty() {
                self.finding(
                    "unknown_object_kind",
                    &self.report.scene_path.clone(),
                    Some(&format!("/objects/{index}")),
                );
            }
            self.report.objects.push(ObjectInventory {
                index,
                id: object.get("id").and_then(Value::as_i64),
                parent_id: object.get("parent").and_then(Value::as_i64),
                parent_declared: object.get("parent").is_some(),
                kinds,
            });
        }
    }

    fn walk(
        &mut self,
        source: &str,
        pointer: &str,
        key: &str,
        value: &Value,
        depth: usize,
        allow_references: bool,
    ) {
        if self.stopped_observations {
            return;
        }
        if depth > self.report.limits.max_depth
            || self.report.coverage.nodes >= self.report.limits.max_nodes
        {
            self.partial("json_traversal_limit", source, Some(pointer));
            return;
        }
        self.report.coverage.nodes += 1;
        // Script values are opaque. No recursion, snippets or inferred paths.
        if matches!(key, "script" | "scenescript") {
            self.feature("scene_script", source, pointer);
            return;
        }
        if let Some(object) = value.as_object() {
            if object.contains_key("user") && object.contains_key("value") {
                self.feature("property_binding", source, pointer);
            }
            for (field, child) in object {
                let normalized = field.to_ascii_lowercase();
                let next = format!("{pointer}/{}", safe_field(&normalized));
                match normalized.as_str() {
                    "image" => self.feature("image", source, &next),
                    "particle" => self.feature("particle", source, &next),
                    "sound" => self.feature("audio", source, &next),
                    "text" => self.feature("text", source, &next),
                    "font" => self.feature("font", source, &next),
                    "parent" => self.feature("hierarchy", source, &next),
                    "puppet" => self.feature("puppet", source, &next),
                    "model" => self.feature("model_reference", source, &next),
                    "effects" => {
                        if let Some(effects) = child.as_array() {
                            for index in 0..effects.len() {
                                self.feature("effect", source, &format!("{next}/{index}"));
                                if self.stopped_observations {
                                    break;
                                }
                            }
                        } else {
                            self.finding("unknown_effect_shape", source, Some(&next));
                        }
                    }
                    "children" if source.starts_with("particles/") => {
                        if let Some(children) = child.as_array() {
                            for (index, declaration) in children.iter().enumerate() {
                                self.feature("particle_child", source, &format!("{next}/{index}"));
                                if allow_references
                                    && let Some(identifier) =
                                        declaration.get("name").and_then(Value::as_str)
                                {
                                    if is_asset(identifier) {
                                        self.dependency(
                                            source,
                                            &format!("{next}/{index}/name"),
                                            "particle_child",
                                            identifier,
                                        );
                                    } else {
                                        self.unresolved(
                                            source,
                                            &format!("{next}/{index}/name"),
                                            "particle_child",
                                            "unknown_reference_convention",
                                        );
                                    }
                                }
                                if self.stopped_observations {
                                    break;
                                }
                            }
                        } else if !child.is_null() {
                            self.finding("unknown_particle_children", source, Some(&next));
                        }
                    }
                    "emitters" | "initializers" | "operators" | "renderers" | "emitter"
                    | "initializer" | "operator" | "renderer"
                        if source.starts_with("particles/") =>
                    {
                        let components = child
                            .as_array()
                            .map(Vec::as_slice)
                            .unwrap_or(std::slice::from_ref(child));
                        for (index, component) in components.iter().enumerate() {
                            self.feature(
                                &format!("particle_{normalized}"),
                                source,
                                &format!("{next}/{index}"),
                            );
                            let location = format!("{next}/{index}");
                            let identifier =
                                component
                                    .get("name")
                                    .and_then(Value::as_str)
                                    .filter(|name| {
                                        allow_references
                                            && !name.is_empty()
                                            && name.len() <= 96
                                            && name.bytes().all(|byte| {
                                                byte.is_ascii_alphanumeric() || byte == b'_'
                                            })
                                    });
                            if self.observation(&[
                                source,
                                &location,
                                &normalized,
                                identifier.unwrap_or(""),
                            ]) {
                                self.report.particle_components.push(
                                    ParticleComponentDeclaration {
                                        resource: source.into(),
                                        pointer: location,
                                        category: normalized.clone(),
                                        type_identifier: identifier.map(str::to_owned),
                                    },
                                );
                            }
                            if identifier.is_none() {
                                self.finding(
                                    "unknown_particle_component_type",
                                    source,
                                    Some(&format!("{next}/{index}/name")),
                                );
                            }
                            if self.stopped_observations {
                                break;
                            }
                        }
                    }
                    _ => {}
                }
                if !known_field(&normalized) && self.observation(&[source, &next, shape(child)]) {
                    self.report.uninterpreted_fields.push(UninterpretedField {
                        resource: source.into(),
                        pointer: next.clone(),
                        shape: shape(child).into(),
                    });
                }
                // A property's name and a text body's value are private data,
                // not asset references. Bindings still inspect their `value`.
                if matches!(
                    normalized.as_str(),
                    "user" | "name" | "title" | "description" | "label"
                ) || (normalized == "text" && !child.is_object())
                {
                    continue;
                }
                let inherited_key = if normalized == "value"
                    && matches!(
                        key,
                        "textures"
                            | "texture"
                            | "image"
                            | "particle"
                            | "material"
                            | "puppet"
                            | "model"
                            | "sound"
                            | "file"
                            | "child"
                            | "font"
                            | "shader"
                    ) {
                    key
                } else {
                    &normalized
                };
                let child_references = allow_references
                    && known_field(&normalized)
                    && normalized != "text"
                    && !(key == "text" && normalized == "value");
                self.walk(
                    source,
                    &next,
                    inherited_key,
                    child,
                    depth + 1,
                    child_references,
                );
            }
        } else if let Some(array) = value.as_array() {
            for (index, child) in array.iter().enumerate() {
                self.walk(
                    source,
                    &format!("{pointer}/{index}"),
                    key,
                    child,
                    depth + 1,
                    allow_references,
                );
                if self.stopped_observations
                    || self.report.coverage.nodes >= self.report.limits.max_nodes
                {
                    if index + 1 < array.len() {
                        self.partial("json_traversal_limit", source, Some(pointer));
                    }
                    break;
                }
            }
        } else if let Some(identifier) = value.as_str() {
            if allow_references {
                self.reference(source, pointer, key, identifier);
            } else if is_asset(identifier) {
                self.unresolved(
                    source,
                    pointer,
                    "unknown_context",
                    "possible_resource_reference; value_omitted",
                );
            }
        }
    }

    fn reference(&mut self, source: &str, pointer: &str, key: &str, identifier: &str) {
        if source == "project.json" {
            return;
        }
        if key == "shader" {
            self.feature("shader", source, pointer);
            if matches!(extension(identifier), "vert" | "frag" | "glsl") {
                self.dependency(source, pointer, "shader", identifier);
            } else {
                for extension in ["vert", "frag"] {
                    self.dependency(
                        source,
                        pointer,
                        "shader",
                        &format!("shaders/{identifier}.{extension}"),
                    );
                }
            }
        } else if matches!(key, "textures" | "texture") {
            self.texture_dependency(source, pointer, "texture", identifier);
        } else if key == "font" {
            if matches!(extension(identifier), "ttf" | "otf") {
                self.dependency(source, pointer, "font", identifier);
            } else {
                self.unresolved(source, pointer, "font", "unresolved_font_family");
            }
        } else if is_asset(identifier)
            && matches!(
                key,
                "image"
                    | "particle"
                    | "material"
                    | "puppet"
                    | "model"
                    | "sound"
                    | "file"
                    | "child"
                    | "fontfile"
            )
        {
            self.dependency(
                source,
                pointer,
                if key.is_empty() { "resource" } else { key },
                identifier,
            );
        } else if matches!(
            key,
            "image" | "particle" | "material" | "puppet" | "model" | "sound" | "file" | "child"
        ) {
            self.unresolved(source, pointer, key, "unknown_reference_convention");
        } else if is_asset(identifier) {
            self.unresolved(
                source,
                pointer,
                "unknown_field",
                "possible_resource_reference; value_omitted",
            );
        }
    }

    fn texture_dependency(&mut self, source: &str, pointer: &str, kind: &str, identifier: &str) {
        if identifier.starts_with("_rt_")
            || identifier.starts_with("_alias_")
            || identifier.is_empty()
        {
            self.unresolved(source, pointer, kind, "implicit_runtime_resource");
        } else {
            self.dependency(source, pointer, kind, &texture_path(identifier));
        }
    }

    fn shader(&mut self, path: &str, source: &str) {
        for (index, line) in source.lines().enumerate() {
            if self.stopped_observations {
                break;
            }
            let pointer = format!("/line/{}", index + 1);
            let trimmed = line.trim();
            if let Some(include) = trimmed.strip_prefix("#include") {
                let include = include.trim();
                if let Some(identifier) = include
                    .strip_prefix('"')
                    .and_then(|value| value.split_once('"').map(|(identifier, _)| identifier))
                {
                    self.dependency(
                        path,
                        &pointer,
                        "shader_include",
                        &format!("shaders/{identifier}"),
                    );
                } else {
                    self.unresolved(path, &pointer, "shader_include", "unknown_include_syntax");
                }
            }
            if ["g_AudioSpectrum", "g_AudioLevel", "g_AudioVolume"]
                .iter()
                .any(|name| line.contains(name))
            {
                self.feature(
                    if trimmed.starts_with("uniform ") {
                        "audio_uniform_declaration"
                    } else {
                        "audio_reactive_reference"
                    },
                    path,
                    &pointer,
                );
            }
            if ["g_MousePosition", "g_CursorPosition"]
                .iter()
                .any(|name| line.contains(name))
            {
                self.feature("mouse_input", path, &pointer);
            }
            if line.contains("sampler")
                && let Some((_, metadata)) = line.split_once("//")
                && metadata.trim().starts_with('{')
            {
                match serde_json::from_str::<Value>(metadata.trim()) {
                    Ok(value) => {
                        if let Some(identifier) = value.get("default").and_then(Value::as_str) {
                            self.texture_dependency(
                                path,
                                &pointer,
                                "shader_texture_default",
                                identifier,
                            );
                        }
                    }
                    Err(_) => self.finding("unparsed_shader_metadata", path, Some(&pointer)),
                }
            }
        }
    }

    fn finish(&mut self) {
        let mut reachable = BTreeSet::from([self.report.scene_path.clone()]);
        let mut adjacency: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
        for dependency in &self.report.dependencies {
            if let Some(target) = &dependency.target {
                adjacency
                    .entry(&dependency.source)
                    .or_default()
                    .push(target);
            }
        }
        let mut queue = VecDeque::from([self.report.scene_path.clone()]);
        while let Some(source) = queue.pop_front() {
            for target in adjacency.get(source.as_str()).into_iter().flatten() {
                if self.indices.contains_key(*target) && reachable.insert((*target).into()) {
                    queue.push_back((*target).into());
                }
            }
        }
        let mut custom_shaders = Vec::new();
        let mut custom_effects = Vec::new();
        let mut script_resources = Vec::new();
        for resource in &mut self.report.resources {
            resource.reachable = reachable.contains(&resource.path);
            if resource
                .origin
                .is_some_and(|origin| origin != ResourceOrigin::Builtin)
                && resource.role == "shader"
            {
                // Custom origin is observable, without asserting the shader is
                // a novel algorithm or that it differs from a built-in file.
                custom_shaders.push(resource.path.clone());
            }
            if resource
                .origin
                .is_some_and(|origin| origin != ResourceOrigin::Builtin)
                && resource.path.starts_with("effects/")
                && resource.path.ends_with("effect.json")
            {
                custom_effects.push(resource.path.clone());
            }
            if resource.role == "script" {
                script_resources.push(resource.path.clone());
            }
        }
        for path in custom_shaders {
            self.feature("project_shader_resource", &path, "");
        }
        for path in custom_effects {
            self.feature("project_effect_resource", &path, "");
        }
        for path in script_resources {
            self.feature("script_resource", &path, "");
        }
        for feature in self.report.features.values_mut() {
            for location in &feature.locations {
                if location.resource == "project.json" {
                    feature.project_metadata += 1;
                } else if reachable.contains(&location.resource) {
                    feature.reachable += 1;
                } else {
                    feature.packaged_only += 1;
                }
            }
        }
        self.report
            .resources
            .sort_by(|left, right| left.path.cmp(&right.path));
    }
}

fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn extension(path: &str) -> &str {
    path.rsplit_once('.').map_or("", |(_, extension)| extension)
}
fn role(path: &str) -> &'static str {
    match extension(path) {
        "json" | "tex-json" => "json",
        "tex" => "texture",
        "vert" | "frag" | "h" | "glsl" => "shader",
        "mp3" | "flac" | "wav" | "ogg" | "opus" | "aac" | "m4a" => "audio",
        "ttf" | "otf" => "font",
        "mdl" | "fbx" | "obj" | "glb" | "gltf" => "model_binary",
        "js" | "lua" => "script",
        "png" | "jpeg" | "jpg" | "gif" | "webp" => "image_file",
        "mp4" | "webm" | "avi" => "video_file",
        _ => "unclassified",
    }
}
fn is_asset(identifier: &str) -> bool {
    identifier.len() <= 4096
        && !identifier.contains(['\n', '\r'])
        && role(identifier) != "unclassified"
}
fn texture_path(identifier: &str) -> String {
    if is_asset(identifier) {
        identifier.into()
    } else if identifier.starts_with("materials/") {
        format!("{identifier}.tex")
    } else {
        format!("materials/{identifier}.tex")
    }
}
fn shape(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}
fn vector(value: &Value) -> Option<[f64; 3]> {
    if let Some(string) = value.as_str() {
        let mut parts = string.split_whitespace();
        let result = [
            parts.next()?.parse().ok()?,
            parts.next()?.parse().ok()?,
            parts.next()?.parse().ok()?,
        ];
        if parts.next().is_some() {
            return None;
        }
        if result.iter().any(|number: &f64| !number.is_finite()) {
            return None;
        }
        Some(result)
    } else {
        let array = value.as_array()?;
        if array.len() != 3 {
            return None;
        }
        Some([array[0].as_f64()?, array[1].as_f64()?, array[2].as_f64()?])
    }
}
fn error_code(error: &SceneAssetError) -> &'static str {
    match error {
        SceneAssetError::NotFound { .. } => "missing_resource",
        SceneAssetError::UnsafePath { .. } => "unsafe_reference",
        SceneAssetError::LimitExceeded { .. } => "resource_limit",
        SceneAssetError::Unsupported { .. } => "unknown_framing",
        SceneAssetError::InvalidTexture { .. } | SceneAssetError::InvalidPackage { .. } => {
            "invalid_framing"
        }
        SceneAssetError::Io { .. } => "resource_io_error",
    }
}
fn safe_field(field: &str) -> String {
    if known_field(field) {
        field.replace('~', "~0").replace('/', "~1")
    } else {
        format!("<field:{}>", &hash(field.as_bytes())[..12])
    }
}
fn known_field(field: &str) -> bool {
    matches!(
        field,
        "general"
            | "camera"
            | "objects"
            | "id"
            | "image"
            | "particle"
            | "text"
            | "sound"
            | "model"
            | "light"
            | "video"
            | "web"
            | "parent"
            | "puppet"
            | "effects"
            | "file"
            | "material"
            | "textures"
            | "texture"
            | "shader"
            | "font"
            | "child"
            | "children"
            | "emitters"
            | "initializers"
            | "operators"
            | "renderers"
            | "emitter"
            | "initializer"
            | "operator"
            | "renderer"
            | "script"
            | "scenescript"
            | "user"
            | "value"
            | "visible"
            | "origin"
            | "scale"
            | "angles"
            | "size"
            | "autosize"
            | "passes"
            | "fbos"
            | "target"
            | "bind"
            | "index"
            | "name"
            | "combos"
            | "constantshadervalues"
            | "constants"
            | "settings"
            | "hdr"
            | "bloom"
            | "cameraparallax"
            | "camerashake"
            | "camerazoom"
            | "zoom"
            | "eye"
            | "center"
            | "up"
            | "orthogonalprojection"
            | "width"
            | "height"
            | "blending"
            | "depthtest"
            | "depthwrite"
            | "cullmode"
            | "volume"
            | "properties"
            | "title"
            | "description"
            | "label"
    )
}

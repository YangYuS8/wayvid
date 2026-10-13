use std::{
    fs,
    io::{Seek, SeekFrom, Write},
    path::Path,
};

use lwe_scene::{
    InventoryLimits, SceneAssetError, SceneAssets, SceneInventory, Texture, TextureInspectionBudget,
};
use serde_json::{Value, json};
use tempfile::TempDir;

fn write(root: &Path, name: &str, bytes: &[u8]) {
    let path = root.join(name);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, bytes).unwrap();
}
fn document(root: &Path, name: &str, value: Value) {
    write(root, name, &serde_json::to_vec(&value).unwrap());
}
fn tex(video_size: u32, animation: bool) -> Vec<u8> {
    let mut bytes = b"TEXV0005\0TEXI0001\0".to_vec();
    for value in [0_u32, 0, 1, 1, 1, 1, 0] {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    bytes.extend_from_slice(b"TEXB0001\0");
    for value in [1_u32, 1, 1, 1, if video_size > 0 { video_size } else { 4 }] {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    if video_size > 0 {
        bytes.extend_from_slice(b"\0\0\0\x20ftyp");
    } else {
        bytes.extend_from_slice(&[255, 0, 0, 255]);
    }
    if animation {
        bytes.extend_from_slice(b"TEXS0003\0");
        for value in [1_u32, 1, 1, 0] {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        bytes.extend_from_slice(&0.1_f32.to_le_bytes());
        for _ in 0..6 {
            bytes.extend_from_slice(&0_f32.to_le_bytes());
        }
    }
    bytes
}
fn collect(root: &Path) -> SceneInventory {
    SceneInventory::collect(
        &SceneAssets::open(root, None).unwrap(),
        "scene.json",
        InventoryLimits::default(),
    )
    .unwrap()
}
fn feature(report: &SceneInventory, name: &str) -> usize {
    report
        .features
        .get(name)
        .map_or(0, |feature| feature.reachable)
}

#[test]
fn collects_independent_dependencies_without_exporting_private_values() {
    let fixture = TempDir::new().unwrap();
    let root = fixture.path();
    document(
        root,
        "scene.json",
        json!({
            "general":{"hdr":false,"bloom":{"user":"hdr","value":true}},
            "camera":{"eye":"2 3 1","center":"2 3 0","orthogonalprojection":{"width":1,"height":1}},
            "objects":[
                {"id":1,"image":"models/image.json","effects":[{"file":"effects/custom/effect.json"}],"visible":{"user":"private-property-name","value":true}},
                {"id":2,"particle":"particles/root.json","parent":1},
            {"id":3,"text":{"value":"PRIVATE_TEXT_MARKER","script":"PRIVATE_SCRIPT_MARKER()"},"font":"fonts/missing.ttf"},
                {"id":4,"sound":["audio/missing.flac"]},
                {"id":5,"image":"models/puppet.json"}
            ],
            "private-secret":{"value":"PRIVATE_SECRET_MARKER.mp3"}
        }),
    );
    document(
        root,
        "models/image.json",
        json!({"material":"materials/image.json"}),
    );
    document(
        root,
        "models/puppet.json",
        json!({"puppet":"models/missing.mdl","material":"materials/image.json"}),
    );
    document(
        root,
        "materials/image.json",
        json!({"passes":[{"shader":"custom","textures":["image","animated","video"]}]}),
    );
    document(
        root,
        "particles/root.json",
        json!({"children":[{"name":"particles/child.json"}],"emitters":[{"id":0,"name":"boxrandom"}],"operators":[{"id":1,"name":"movement"}],"material":"materials/image.json"}),
    );
    document(
        root,
        "particles/child.json",
        json!({"children":[{"name":"particles/root.json"}],"initializers":[{"id":1,"name":"lifetimerandom"}]}),
    );
    document(
        root,
        "effects/custom/effect.json",
        json!({"passes":[{"material":"materials/missing.json"}]}),
    );
    write(
        root,
        "shaders/custom.vert",
        b"#include \"missing.h\"\nvoid main(){}\n",
    );
    write(
        root,
        "shaders/custom.frag",
        b"uniform sampler2D g_Texture0; // {\"default\":\"missing\"}\n// PRIVATE_SHADER_MARKER\n",
    );
    write(root, "materials/image.tex", &tex(0, false));
    write(root, "materials/animated.tex", &tex(0, true));
    write(root, "materials/video.tex", &tex(8, false));
    let report = collect(root);
    assert_eq!(feature(&report, "scene_script"), 1);
    assert_eq!(feature(&report, "property_binding"), 2);
    assert_eq!(feature(&report, "hdr"), 0);
    assert_eq!(feature(&report, "bloom"), 1);
    assert!(
        report
            .particle_components
            .iter()
            .any(|component| component.type_identifier.as_deref() == Some("boxrandom"))
    );
    for name in [
        "saved_camera_xy",
        "hierarchy",
        "puppet",
        "particle_child",
        "text",
        "font",
        "audio",
        "texture_animation",
        "video_texture",
        "effect",
    ] {
        assert!(feature(&report, name) > 0, "missing feature {name}");
    }
    let missing: Vec<_> = report
        .dependencies
        .iter()
        .filter(|reference| reference.status == "missing_resource")
        .collect();
    assert!(missing.len() >= 6);
    assert!(
        report
            .resources
            .iter()
            .any(|resource| resource.path == "particles/child.json" && resource.reachable)
    );
    let serialized = serde_json::to_string(&report).unwrap();
    for secret in [
        "PRIVATE_TEXT_MARKER",
        "PRIVATE_SCRIPT_MARKER",
        "PRIVATE_SECRET_MARKER",
        "PRIVATE_SHADER_MARKER",
        "private-property-name",
        "private-secret",
    ] {
        assert!(
            !serialized.contains(secret),
            "exported private payload {secret}"
        );
    }
    assert!(!serialized.contains("\"runnable\""));
}

#[test]
fn aggregate_limits_and_invalid_documents_are_partial_not_first_blockers() {
    let fixture = TempDir::new().unwrap();
    document(
        fixture.path(),
        "scene.json",
        json!({"objects":[{"image":"models/broken.json","particle":"particles/good.json"}],"unknown":[1,2,3,4,5,6]}),
    );
    write(
        fixture.path(),
        "models/broken.json",
        b"{ BROKEN_PRIVATE_MARKER",
    );
    document(
        fixture.path(),
        "particles/good.json",
        json!({"children":[]}),
    );
    let report = collect(fixture.path());
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.code == "invalid_json")
    );
    assert!(report.resources.iter().any(
        |resource| resource.path == "particles/good.json" && resource.status == "json_observed"
    ));
    assert!(!report.coverage.traversal_complete);
    assert!(
        !serde_json::to_string(&report)
            .unwrap()
            .contains("BROKEN_PRIVATE_MARKER")
    );
    let assets = SceneAssets::open(fixture.path(), None).unwrap();
    let limits = InventoryLimits {
        max_total_document_bytes: fs::metadata(fixture.path().join("scene.json"))
            .unwrap()
            .len() as usize,
        ..InventoryLimits::default()
    };
    let partial = SceneInventory::collect(&assets, "scene.json", limits).unwrap();
    assert!(!partial.coverage.traversal_complete);
    assert!(
        partial
            .findings
            .iter()
            .any(|finding| finding.code == "document_limit")
    );
    let limits = InventoryLimits {
        max_observations: 2,
        ..InventoryLimits::default()
    };
    let partial = SceneInventory::collect(&assets, "scene.json", limits).unwrap();
    assert!(partial.coverage.observations <= 2);
    assert!(
        partial
            .findings
            .iter()
            .any(|finding| finding.code == "observation_limit")
    );
    let limits = InventoryLimits {
        max_report_string_bytes: 16,
        ..InventoryLimits::default()
    };
    let partial = SceneInventory::collect(&assets, "scene.json", limits).unwrap();
    assert!(partial.coverage.report_string_bytes <= 16);
    assert!(!partial.coverage.traversal_complete);
}

#[test]
fn unsafe_and_symlink_references_are_reported_without_reading_targets() {
    use std::os::unix::fs::symlink;
    let fixture = TempDir::new().unwrap();
    document(
        fixture.path(),
        "scene.json",
        json!({"objects":[{"image":"../secret.json"},{"image":"models/link.json"}]}),
    );
    fs::create_dir(fixture.path().join("models")).unwrap();
    symlink("/etc/passwd", fixture.path().join("models/link.json")).unwrap();
    let report = collect(fixture.path());
    assert!(
        report
            .dependencies
            .iter()
            .any(|reference| reference.status == "unsafe_reference" && reference.target.is_none())
    );
    assert!(
        report
            .dependencies
            .iter()
            .any(|reference| reference.status == "resource_io_error")
    );
    assert!(
        !serde_json::to_string(&report)
            .unwrap()
            .contains("../secret.json")
    );
}

#[test]
fn sparse_video_framing_uses_ranges_and_checks_declared_length() {
    let fixture = TempDir::new().unwrap();
    let size = 100 * 1024 * 1024;
    let header = tex(size, false);
    let path = fixture.path().join("video.tex");
    let mut file = fs::File::create(&path).unwrap();
    file.write_all(&header).unwrap();
    file.set_len(header.len() as u64 + size as u64 - 8).unwrap();
    let assets = SceneAssets::open(fixture.path(), None).unwrap();
    let resource = assets.resource("video.tex").unwrap();
    let metadata = Texture::inspect_resource(&resource).unwrap();
    assert!(metadata.video);
    assert_eq!(metadata.mips[0].stored_size, size);
    assert!(resource.read_range(resource.len() - 1, 2, 2).is_err());
    assert!(resource.read_range(0, 10, 9).is_err());
    file.seek(SeekFrom::Start(0)).unwrap();
    file.set_len(header.len() as u64).unwrap();
    let resource = assets.resource("video.tex").unwrap();
    assert!(Texture::inspect_resource(&resource).is_err());
}

#[test]
fn package_only_script_and_texture_are_not_active_scene_dependencies() {
    let fixture = TempDir::new().unwrap();
    let entries = [
        (
            "scene.json",
            serde_json::to_vec(&json!({"objects":[]})).unwrap(),
        ),
        (
            "unused.json",
            serde_json::to_vec(&json!({"script":"DO_NOT_EXPORT"})).unwrap(),
        ),
        ("unused.tex", tex(0, true)),
    ];
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&8_u32.to_le_bytes());
    bytes.extend_from_slice(b"PKGV0021");
    bytes.extend_from_slice(&(entries.len() as u32).to_le_bytes());
    let mut offset = 0_u32;
    for (name, payload) in &entries {
        bytes.extend_from_slice(&(name.len() as u32).to_le_bytes());
        bytes.extend_from_slice(name.as_bytes());
        bytes.extend_from_slice(&offset.to_le_bytes());
        bytes.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        offset += payload.len() as u32;
    }
    for (_, payload) in &entries {
        bytes.extend_from_slice(payload);
    }
    write(fixture.path(), "scene.pkg", &bytes);
    let report = collect(fixture.path());
    assert_eq!(feature(&report, "scene_script"), 0);
    assert_eq!(report.features["scene_script"].packaged_only, 1);
    assert_eq!(report.features["texture_animation"].packaged_only, 1);
    assert_eq!(report.coverage.package_entries, 3);
    assert!(
        !serde_json::to_string(&report)
            .unwrap()
            .contains("DO_NOT_EXPORT")
    );
    let assets = SceneAssets::open(fixture.path(), None).unwrap();
    assert_eq!(
        assets
            .package_sha256(bytes.len() as u64)
            .unwrap()
            .unwrap()
            .len(),
        64
    );
    assert!(assets.package_sha256(bytes.len() as u64 - 1).is_err());
    let original_hash = assets.package_sha256(bytes.len() as u64).unwrap();
    // A replaced pathname must not redirect an indexed reader to a new inode.
    fs::hard_link(
        fixture.path().join("scene.pkg"),
        fixture.path().join("original.pkg"),
    )
    .unwrap();
    let mut replacement = bytes.clone();
    *replacement.last_mut().unwrap() = 1;
    write(fixture.path(), "replacement.pkg", &replacement);
    fs::rename(
        fixture.path().join("replacement.pkg"),
        fixture.path().join("scene.pkg"),
    )
    .unwrap();
    assert!(assets.package_is_unchanged().unwrap());
    assert_eq!(
        assets.package_sha256(bytes.len() as u64).unwrap(),
        original_hash
    );
    assert_eq!(assets.read("scene.json", 4096).unwrap(), entries[0].1);
    let current = SceneAssets::open(fixture.path(), None).unwrap();
    assert_ne!(
        current.package_sha256(bytes.len() as u64).unwrap(),
        original_hash
    );
    // An ordinary in-place change of the original descriptor is a partial
    // inventory, even if all old directory ranges remain readable.
    fs::OpenOptions::new()
        .write(true)
        .open(fixture.path().join("original.pkg"))
        .unwrap()
        .set_len(bytes.len() as u64 + 1)
        .unwrap();
    assert!(!assets.package_is_unchanged().unwrap());
    assert!(assets.package_sha256(bytes.len() as u64 + 1).is_err());
    let changed =
        SceneInventory::collect(&assets, "scene.json", InventoryLimits::default()).unwrap();
    assert!(!changed.coverage.traversal_complete);
    assert!(
        changed
            .findings
            .iter()
            .any(|finding| finding.code == "package_changed")
    );
}

#[test]
fn binding_texture_default_keeps_reference_context_and_node_truncation_is_explicit() {
    let fixture = TempDir::new().unwrap();
    document(
        fixture.path(),
        "scene.json",
        json!({"objects":[{"image":"models/image.json"}]}),
    );
    document(
        fixture.path(),
        "models/image.json",
        json!({"material":"materials/image.json"}),
    );
    document(
        fixture.path(),
        "materials/image.json",
        json!({"passes":[{"textures":[{"user":"private-name","value":"image"}]}]}),
    );
    write(fixture.path(), "materials/image.tex", &tex(0, false));
    let report = collect(fixture.path());
    assert!(
        report
            .resources
            .iter()
            .any(|resource| resource.path == "materials/image.tex" && resource.reachable)
    );
    assert!(
        !serde_json::to_string(&report)
            .unwrap()
            .contains("private-name")
    );
    document(fixture.path(), "scene.json", json!([1, 2, 3, 4]));
    let assets = SceneAssets::open(fixture.path(), None).unwrap();
    let limits = InventoryLimits {
        max_nodes: 2,
        ..InventoryLimits::default()
    };
    let report = SceneInventory::collect(&assets, "scene.json", limits).unwrap();
    assert!(!report.coverage.traversal_complete);
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.code == "json_traversal_limit")
    );
}

#[test]
fn unmodeled_and_text_payload_subtrees_cannot_export_known_key_values() {
    let fixture = TempDir::new().unwrap();
    document(
        fixture.path(),
        "scene.json",
        json!({
            "objects":[{"text":{"value":{"file":"PRIVATE_TEXT_MARKER.mp3","textures":["PRIVATE_TEXTURE_MARKER.tex"]},"script":"PRIVATE_SCRIPT_MARKER()"}}],
            "private-secret":{"file":"PRIVATE_SECRET_MARKER.mp3","image":"PRIVATE_IMAGE_MARKER.json","nested":{"script":"PRIVATE_NESTED_SCRIPT_MARKER()"}}
        }),
    );
    let report = collect(fixture.path());
    assert_eq!(feature(&report, "scene_script"), 2);
    let serialized = serde_json::to_string(&report).unwrap();
    for marker in [
        "PRIVATE_TEXT_MARKER",
        "PRIVATE_TEXTURE_MARKER",
        "PRIVATE_SECRET_MARKER",
        "PRIVATE_IMAGE_MARKER",
        "PRIVATE_SCRIPT_MARKER",
        "PRIVATE_NESTED_SCRIPT_MARKER",
    ] {
        assert!(!serialized.contains(marker));
    }
    assert!(
        report
            .dependencies
            .iter()
            .any(|reference| reference.kind == "unknown_context" && reference.target.is_none())
    );
}

#[test]
fn framing_budgets_are_consumed_before_mip_and_animation_loops() {
    let fixture = TempDir::new().unwrap();
    let mut incomplete = tex(0, false);
    // TEXV/TEXI + seven u32 + TEXB + image count; second mip is absent.
    incomplete[59..63].copy_from_slice(&2_u32.to_le_bytes());
    incomplete.truncate(63);
    write(fixture.path(), "missing-mips.tex", &incomplete);
    let assets = SceneAssets::open(fixture.path(), None).unwrap();
    let mut budget = TextureInspectionBudget {
        remaining_mips: 1,
        remaining_animation_frames: 1,
    };
    assert!(
        matches!(Texture::inspect_resource_bounded(&assets.resource("missing-mips.tex").unwrap(),&mut budget), Err(SceneAssetError::LimitExceeded {resource,..}) if resource=="texture inspection mips")
    );
    assert_eq!(budget.remaining_mips, 1);
    let mut animation = tex(0, true);
    let tag = animation
        .windows(9)
        .position(|window| window == b"TEXS0003\0")
        .unwrap();
    animation.truncate(tag + 9 + 4);
    write(fixture.path(), "missing-frames.tex", &animation);
    let mut budget = TextureInspectionBudget {
        remaining_mips: 1,
        remaining_animation_frames: 0,
    };
    assert!(
        matches!(Texture::inspect_resource_bounded(&assets.resource("missing-frames.tex").unwrap(),&mut budget), Err(SceneAssetError::LimitExceeded {resource,..}) if resource=="texture inspection animation frames")
    );
    assert_eq!(budget.remaining_mips, 0);
    document(
        fixture.path(),
        "scene.json",
        json!({"objects":[{"image":"models/image.json"}]}),
    );
    document(
        fixture.path(),
        "models/image.json",
        json!({"material":"materials/image.json"}),
    );
    document(
        fixture.path(),
        "materials/image.json",
        json!({"passes":[{"textures":["first","second"]}]}),
    );
    write(fixture.path(), "materials/first.tex", &tex(0, true));
    write(fixture.path(), "materials/second.tex", &tex(0, true));
    let limits = InventoryLimits {
        max_animation_frames: 1,
        ..InventoryLimits::default()
    };
    let report = SceneInventory::collect(&assets, "scene.json", limits).unwrap();
    assert_eq!(report.coverage.framing_animation_frames_budget_used, 1);
    assert_eq!(report.coverage.animation_frames, 1);
    assert!(!report.coverage.traversal_complete);
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.code == "resource_limit")
    );
}

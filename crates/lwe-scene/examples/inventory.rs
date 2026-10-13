//! Read-only static inventory: inventory PROJECTS [ASSETS]
//! Emits metadata only; does not decode textures, run scripts or operate desktops.
use std::{
    env,
    io::{self, Write},
    path::{Path, PathBuf},
};

use lwe_scene::{InventoryLimits, SceneAssets, SceneInventory};
use serde::Serialize;

#[derive(Serialize)]
struct Sample {
    id: String,
    package_sha256: Option<String>,
    inventory: SceneInventory,
}

fn inspect(path: &Path, assets: Option<&Path>) -> Result<Sample, Box<dyn std::error::Error>> {
    let reader = SceneAssets::open(path, assets)?;
    Ok(Sample {
        id: path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned(),
        package_sha256: reader.package_sha256(2 * 1024 * 1024 * 1024)?,
        inventory: SceneInventory::collect(&reader, "scene.json", InventoryLimits::default())?,
    })
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arguments: Vec<_> = env::args_os().skip(1).collect();
    if arguments.is_empty() || arguments.len() > 2 {
        return Err("usage: inventory PROJECTS [ASSETS]".into());
    }
    let root = PathBuf::from(&arguments[0]);
    let assets = arguments.get(1).map(PathBuf::from);
    let projects = if root.join("scene.pkg").is_file() || root.join("scene.json").is_file() {
        vec![root]
    } else {
        let mut projects = Vec::new();
        for entry in std::fs::read_dir(root)? {
            let path = entry?.path();
            if path.join("scene.pkg").is_file() {
                projects.push(path);
            }
            if projects.len() > 64 {
                return Err("inventory example project count exceeds 64".into());
            }
        }
        projects.sort();
        projects
    };
    if projects.is_empty() {
        return Err("no scene projects found".into());
    }
    // Keep one bounded report in memory even when inspecting a corpus.
    let mut output = io::stdout().lock();
    output.write_all(b"{\"schema_version\":1,\"scope\":\"private local corpus; static declarations and framing; no runtime eligibility assertion\",\"samples\":[\n")?;
    for (index, project) in projects.iter().enumerate() {
        if index != 0 {
            output.write_all(b",\n")?;
        }
        serde_json::to_writer_pretty(&mut output, &inspect(project, assets.as_deref())?)?;
    }
    output.write_all(b"\n]}\n")?;
    Ok(())
}

//! Read-only resource acceptance: cargo run -p lwe-scene --example inspect -- PROJECTS [ASSETS]
//! PROJECTS may be one wallpaper directory or a directory of Workshop items.
use std::env;
use std::io::Read;
use std::path::{Path, PathBuf};

use lwe_scene::{MAX_ASSET_BYTES, SceneAssetError, SceneAssets, Texture};

fn inspect(project: &Path, assets: Option<&Path>) -> Result<(), Box<dyn std::error::Error>> {
    let reader = SceneAssets::open(project, assets)?;
    let _scene = reader.read("scene.json", 8 * 1024 * 1024)?;
    let mut decoded = 0;
    let mut limited = 0;
    for entry in reader
        .package_entries()
        .filter(|e| e.name.ends_with(".tex"))
    {
        let bytes = reader.read(&entry.name, MAX_ASSET_BYTES)?;
        let metadata = Texture::inspect(&bytes)?;
        match Texture::decode(&bytes) {
            Ok(texture) => {
                decoded += 1;
                println!(
                    "{} {}: decoded {}x{}, format={}, flags={}, mips={}",
                    project.display(),
                    entry.name,
                    texture.width,
                    texture.height,
                    metadata.format,
                    metadata.flags,
                    metadata.mips.len()
                );
            }
            Err(SceneAssetError::Unsupported { feature }) => {
                limited += 1;
                println!(
                    "{} {}: capability limit: {}; animation={}, video={}",
                    project.display(),
                    entry.name,
                    feature,
                    metadata.animation.is_some(),
                    metadata.video
                );
            }
            Err(error) => return Err(error.into()),
        }
    }
    println!(
        "{}: package={}, entries={}, decoded={}, limited={} (resource check; no runtime acceptance)",
        project.display(),
        reader.package_version().unwrap_or("unpacked"),
        reader.package_entries().count(),
        decoded,
        limited
    );
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = env::args_os().skip(1).collect();
    if args.len() == 2 && args[0] == "--texture" {
        let file = std::fs::File::open(&args[1])?;
        let mut bytes = Vec::new();
        file.take(MAX_ASSET_BYTES as u64 + 1)
            .read_to_end(&mut bytes)?;
        let texture = Texture::decode(&bytes)?;
        println!(
            "{}: decoded {}x{}, format={}, flags={}, TEXB{}, mips={}, rgba_prefix={:02x?}",
            Path::new(&args[1]).display(),
            texture.width,
            texture.height,
            texture.metadata.format,
            texture.metadata.flags,
            texture.metadata.body_version,
            texture.metadata.mips.len(),
            &texture.rgba[..texture.rgba.len().min(16)]
        );
        return Ok(());
    }
    if args.is_empty() || args.len() > 2 {
        return Err("usage: inspect PROJECTS [ASSETS], or inspect --texture FILE".into());
    }
    let path = PathBuf::from(&args[0]);
    let assets = args.get(1).map(PathBuf::from);
    if path.join("scene.pkg").is_file() || path.join("scene.json").is_file() {
        return inspect(&path, assets.as_deref());
    }
    let mut projects: Vec<_> = std::fs::read_dir(&path)?
        .collect::<std::io::Result<Vec<_>>>()?
        .into_iter()
        .map(|entry| entry.path())
        .filter(|path| path.join("scene.pkg").is_file())
        .collect();
    projects.sort();
    if projects.is_empty() {
        return Err("no scene projects found".into());
    }
    for project in projects {
        inspect(&project, assets.as_deref())?;
    }
    Ok(())
}

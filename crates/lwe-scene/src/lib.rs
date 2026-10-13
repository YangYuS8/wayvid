//! Independently implemented resource readers for LWE scenes.
//!
//! Reading a package or texture does not establish scene runtime support.
//! Format decisions are based on inspected local assets and native format
//! observations, rather than another renderer's implementation.

mod inventory;
mod package;
mod texture;

pub use inventory::*;
pub use package::{PackageEntry, ResourceOrigin, SceneAssets, SceneResource};
pub use texture::{
    AnimationFrame, AnimationMetadata, MipMetadata, Texture, TextureInspectionBudget,
    TextureMetadata,
};

/// Hard allocation bound for a single resource read (including video blobs).
pub const MAX_ASSET_BYTES: usize = 512 * 1024 * 1024;
/// Hard allocation bound for decoded pixels or a decompressed mip.
pub const MAX_TEXTURE_BYTES: usize = 256 * 1024 * 1024;
/// Maximum accepted texture dimension, in either direction.
pub const MAX_TEXTURE_DIMENSION: u32 = 16_384;

pub type Result<T> = std::result::Result<T, SceneAssetError>;

#[derive(Debug, thiserror::Error)]
pub enum SceneAssetError {
    #[error("scene resource not found: {name}")]
    NotFound { name: String },
    #[error("unsafe scene resource path: {name}")]
    UnsafePath { name: String },
    #[error("invalid scene package: {reason}")]
    InvalidPackage { reason: String },
    #[error("invalid scene texture: {reason}")]
    InvalidTexture { reason: String },
    #[error("scene resource limit exceeded: {resource} ({size} > {limit})")]
    LimitExceeded {
        resource: String,
        size: u64,
        limit: u64,
    },
    #[error("unsupported scene resource feature: {feature}")]
    Unsupported { feature: String },
    #[error("{context}: {source}")]
    Io {
        context: String,
        #[source]
        source: std::io::Error,
    },
}

fn io(context: impl Into<String>, source: std::io::Error) -> SceneAssetError {
    SceneAssetError::Io {
        context: context.into(),
        source,
    }
}

fn limit(resource: impl Into<String>, size: u64, maximum: u64) -> SceneAssetError {
    SceneAssetError::LimitExceeded {
        resource: resource.into(),
        size,
        limit: maximum,
    }
}

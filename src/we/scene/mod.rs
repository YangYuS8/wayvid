//! Scene wallpaper module for Wallpaper Engine compatibility
//!
//! This module provides support for rendering Wallpaper Engine scene-type wallpapers.
//! Scene wallpapers consist of multiple layers (images, effects) with animations.

// Allow unused re-exports as they are part of the public API
#![allow(unused_imports)]

mod parser;
mod pkg;
mod renderer;
mod shader;
mod texture;
mod types;

pub use parser::{SceneParser, SceneProject};
pub use pkg::{PkgReader, SceneContainer};
pub use renderer::{RenderRect, SceneRenderer};
pub use shader::{QuadMesh, ShaderProgram};
pub use texture::{Texture, TextureFlags, TextureFormat, TextureMipmap};
pub use types::*;

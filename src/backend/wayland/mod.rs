pub mod app;
pub mod conflicts;
pub mod output;
pub mod surface;

use crate::config::Config;
use anyhow::Result;
use std::path::PathBuf;

pub fn run(config: Config, config_path: Option<PathBuf>) -> Result<()> {
    app::run(config, config_path)
}

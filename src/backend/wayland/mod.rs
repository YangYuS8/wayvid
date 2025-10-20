pub mod app;
pub mod output;
pub mod surface;

use crate::config::Config;
use anyhow::Result;

pub fn run(config: Config) -> Result<()> {
    app::run(config)
}

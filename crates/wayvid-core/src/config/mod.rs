//! Configuration types
//!
//! Legacy CLI configuration format (config.yaml) for backwards compatibility.
//! New GUI-first approach uses AppSettings (settings.yaml) instead.

mod pattern;
mod types;

pub use pattern::matches_pattern;
pub use types::*;

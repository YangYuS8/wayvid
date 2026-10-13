//! Read-only CPU preparation: scene_prepare PROJECT [BUILTIN_ASSETS] [--properties JSON_FILE]
//! Does not start a backend, create GL resources, or write settings/session state.

use std::{collections::BTreeMap, env, path::PathBuf, process::ExitCode};

use lwe_engine::scene::{PreparedScene, unsupported_feature_code};
use serde_json::json;

fn main() -> ExitCode {
    let arguments: Vec<_> = env::args_os().skip(1).collect();
    if arguments.is_empty() || arguments.len() > 4 {
        eprintln!("usage: scene_prepare PROJECT [BUILTIN_ASSETS] [--properties JSON_FILE]");
        return ExitCode::FAILURE;
    }
    let project = PathBuf::from(&arguments[0]);
    let property_index = arguments
        .iter()
        .position(|argument| argument == "--properties");
    let assets = arguments
        .get(1)
        .filter(|argument| *argument != "--properties")
        .map(PathBuf::from);
    let properties = (|| -> anyhow::Result<BTreeMap<String, serde_json::Value>> {
        if let Some(index) = property_index {
            anyhow::ensure!(index == 1 || index == 2, "Invalid property option position");
            anyhow::ensure!(
                arguments.len() == index + 2,
                "Properties require exactly one JSON path"
            );
            let path = PathBuf::from(&arguments[index + 1]);
            anyhow::ensure!(
                std::fs::metadata(&path)?.len() <= 64 * 1024,
                "Property file exceeds limit"
            );
            let bytes = std::fs::read(path)?;
            anyhow::ensure!(bytes.len() <= 64 * 1024, "Property file exceeds limit");
            Ok(serde_json::from_slice(&bytes)?)
        } else {
            anyhow::ensure!(arguments.len() <= 2, "Unknown preparation arguments");
            Ok(BTreeMap::new())
        }
    })();
    let prepared = properties.and_then(|properties| {
        PreparedScene::prepare_with_properties(&project, assets.as_deref(), &properties)
    });
    let (result, status) = match prepared {
        Ok(prepared) => {
            let summary = prepared.summary();
            (
                json!({
                    "prepared":true,
                    "layers":summary.layer_count,
                    "effects":summary.effect_count,
                    "passes":summary.pass_count,
                    "animated":summary.animated,
                }),
                ExitCode::SUCCESS,
            )
        }
        Err(error) => (
            json!({
                "prepared":false,
                "feature":unsupported_feature_code(&error),
                "reason":format!("{error:#}"),
            }),
            ExitCode::FAILURE,
        ),
    };
    println!(
        "{}",
        json!({
            "scope":"CPU preparation; no desktop or visual acceptance",
            "result":result,
        })
    );
    status
}

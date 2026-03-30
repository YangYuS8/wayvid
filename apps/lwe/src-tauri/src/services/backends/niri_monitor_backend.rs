use std::process::Command;

use serde_json::Value;

use crate::services::backends::monitor_backend::{
    BackendMonitorDescriptor, BackendMonitorDiscovery, MonitorBackend,
};

pub struct NiriMonitorBackend;

impl MonitorBackend for NiriMonitorBackend {
    fn list_monitors(&self) -> BackendMonitorDiscovery {
        let output = match Command::new("niri").args(["msg", "-j", "outputs"]).output() {
            Ok(output) => output,
            Err(error) => {
                return BackendMonitorDiscovery::Unavailable {
                    reason: format!("Failed to run `niri msg -j outputs`: {error}"),
                };
            }
        };

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            let reason = if stderr.is_empty() {
                format!("`niri msg -j outputs` exited with status {}", output.status)
            } else {
                format!("`niri msg -j outputs` failed: {stderr}")
            };

            return BackendMonitorDiscovery::Unavailable { reason };
        }

        let parsed = match serde_json::from_slice::<Value>(&output.stdout) {
            Ok(parsed) => parsed,
            Err(error) => {
                return BackendMonitorDiscovery::Unavailable {
                    reason: format!("Failed to parse `niri msg -j outputs`: {error}"),
                };
            }
        };

        parse_outputs(parsed)
    }
}

fn parse_outputs(parsed: Value) -> BackendMonitorDiscovery {
    let Some(outputs) = parsed.as_object() else {
        return BackendMonitorDiscovery::Unavailable {
            reason: "`niri msg -j outputs` did not return an object".to_string(),
        };
    };

    let monitors = outputs
        .iter()
        .filter_map(|(output_id, output)| parse_output(output_id, output))
        .collect();

    BackendMonitorDiscovery::Known(monitors)
}

fn parse_output(output_id: &str, output: &Value) -> Option<BackendMonitorDescriptor> {
    let object = output.as_object()?;
    let name = object
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or(output_id)
        .to_string();
    let make = object.get("make").and_then(Value::as_str).unwrap_or("");
    let model = object.get("model").and_then(Value::as_str).unwrap_or("");
    let display_name = format_display_name(make, model, &name);
    let resolution = current_resolution(object)?;

    Some(BackendMonitorDescriptor {
        id: name,
        name: display_name,
        resolution,
    })
}

fn format_display_name(make: &str, model: &str, fallback: &str) -> String {
    let display_name = [make.trim(), model.trim()]
        .into_iter()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" ");

    if display_name.is_empty() {
        fallback.to_string()
    } else {
        display_name
    }
}

fn current_resolution(output: &serde_json::Map<String, Value>) -> Option<String> {
    let modes = output.get("modes")?.as_array()?;
    let current_mode = output.get("current_mode")?.as_u64()? as usize;
    let mode = modes.get(current_mode)?.as_object()?;
    let width = mode.get("width")?.as_u64()?;
    let height = mode.get("height")?.as_u64()?;

    Some(format!("{width}x{height}"))
}

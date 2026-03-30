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

    let mut monitors = Vec::with_capacity(outputs.len());

    for (output_id, output) in outputs {
        let monitor = match parse_output(output_id, output) {
            Ok(monitor) => monitor,
            Err(reason) => return BackendMonitorDiscovery::Unavailable { reason },
        };

        monitors.push(monitor);
    }

    BackendMonitorDiscovery::Known(monitors)
}

fn parse_output(output_id: &str, output: &Value) -> Result<BackendMonitorDescriptor, String> {
    let object = output
        .as_object()
        .ok_or_else(|| format!("Output `{output_id}` was not an object"))?;
    let name = object
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or(output_id)
        .to_string();
    let make = object.get("make").and_then(Value::as_str).unwrap_or("");
    let model = object.get("model").and_then(Value::as_str).unwrap_or("");
    let serial = object.get("serial").and_then(Value::as_str).unwrap_or("");
    let display_name = format_display_name(make, model, &name);
    let resolution = current_resolution(output_id, object)?;

    Ok(BackendMonitorDescriptor {
        id: stable_monitor_id(make, model, serial, &name),
        name: display_name,
        resolution,
    })
}

fn stable_monitor_id(make: &str, model: &str, serial: &str, connector: &str) -> String {
    let mut parts = vec!["niri".to_string()];

    for part in [make, model] {
        let part = part.trim();
        if !part.is_empty() {
            parts.push(part.to_string());
        }
    }

    let serial = serial.trim();
    if !serial.is_empty() {
        parts.push(serial.to_string());
        return parts.join(":");
    }

    let connector = connector.trim();
    if !connector.is_empty() {
        parts.push(connector.to_string());
    }

    parts.join(":")
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

fn current_resolution(
    output_id: &str,
    output: &serde_json::Map<String, Value>,
) -> Result<String, String> {
    let modes = output
        .get("modes")
        .and_then(Value::as_array)
        .ok_or_else(|| format!("Output `{output_id}` is missing modes"))?;
    let current_mode = output
        .get("current_mode")
        .and_then(Value::as_u64)
        .ok_or_else(|| format!("Output `{output_id}` is missing current_mode"))?
        as usize;
    let mode = modes
        .get(current_mode)
        .and_then(Value::as_object)
        .ok_or_else(|| format!("Output `{output_id}` is missing the current resolution"))?;
    let width = mode
        .get("width")
        .and_then(Value::as_u64)
        .ok_or_else(|| format!("Output `{output_id}` is missing resolution width"))?;
    let height = mode
        .get("height")
        .and_then(Value::as_u64)
        .ok_or_else(|| format!("Output `{output_id}` is missing resolution height"))?;

    Ok(format!("{width}x{height}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parse_outputs_reports_unavailable_when_any_output_shape_is_invalid() {
        let result = parse_outputs(json!({
            "eDP-1": {
                "name": "eDP-1",
                "make": "BOE",
                "model": "0x0893",
                "serial": null,
                "modes": [
                    { "width": 2160, "height": 1440 }
                ],
                "current_mode": 0
            },
            "HDMI-A-1": {
                "name": "HDMI-A-1",
                "make": "Dell",
                "model": "U2720Q",
                "serial": null,
                "modes": [],
                "current_mode": 0
            }
        }));

        assert!(matches!(
            result,
            BackendMonitorDiscovery::Unavailable { reason }
                if reason.contains("HDMI-A-1") && reason.contains("resolution")
        ));
    }

    #[test]
    fn parse_outputs_uses_serial_backed_monitor_id_when_available() {
        let result = parse_outputs(json!({
            "DP-1": {
                "name": "DP-1",
                "make": "Dell",
                "model": "U2720Q",
                "serial": "CN12345678",
                "modes": [
                    { "width": 3840, "height": 2160 }
                ],
                "current_mode": 0
            }
        }));

        assert!(matches!(
            result,
            BackendMonitorDiscovery::Known(monitors)
                if monitors.len() == 1
                    && monitors[0].id == "niri:Dell:U2720Q:CN12345678"
                    && monitors[0].name == "Dell U2720Q"
                    && monitors[0].resolution == "3840x2160"
        ));
    }
}

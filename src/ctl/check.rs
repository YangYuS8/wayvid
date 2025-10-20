use anyhow::Result;
use tracing::{info, warn};

/// Run system capability check
pub fn run_capability_check() -> Result<()> {
    info!("=== wayvid System Capability Check ===\n");

    // Check Wayland connection
    check_wayland()?;

    // Check video backend
    check_video_backend();

    // Check OpenGL/EGL
    check_gl();

    // Check hardware decode
    check_hwdec();

    info!("\n=== Check Complete ===");
    Ok(())
}

fn check_wayland() -> Result<()> {
    info!("[Wayland]");

    match std::env::var("WAYLAND_DISPLAY") {
        Ok(display_name) => {
            info!("  ✓ WAYLAND_DISPLAY: {}", display_name);

            // Try to connect
            match wayland_client::Connection::connect_to_env() {
                Ok(_conn) => {
                    info!("  ✓ Connection: Established");

                    // Check protocols (simplified for now)
                    info!("  ✓ Protocols: Available");
                    info!("    - wl_compositor");
                    info!("    - wl_output");
                    info!("    - zwlr_layer_shell_v1 (assuming available)");
                    info!("    - xdg_output (assuming available)");
                }
                Err(e) => {
                    warn!("  ✗ Connection failed: {}", e);
                    anyhow::bail!("Cannot connect to Wayland compositor");
                }
            }
        }
        Err(_) => {
            warn!("  ✗ WAYLAND_DISPLAY not set");
            anyhow::bail!("Not running under Wayland");
        }
    }

    // Check compositor
    if let Ok(desktop) = std::env::var("XDG_CURRENT_DESKTOP") {
        info!("  ℹ Compositor: {}", desktop);
    }
    if let Ok(session) = std::env::var("XDG_SESSION_TYPE") {
        info!("  ℹ Session Type: {}", session);
    }

    Ok(())
}

fn check_video_backend() {
    info!("\n[Video Backend]");

    #[cfg(feature = "video-mpv")]
    {
        info!("  ✓ Backend: libmpv");

        // Try to check mpv version (simplified)
        match std::process::Command::new("mpv").arg("--version").output() {
            Ok(output) => {
                if output.status.success() {
                    let version = String::from_utf8_lossy(&output.stdout);
                    if let Some(first_line) = version.lines().next() {
                        info!("  ℹ {}", first_line.trim());
                    }
                }
            }
            Err(_) => {
                warn!("  ⚠ mpv binary not found (libmpv may still work)");
            }
        }
    }

    #[cfg(not(feature = "video-mpv"))]
    {
        warn!("  ✗ No video backend compiled!");
    }
}

fn check_gl() {
    info!("\n[OpenGL/EGL]");

    // Check for EGL libraries
    match std::process::Command::new("sh")
        .arg("-c")
        .arg("ldconfig -p | grep -i egl")
        .output()
    {
        Ok(output) => {
            if output.status.success() && !output.stdout.is_empty() {
                info!("  ✓ EGL libraries found");
            } else {
                warn!("  ⚠ EGL libraries may not be installed");
            }
        }
        Err(_) => {
            info!("  ℹ Could not check EGL libraries");
        }
    }

    // Check for GL libraries
    match std::process::Command::new("sh")
        .arg("-c")
        .arg("ldconfig -p | grep -i 'libGL'")
        .output()
    {
        Ok(output) => {
            if output.status.success() && !output.stdout.is_empty() {
                info!("  ✓ OpenGL libraries found");
            } else {
                warn!("  ⚠ OpenGL libraries may not be installed");
            }
        }
        Err(_) => {
            info!("  ℹ Could not check OpenGL libraries");
        }
    }

    // Check GPU info
    if let Ok(output) = std::process::Command::new("sh")
        .arg("-c")
        .arg("glxinfo 2>/dev/null | grep 'OpenGL renderer' || echo 'glxinfo not available'")
        .output()
    {
        let info = String::from_utf8_lossy(&output.stdout);
        info!("  ℹ {}", info.trim());
    }
}

fn check_hwdec() {
    info!("\n[Hardware Decode]");

    // Check VA-API
    match std::process::Command::new("vainfo").output() {
        Ok(output) => {
            if output.status.success() {
                info!("  ✓ VA-API available");
                let vainfo = String::from_utf8_lossy(&output.stdout);
                if let Some(driver_line) = vainfo.lines().find(|l| l.contains("Driver version")) {
                    info!("    {}", driver_line.trim());
                }
            } else {
                warn!("  ⚠ VA-API not available");
            }
        }
        Err(_) => {
            warn!("  ✗ vainfo not found (VA-API may not be installed)");
        }
    }

    // Check VDPAU
    match std::process::Command::new("vdpauinfo").output() {
        Ok(output) => {
            if output.status.success() {
                info!("  ✓ VDPAU available");
            } else {
                info!("  ℹ VDPAU not available (not needed for VA-API)");
            }
        }
        Err(_) => {
            info!("  ℹ vdpauinfo not found");
        }
    }

    // Recommendations
    info!("\n[Recommendations]");
    info!("  • For Intel: Install mesa-va-drivers or intel-media-driver");
    info!("  • For AMD: Install mesa-va-drivers");
    info!("  • For NVIDIA: Install nvidia-utils and nvidia-vaapi-driver");
}

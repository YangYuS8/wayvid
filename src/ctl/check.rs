use anyhow::Result;
use tracing::{error, info, warn};

/// Run system capability check
pub fn run_capability_check() -> Result<()> {
    // Initialize tracing for this command
    use tracing_subscriber::EnvFilter;
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_writer(std::io::stdout)
        .without_time()
        .with_target(false)
        .with_level(false)
        .init();

    println!("╔══════════════════════════════════════════════════╗");
    println!("║   wayvid System Diagnostic & Capability Check   ║");
    println!("╚══════════════════════════════════════════════════╝\n");

    let mut all_ok = true;

    // Check Wayland connection
    if let Err(e) = check_wayland() {
        error!("✗ Wayland check failed: {}", e);
        all_ok = false;
    }

    // Check compositor compatibility
    check_compositor();

    // Check Niri integration
    check_niri();

    // Check video backend
    check_video_backend();

    // Check OpenGL/EGL
    check_gl();

    // Check hardware decode
    check_hwdec();

    // Check daemon status
    check_daemon_status();

    // Check config file
    check_config();

    println!("\n╔══════════════════════════════════════════════════╗");
    if all_ok {
        println!("║              ✓ All Checks Passed!               ║");
        println!("╚══════════════════════════════════════════════════╝");
        println!("\nYou're ready to use wayvid! Run: wayvid run");
    } else {
        println!("║           ⚠  Some Issues Detected               ║");
        println!("╚══════════════════════════════════════════════════╝");
        println!("\nSee recommendations above to fix issues.");
    }

    Ok(())
}

fn check_compositor() {
    println!("\n[🖥️  Compositor Compatibility]");

    if let Ok(desktop) = std::env::var("XDG_CURRENT_DESKTOP") {
        info!("  ℹ Desktop: {}", desktop);

        let desktop_lower = desktop.to_lowercase();
        if desktop_lower.contains("hyprland") {
            info!("  ✓ Hyprland detected - Fully supported!");
        } else if desktop_lower.contains("niri") {
            info!("  ✓ Niri detected - Fully supported with optimizations!");
        } else if desktop_lower.contains("sway") {
            info!("  ✓ Sway detected - Should work (uses wlr-layer-shell)");
        } else if desktop_lower.contains("river") {
            info!("  ℹ River detected - Should work (uses wlr-layer-shell)");
        } else if desktop_lower.contains("kde") || desktop_lower.contains("plasma") {
            warn!("  ✗ KDE Plasma detected - NOT SUPPORTED");
            warn!("    Plasma does not support wlr-layer-shell protocol");
            warn!("    Use Hyprland, Sway, or Niri instead");
        } else if desktop_lower.contains("gnome") {
            warn!("  ✗ GNOME detected - NOT SUPPORTED");
            warn!("    GNOME does not support wlr-layer-shell protocol");
            warn!("    Use Hyprland, Sway, or Niri instead");
        } else {
            warn!("  ⚠ Unknown compositor: {}", desktop);
            warn!("    wayvid requires wlr-layer-shell support");
        }
    } else {
        warn!("  ⚠ Cannot detect compositor (XDG_CURRENT_DESKTOP not set)");
    }
}

fn check_niri() {
    println!("\n[🎯 Niri Integration]");

    if let Ok(socket) = std::env::var("NIRI_SOCKET") {
        info!("  ✓ Niri socket detected: {}", socket);

        // Check if socket exists
        if std::path::Path::new(&socket).exists() {
            info!("  ✓ Socket file exists");
            info!("  ✓ Workspace-aware optimizations will be enabled");
            info!("    • Inactive workspaces: throttled to 1 FPS");
            info!("    • Active workspace: full FPS");
        } else {
            warn!("  ⚠ Socket path set but file doesn't exist");
        }
    } else {
        info!("  ℹ Not running under Niri (NIRI_SOCKET not set)");
        info!("    Niri-specific optimizations will be disabled");
    }
}

fn check_daemon_status() {
    println!("\n[🔌 Daemon Status]");

    use crate::ctl::ipc_client::IpcClient;

    if IpcClient::is_running() {
        info!("  ✓ wayvid daemon is running");

        // Try to get status
        match IpcClient::connect() {
            Ok(mut client) => {
                use crate::ctl::protocol::IpcCommand;
                match client.send_command(&IpcCommand::GetStatus) {
                    Ok(response) => {
                        info!("  ✓ IPC communication working");
                        info!("    Response: {:?}", response);
                    }
                    Err(e) => {
                        warn!("  ⚠ IPC communication failed: {}", e);
                    }
                }
            }
            Err(e) => {
                warn!("  ⚠ Could not connect to daemon: {}", e);
            }
        }
    } else {
        info!("  ℹ wayvid daemon is not running");
        info!("    Start with: wayvid run");
        info!("    Or enable systemd service: systemctl --user enable --now wayvid.service");
    }
}

fn check_config() {
    println!("\n[⚙️  Configuration]");

    let config_path = dirs::config_dir()
        .map(|p| p.join("wayvid/config.yaml"))
        .unwrap_or_else(|| std::path::PathBuf::from("~/.config/wayvid/config.yaml"));

    if config_path.exists() {
        info!("  ✓ Config file found: {}", config_path.display());

        // Try to parse it
        match std::fs::read_to_string(&config_path) {
            Ok(content) => match serde_yaml::from_str::<serde_yaml::Value>(&content) {
                Ok(_) => {
                    info!("  ✓ Config file is valid YAML");
                }
                Err(e) => {
                    error!("  ✗ Config file has YAML errors: {}", e);
                    error!("    Fix the syntax errors in your config.yaml");
                }
            },
            Err(e) => {
                warn!("  ⚠ Cannot read config file: {}", e);
            }
        }
    } else {
        info!("  ℹ No config file found (using defaults)");
        info!("    Create one at: {}", config_path.display());
        info!(
            "    Example: cp /usr/share/wayvid/config.example.yaml {}",
            config_path.display()
        );
    }
}

fn check_wayland() -> Result<()> {
    println!("\n[🌊 Wayland Environment]");

    match std::env::var("WAYLAND_DISPLAY") {
        Ok(wayland_display) => {
            info!("  ✓ WAYLAND_DISPLAY: {}", wayland_display);
        }
        Err(_) => {
            error!("  ✗ WAYLAND_DISPLAY is not set");
            error!("    wayvid requires Wayland to run");
            return Err(anyhow::anyhow!("Not running in a Wayland session"));
        }
    }

    match std::env::var("XDG_RUNTIME_DIR") {
        Ok(dir) => {
            info!("  ✓ XDG_RUNTIME_DIR: {}", dir);
        }
        Err(_) => {
            error!("  ✗ XDG_RUNTIME_DIR is not set");
            error!("    Required for IPC socket communication");
            return Err(anyhow::anyhow!("XDG_RUNTIME_DIR not set"));
        }
    }

    Ok(())
}

fn check_video_backend() {
    println!("\n[🎬 Video Backend]");

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
        error!("  ✗ No video backend compiled!");
        error!("    Rebuild with: cargo build --features video-mpv");
    }
}

fn check_gl() {
    println!("\n[✨ OpenGL/EGL]");

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
                warn!("    Install mesa or your GPU driver package");
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
                warn!("    Install mesa or your GPU driver package");
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
        let gpu_info = String::from_utf8_lossy(&output.stdout);
        let trimmed = gpu_info.trim();
        if !trimmed.is_empty() && !trimmed.contains("not available") {
            info!("  ℹ {}", trimmed);
        }
    }
}

fn check_hwdec() {
    println!("\n[🚀 Hardware Decode]");

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
                warn!("  ⚠ VA-API not working properly");
            }
        }
        Err(_) => {
            warn!("  ✗ vainfo not found (VA-API may not be installed)");
            println!("\n[💡 Hardware Decode Recommendations]");
            println!("  • For Intel: Install mesa-va-drivers or intel-media-driver");
            println!("  • For AMD: Install mesa-va-drivers");
            println!("  • For NVIDIA: Install nvidia-utils and nvidia-vaapi-driver");
        }
    }

    // Check VDPAU
    match std::process::Command::new("vdpauinfo").output() {
        Ok(output) => {
            if output.status.success() {
                info!("  ✓ VDPAU available");
            } else {
                info!("  ℹ VDPAU not available (VA-API is preferred)");
            }
        }
        Err(_) => {
            info!("  ℹ vdpauinfo not found (optional)");
        }
    }
}

use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

mod config;
mod core;
mod ctl;
mod we;

#[cfg(feature = "backend-wayland")]
mod backend;

#[cfg(feature = "video-mpv")]
mod video;

#[derive(Parser)]
#[command(name = "wayvid")]
#[command(about = "Dynamic video wallpaper engine for Wayland", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Log level (trace, debug, info, warn, error)
    #[arg(long, default_value = "info", global = true)]
    log_level: String,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the wallpaper engine
    Run {
        /// Path to configuration file
        #[arg(short, long, default_value = "~/.config/wayvid/config.yaml")]
        config: String,
    },
    /// Check system capabilities
    Check,
    /// Import Wallpaper Engine project
    Import {
        /// Path to Wallpaper Engine project directory
        project_dir: String,

        /// Output config file path (optional, defaults to stdout)
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Reload configuration (via IPC, future)
    #[cfg(feature = "ipc")]
    Reload,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    let level = match cli.log_level.to_lowercase().as_str() {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "info" => Level::INFO,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => Level::INFO,
    };

    let subscriber = FmtSubscriber::builder()
        .with_max_level(level)
        .with_target(true)
        .with_thread_ids(false)
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

    info!("wayvid version {}", env!("CARGO_PKG_VERSION"));

    match cli.command {
        Commands::Run { config } => {
            let config_path = shellexpand::tilde(&config).to_string();
            info!("Loading configuration from: {}", config_path);

            #[cfg(feature = "backend-wayland")]
            {
                let cfg = config::Config::from_file(&config_path)?;
                let path = std::path::PathBuf::from(config_path);
                backend::wayland::run(cfg, Some(path))?;
            }

            #[cfg(not(feature = "backend-wayland"))]
            {
                anyhow::bail!("No backend enabled. Please compile with --features backend-wayland");
            }
        }
        Commands::Check => {
            ctl::check::run_capability_check()?;
        }
        Commands::Import { project_dir, output } => {
            use std::fs;
            use std::path::Path;

            info!("🔍 Importing Wallpaper Engine project from: {}", project_dir);

            let project_path = Path::new(&project_dir);

            // Detect project
            let project_file = we::detect_we_project(project_path)?;

            // Parse project
            let (project, video_path) = we::parse_we_project(&project_file)?;

            // Generate config
            let config_yaml = we::converter::generate_config_with_metadata(&project, video_path)?;

            // Output
            if let Some(output_path) = output {
                let output_path = shellexpand::tilde(&output_path).to_string();
                fs::write(&output_path, &config_yaml)?;
                info!("✅ Config written to: {}", output_path);
            } else {
                println!("{}", config_yaml);
            }

            info!("🎉 Import completed successfully");
        }
        #[cfg(feature = "ipc")]
        Commands::Reload => {
            info!("Reloading configuration...");
            use ctl::protocol::IpcCommand;
            ctl::ipc_server::send_command(&IpcCommand::ReloadConfig)?;
        }
    }

    Ok(())
}

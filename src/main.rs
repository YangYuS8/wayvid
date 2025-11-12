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
    /// Workshop commands
    Workshop {
        #[command(subcommand)]
        command: WorkshopCommands,
    },
    /// Reload configuration (via IPC, future)
    #[cfg(feature = "ipc")]
    Reload,
}

#[derive(Subcommand)]
enum WorkshopCommands {
    /// List local Workshop items (from Steam)
    List,
    /// Show item details
    Info {
        /// Workshop item ID
        id: u64,
    },
    /// Search Workshop items online
    Search {
        /// Search query
        query: String,
        /// Page number (default: 1)
        #[arg(short, long, default_value = "1")]
        page: u32,
    },
    /// Download Workshop item from Steam
    Download {
        /// Workshop item ID
        id: u64,
    },
    /// Install Workshop item (download + import)
    Install {
        /// Workshop item ID
        id: u64,
        /// Output config file
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Import Workshop item to config (local or cached)
    Import {
        /// Workshop item ID
        id: u64,
        /// Output config file
        #[arg(short, long)]
        output: Option<String>,
    },
    /// List cached downloads
    Cache {
        /// Clear cache
        #[arg(short, long)]
        clear: bool,
        /// Clear specific item
        #[arg(long)]
        clear_item: Option<u64>,
    },
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
                anyhow::bail!(
                    "❌ No backend enabled\n\n\
                     wayvid requires Wayland backend support to run.\n\
                     Please rebuild with: cargo build --features backend-wayland\n\n\
                     Or if using a package manager, install the correct package variant."
                );
            }
        }
        Commands::Check => {
            ctl::check::run_capability_check()?;
        }
        Commands::Import {
            project_dir,
            output,
        } => {
            use std::fs;
            use std::path::Path;

            info!(
                "🔍 Importing Wallpaper Engine project from: {}",
                project_dir
            );

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
        Commands::Workshop { command } => {
            use we::{SteamLibrary, WorkshopDownloader, WorkshopScanner, WALLPAPER_ENGINE_APP_ID};

            match command {
                WorkshopCommands::List => {
                    info!("🔍 Scanning local Steam Workshop items...");

                    let steam = SteamLibrary::discover()?;
                    let paths = steam.find_workshop_items(WALLPAPER_ENGINE_APP_ID)?;
                    let scanner = WorkshopScanner::scan(&paths)?;

                    println!("\n📦 Found {} Workshop items:\n", scanner.items().len());
                    for item in scanner.items() {
                        println!("  [{}] {}", item.id, item.title());
                        if let Some(path) = item.video_path() {
                            println!("      📁 {}", path.display());
                        }
                        println!();
                    }
                }
                WorkshopCommands::Info { id } => {
                    let steam = SteamLibrary::discover()?;
                    let paths = steam.find_workshop_items(WALLPAPER_ENGINE_APP_ID)?;
                    let scanner = WorkshopScanner::scan(&paths)?;

                    let item = scanner
                        .find(id)
                        .ok_or_else(|| anyhow::anyhow!("Workshop item {} not found", id))?;

                    println!("\n📦 Workshop Item {}\n", id);
                    println!("Title: {}", item.title());
                    println!("Path:  {}", item.path.display());
                    if let Some(video) = item.video_path() {
                        println!("Video: {}", video.display());
                    }
                    if let Some(ref proj) = item.project {
                        if let Some(ref desc) = &proj.description {
                            println!("\nDescription:\n{}", desc);
                        }
                    }
                }
                WorkshopCommands::Search { query, page } => {
                    println!("\n🔍 Workshop Search\n");
                    println!("ℹ️  Steam's public API doesn't support direct search.");
                    println!("   Please use one of these methods:\n");
                    
                    println!("📌 Method 1: Browse Workshop in Steam");
                    println!("   1. Open Steam Workshop: https://steamcommunity.com/app/431960/workshop/");
                    println!("   2. Search for: {}", query);
                    println!("   3. Find item ID in URL (e.g., id=123456789)");
                    println!("   4. Download: wayvid workshop download <id>\n");
                    
                    println!("📌 Method 2: Subscribe in Steam Client");
                    println!("   1. Subscribe to items in Steam");
                    println!("   2. Run: wayvid workshop list");
                    println!("   3. Import: wayvid workshop import <id>\n");
                    
                    println!("📌 Method 3: Use Item ID Directly");
                    println!("   wayvid workshop install <id> -o ~/.config/wayvid/config.yaml\n");
                    
                    println!("💡 Popular wallpapers can be found at:");
                    println!("   https://steamcommunity.com/app/431960/workshop/?browsesort=trend");
                    
                    // Prevent unused variable warnings
                    let _ = (query, page);
                }
                WorkshopCommands::Download { id } => {
                    info!("⬇️  Downloading Workshop item {}...", id);

                    let downloader = WorkshopDownloader::new()?;
                    let item_dir = downloader.download(id)?;

                    println!("\n✅ Downloaded successfully!");
                    println!("📁 Location: {}", item_dir.display());
                    println!("\n💡 To import: wayvid workshop import {}", id);
                }
                WorkshopCommands::Install { id, output } => {
                    info!("🚀 Installing Workshop item {}...", id);

                    let downloader = WorkshopDownloader::new()?;
                    
                    // Download if not cached
                    let item_dir = if downloader.list_cached()?.contains(&id) {
                        info!("📦 Using cached item");
                        downloader.cache_dir().join(id.to_string())
                    } else {
                        info!("⬇️  Downloading...");
                        downloader.download(id)?
                    };

                    // Parse and import
                    info!("🔄 Importing configuration...");
                    let project_file = we::detect_we_project(&item_dir)?;
                    let (project, video_path) = we::parse_we_project(&project_file)?;
                    let config_yaml = we::converter::generate_config_with_metadata(&project, video_path)?;

                    if let Some(output_path) = output {
                        let output_path = shellexpand::tilde(&output_path).to_string();
                        std::fs::write(&output_path, &config_yaml)?;
                        println!("\n✅ Installed successfully!");
                        println!("📄 Config: {}", output_path);
                    } else {
                        println!("\n✅ Configuration generated:\n");
                        println!("{}", config_yaml);
                        println!("\n💡 To save: wayvid workshop install {} -o ~/.config/wayvid/config.yaml", id);
                    }
                }
                WorkshopCommands::Import { id, output } => {
                    info!("🔍 Importing Workshop item {}...", id);

                    let steam = SteamLibrary::discover()?;
                    let paths = steam.find_workshop_items(WALLPAPER_ENGINE_APP_ID)?;
                    let scanner = WorkshopScanner::scan(&paths)?;

                    let item = scanner
                        .find(id)
                        .ok_or_else(|| anyhow::anyhow!("Workshop item {} not found", id))?;

                    let project = item
                        .project
                        .as_ref()
                        .ok_or_else(|| anyhow::anyhow!("Invalid project"))?;

                    let video_path = item
                        .video_path()
                        .ok_or_else(|| anyhow::anyhow!("No video file"))?;

                    let config_yaml =
                        we::converter::generate_config_with_metadata(project, video_path)?;

                    if let Some(output_path) = output {
                        let output_path = shellexpand::tilde(&output_path).to_string();
                        std::fs::write(&output_path, &config_yaml)?;
                        info!("✅ Config written to: {}", output_path);
                    } else {
                        println!("{}", config_yaml);
                    }

                    info!("🎉 Import completed");
                }
                WorkshopCommands::Cache { clear, clear_item } => {
                    let downloader = WorkshopDownloader::new()?;

                    if let Some(item_id) = clear_item {
                        info!("🗑️  Clearing cache for item {}...", item_id);
                        downloader.clear_cache(item_id)?;
                        println!("✅ Cache cleared for item {}", item_id);
                    } else if clear {
                        info!("🗑️  Clearing all cached downloads...");
                        downloader.clear_all_cache()?;
                        println!("✅ All cache cleared");
                    } else {
                        // List cached items
                        let cached = downloader.list_cached()?;
                        
                        if cached.is_empty() {
                            println!("\n📦 No cached downloads");
                        } else {
                            println!("\n📦 Cached downloads ({}):\n", cached.len());
                            for item_id in cached {
                                println!("  [{}]", item_id);
                            }
                            println!("\n💡 To clear all: wayvid workshop cache --clear");
                            println!("💡 To clear one: wayvid workshop cache --clear-item <id>");
                        }
                        
                        println!("\n📁 Cache location: {}", downloader.cache_dir().display());
                    }
                }
            }
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

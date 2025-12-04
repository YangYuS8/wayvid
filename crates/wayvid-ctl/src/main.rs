//! wayvid-ctl: Command-line control tool for wayvid
//!
//! This is a simplified CLI for scripting and automation:
//! - apply: Apply a wallpaper
//! - pause: Pause playback
//! - resume: Resume playback
//! - status: Show current status
//!
//! This is a placeholder for Phase 1 - full implementation in Phase 5

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "wayvid-ctl")]
#[command(about = "wayvid command-line control tool")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Apply a wallpaper
    Apply {
        /// Wallpaper path
        path: String,
        /// Target output (optional, default: all)
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Pause wallpaper playback
    Pause,
    /// Resume wallpaper playback
    Resume,
    /// Show current status
    Status {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Apply { path, output } => {
            println!("Apply wallpaper: {} (output: {:?})", path, output);
            println!("TODO: Implement IPC to wayvid GUI");
        }
        Commands::Pause => {
            println!("Pause wallpaper");
            println!("TODO: Implement IPC to wayvid GUI");
        }
        Commands::Resume => {
            println!("Resume wallpaper");
            println!("TODO: Implement IPC to wayvid GUI");
        }
        Commands::Status { json } => {
            if json {
                println!(r#"{{"status": "placeholder"}}"#);
            } else {
                println!("Status: placeholder");
            }
            println!("TODO: Implement IPC to wayvid GUI");
        }
    }
}

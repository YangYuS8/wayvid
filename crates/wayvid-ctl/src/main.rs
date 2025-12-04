//! wayvid-ctl: Command-line control tool for wayvid
//!
//! A lightweight CLI for controlling the wayvid wallpaper daemon:
//! - apply: Apply a wallpaper to one or all outputs
//! - pause: Pause wallpaper playback
//! - resume: Resume wallpaper playback
//! - stop: Stop wallpaper playback
//! - status: Show current daemon status
//! - list: List available outputs
//!
//! Communicates with wayvid daemon via Unix socket IPC.

mod ipc;

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::Colorize;

use ipc::{IpcClient, Response};

#[derive(Parser)]
#[command(name = "wayvid-ctl")]
#[command(about = "wayvid command-line control tool")]
#[command(version)]
#[command(after_help = "Examples:
  wayvid-ctl apply ~/Videos/wallpaper.mp4
  wayvid-ctl apply ~/Pictures/bg.jpg --output DP-1
  wayvid-ctl pause
  wayvid-ctl status --json")]
struct Cli {
    /// Enable verbose logging
    #[arg(short, long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Apply a wallpaper to display(s)
    Apply {
        /// Path to wallpaper file or directory
        path: PathBuf,
        /// Target output name (default: all outputs)
        #[arg(short, long)]
        output: Option<String>,
        /// Fill mode: fill, contain, stretch, centre
        #[arg(short, long, default_value = "fill")]
        mode: String,
    },
    /// Pause wallpaper playback
    Pause {
        /// Target output (default: all)
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Resume wallpaper playback
    Resume {
        /// Target output (default: all)
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Stop wallpaper playback completely
    Stop {
        /// Target output (default: all)
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Show current daemon status
    Status {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// List available outputs/monitors
    Outputs {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Reload configuration
    Reload,
    /// Check if daemon is running
    Ping,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Setup logging
    if cli.verbose {
        tracing_subscriber::fmt()
            .with_env_filter("wayvid_ctl=debug")
            .init();
    }

    // Create IPC client
    let client = IpcClient::new();

    // Execute command
    match cli.command {
        Commands::Apply { path, output, mode } => {
            let path = path.canonicalize().context("Invalid wallpaper path")?;

            if !path.exists() {
                anyhow::bail!("Wallpaper not found: {}", path.display());
            }

            let response = client.apply(&path, output.as_deref(), &mode)?;
            handle_response(response, cli.verbose)?;
        }

        Commands::Pause { output } => {
            let response = client.pause(output.as_deref())?;
            handle_response(response, cli.verbose)?;
        }

        Commands::Resume { output } => {
            let response = client.resume(output.as_deref())?;
            handle_response(response, cli.verbose)?;
        }

        Commands::Stop { output } => {
            let response = client.stop(output.as_deref())?;
            handle_response(response, cli.verbose)?;
        }

        Commands::Status { json } => {
            let response = client.status()?;
            if json {
                println!("{}", serde_json::to_string_pretty(&response)?);
            } else {
                print_status(&response);
            }
        }

        Commands::Outputs { json } => {
            let response = client.outputs()?;
            if json {
                println!("{}", serde_json::to_string_pretty(&response)?);
            } else {
                print_outputs(&response);
            }
        }

        Commands::Reload => {
            let response = client.reload()?;
            handle_response(response, cli.verbose)?;
        }

        Commands::Ping => match client.ping() {
            Ok(_) => {
                println!("{} Daemon is running", "✓".green());
            }
            Err(e) => {
                println!("{} Daemon not responding: {}", "✗".red(), e);
                std::process::exit(1);
            }
        },
    }

    Ok(())
}

fn handle_response(response: Response, verbose: bool) -> Result<()> {
    match response {
        Response::Ok { message } => {
            if verbose || message.is_some() {
                println!(
                    "{} {}",
                    "✓".green(),
                    message.unwrap_or_else(|| "Success".to_string())
                );
            }
        }
        Response::Error { error } => {
            println!("{} {}", "✗".red(), error);
            std::process::exit(1);
        }
        _ => {}
    }
    Ok(())
}

fn print_status(response: &Response) {
    if let Response::Status {
        running,
        outputs,
        version,
    } = response
    {
        println!("Wayvid Daemon Status");
        println!("====================");
        println!(
            "Status:  {}",
            if *running {
                "Running".green()
            } else {
                "Stopped".red()
            }
        );
        if let Some(v) = version {
            println!("Version: {}", v);
        }
        println!();

        if outputs.is_empty() {
            println!("No active outputs");
        } else {
            println!("Active Outputs:");
            for output in outputs {
                let status = if output.paused {
                    "Paused".yellow()
                } else {
                    "Playing".green()
                };
                println!("  {} [{}]", output.name.bold(), status);
                if let Some(ref wp) = output.wallpaper {
                    println!("    Wallpaper: {}", wp);
                }
            }
        }
    }
}

fn print_outputs(response: &Response) {
    if let Response::Outputs { outputs } = response {
        if outputs.is_empty() {
            println!("No outputs detected");
            return;
        }

        println!("Available Outputs:");
        for output in outputs {
            println!(
                "  {} {}x{} @ {}Hz",
                output.name.bold(),
                output.width,
                output.height,
                output.refresh.unwrap_or(60)
            );
            if let Some(ref make) = output.make {
                println!("    Make: {}", make);
            }
            if let Some(ref model) = output.model {
                println!("    Model: {}", model);
            }
        }
    }
}

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

use wayvid::core::types::VideoSource;
use wayvid::ctl::ipc_server::send_command;
use wayvid::ctl::protocol::{IpcCommand, IpcResponse};

#[derive(Parser)]
#[command(name = "wayvid-ctl")]
#[command(version, about = "Control wayvid daemon", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Get current status
    Status,

    /// Pause playback
    Pause {
        /// Output name (optional, pause all if not specified)
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Resume playback
    Resume {
        /// Output name (optional, resume all if not specified)
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Seek to specific time
    Seek {
        /// Output name
        #[arg(short, long)]
        output: String,

        /// Time in seconds
        time: f64,
    },

    /// Switch video source
    Switch {
        /// Output name
        #[arg(short, long)]
        output: String,

        /// New video source (file path or URL)
        source: String,
    },

    /// Reload configuration
    Reload,

    /// Set playback rate (speed)
    Rate {
        /// Output name
        #[arg(short, long)]
        output: String,

        /// Playback rate (1.0 = normal)
        rate: f64,
    },

    /// Set volume
    Volume {
        /// Output name
        #[arg(short, long)]
        output: String,

        /// Volume level (0.0 - 1.0)
        volume: f64,
    },

    /// Toggle mute
    Mute {
        /// Output name
        #[arg(short, long)]
        output: String,
    },

    /// Set layout mode
    Layout {
        /// Output name
        #[arg(short, long)]
        output: String,

        /// Layout mode (fill, contain, stretch, cover, centre)
        mode: String,
    },

    /// Quit the daemon
    Quit,
}

/// Parse a source string into VideoSource
/// Supports:
/// - file:///path/to/video.mp4
/// - /path/to/video.mp4 (assumes file)
/// - http://... or https://...
/// - rtsp://...
/// - pipe:///path or pipe:// (stdin)
fn parse_video_source(source: &str) -> Result<VideoSource> {
    if source.starts_with("file://") {
        Ok(VideoSource::File {
            path: source.strip_prefix("file://").unwrap().to_string(),
        })
    } else if source.starts_with("http://") || source.starts_with("https://") {
        Ok(VideoSource::Url {
            url: source.to_string(),
        })
    } else if source.starts_with("rtsp://") {
        Ok(VideoSource::Rtsp {
            url: source.to_string(),
        })
    } else if source.starts_with("pipe://") {
        let path = source.strip_prefix("pipe://").unwrap().to_string();
        Ok(VideoSource::Pipe { path })
    } else if source.starts_with('/') || source.contains("./") {
        // Assume local file path
        Ok(VideoSource::File {
            path: source.to_string(),
        })
    } else {
        anyhow::bail!("Invalid source format. Use file://, http://, https://, rtsp://, pipe://, or absolute path");
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let command = match cli.command {
        Commands::Status => IpcCommand::GetStatus,
        Commands::Pause { output } => IpcCommand::Pause { output },
        Commands::Resume { output } => IpcCommand::Resume { output },
        Commands::Seek { output, time } => IpcCommand::Seek { output, time },
        Commands::Switch { output, source } => {
            let video_source = parse_video_source(&source)
                .context("Failed to parse video source")?;
            IpcCommand::SwitchSource {
                output,
                source: video_source,
            }
        }
        Commands::Reload => IpcCommand::ReloadConfig,
        Commands::Rate { output, rate } => IpcCommand::SetPlaybackRate { output, rate },
        Commands::Volume { output, volume } => IpcCommand::SetVolume { output, volume },
        Commands::Mute { output } => IpcCommand::ToggleMute { output },
        Commands::Layout { output, mode } => IpcCommand::SetLayout {
            output,
            layout: mode,
        },
        Commands::Quit => IpcCommand::Quit,
    };

    let response = send_command(&command).context("Failed to send command to daemon")?;

    match response {
        IpcResponse::Success { data } => {
            if let Some(data) = data {
                println!("{}", serde_json::to_string_pretty(&data)?);
            } else {
                println!("✓ Command executed successfully");
            }
        }
        IpcResponse::Error { message } => {
            eprintln!("✗ Error: {}", message);
            std::process::exit(1);
        }
    }

    Ok(())
}

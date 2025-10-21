use anyhow::Result;
use std::ffi::CString;
use tracing::{error, info};
use wayland_client::protocol::wl_surface;
use wayland_client::QueueHandle;
use wayland_protocols_wlr::layer_shell::v1::client::{zwlr_layer_shell_v1, zwlr_layer_surface_v1};

use crate::config::EffectiveConfig;
use crate::core::types::OutputInfo;

#[cfg(feature = "video-mpv")]
use crate::video::mpv::MpvPlayer;

pub struct WaylandSurface {
    pub wl_surface: wl_surface::WlSurface,
    pub layer_surface: zwlr_layer_surface_v1::ZwlrLayerSurfaceV1,
    pub output_info: OutputInfo,
    pub config: EffectiveConfig,

    // Video player
    #[cfg(feature = "video-mpv")]
    player: Option<MpvPlayer>,

    configured: bool,
    initial_configure_done: bool,
}

impl WaylandSurface {
    pub fn new(
        wl_surface: wl_surface::WlSurface,
        layer_shell: &zwlr_layer_shell_v1::ZwlrLayerShellV1,
        output_info: OutputInfo,
        config: EffectiveConfig,
        wl_output: &wayland_client::protocol::wl_output::WlOutput,
        qh: &QueueHandle<crate::backend::wayland::app::AppState>,
    ) -> Result<Self> {
        // Create layer surface
        let layer_surface = layer_shell.get_layer_surface(
            &wl_surface,
            Some(wl_output),
            zwlr_layer_shell_v1::Layer::Background,
            CString::new("wayvid").unwrap().into_string().unwrap(),
            qh,
            (),
        );

        // Configure layer surface for input passthrough
        layer_surface.set_exclusive_zone(0);
        layer_surface
            .set_keyboard_interactivity(zwlr_layer_surface_v1::KeyboardInteractivity::None);

        // Set size to output dimensions
        layer_surface.set_size(output_info.width as u32, output_info.height as u32);

        wl_surface.commit();

        info!(
            "Created layer surface for output {} ({}x{})",
            output_info.name, output_info.width, output_info.height
        );

        Ok(Self {
            wl_surface,
            layer_surface,
            output_info,
            config,
            #[cfg(feature = "video-mpv")]
            player: None,
            configured: false,
            initial_configure_done: false,
        })
    }

    pub fn configure(&mut self, width: u32, height: u32, serial: u32) {
        // Only log and process first configure
        let is_first = !self.initial_configure_done;
        
        if is_first {
            info!(
                "Initial configure for surface {} to {}x{}",
                self.output_info.name, width, height
            );
            self.initial_configure_done = true;
        }

        self.output_info.width = width as i32;
        self.output_info.height = height as i32;
        self.configured = true;

        // Initialize player after first configuration
        // TODO: Fix libmpv version mismatch before enabling
        // #[cfg(feature = "video-mpv")]
        // {
        //     if is_first && self.player.is_none() {
        //         match self.init_player() {
        //             Ok(()) => info!("Player initialized for {}", self.output_info.name),
        //             Err(e) => error!("Failed to initialize player: {}", e),
        //         }
        //     }
        // }

        self.layer_surface.ack_configure(serial);
        
        // Only commit on initial configure to avoid loops
        if is_first {
            self.wl_surface.commit();
        }
    }

    #[cfg(feature = "video-mpv")]
    fn init_player(&mut self) -> Result<()> {
        let player = MpvPlayer::new(&self.config, &self.output_info)?;
        self.player = Some(player);
        Ok(())
    }

    pub fn render(&mut self) -> Result<()> {
        if !self.configured {
            return Ok(());
        }

        // Render video
        #[cfg(feature = "video-mpv")]
        {
            if let Some(ref mut player) = self.player {
                if let Err(e) = player.render() {
                    error!("Render error: {}", e);
                }
            }
        }

        self.wl_surface.commit();
        Ok(())
    }

    pub fn destroy(&mut self) {
        self.layer_surface.destroy();
        self.wl_surface.destroy();
        info!("Destroyed surface for output {}", self.output_info.name);
    }
}

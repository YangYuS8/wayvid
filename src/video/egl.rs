use anyhow::{anyhow, Context, Result};
use khronos_egl as egl;
use wayland_client::protocol::wl_surface::WlSurface;
use wayland_client::Proxy;
use wayland_egl as wegl;

/// EGL context manager for OpenGL rendering on Wayland
pub struct EglContext {
    display: egl::Display,
    config: egl::Config,
    context: egl::Context,
    instance: egl::DynamicInstance<egl::EGL1_4>,
}

/// Per-surface EGL window
pub struct EglWindow {
    egl_window: wegl::WlEglSurface,
    egl_surface: egl::Surface,
    width: i32,
    height: i32,
}

impl EglContext {
    /// Initialize EGL display and create OpenGL context
    pub fn new(wl_display: *mut std::ffi::c_void) -> Result<Self> {
        // 1. Load EGL library
        let instance = unsafe {
            egl::DynamicInstance::<egl::EGL1_4>::load_required()
                .context("Failed to load EGL library")?
        };

        // 2. Get EGL display from Wayland display
        let display = unsafe {
            instance
                .get_display(wl_display as egl::NativeDisplayType)
                .context("Failed to get EGL display")?
        };

        // 3. Initialize EGL
        let (major, minor) = instance
            .initialize(display)
            .context("Failed to initialize EGL")?;

        tracing::info!("EGL initialized: {}.{}", major, minor);

        // 4. Bind OpenGL API
        instance
            .bind_api(egl::OPENGL_API)
            .context("Failed to bind OpenGL API")?;

        // 5. Choose EGL config
        let config_attribs = [
            egl::SURFACE_TYPE,
            egl::WINDOW_BIT,
            egl::RENDERABLE_TYPE,
            egl::OPENGL_BIT,
            egl::RED_SIZE,
            8,
            egl::GREEN_SIZE,
            8,
            egl::BLUE_SIZE,
            8,
            egl::ALPHA_SIZE,
            8,
            egl::DEPTH_SIZE,
            24,
            egl::STENCIL_SIZE,
            8,
            egl::NONE,
        ];

        let configs = instance
            .choose_first_config(display, &config_attribs)
            .context("Failed to choose EGL config")?
            .ok_or_else(|| anyhow!("No suitable EGL config found"))?;

        tracing::debug!("EGL config selected");

        // 6. Create OpenGL context
        let context_attribs = [
            egl::CONTEXT_MAJOR_VERSION,
            3,
            egl::CONTEXT_MINOR_VERSION,
            0,
            egl::CONTEXT_OPENGL_PROFILE_MASK,
            egl::CONTEXT_OPENGL_CORE_PROFILE_BIT,
            egl::NONE,
        ];

        let context = instance
            .create_context(display, configs, None, &context_attribs)
            .context("Failed to create EGL context")?;

        tracing::info!("EGL context created successfully");

        Ok(Self {
            display,
            config: configs,
            context,
            instance,
        })
    }

    /// Create EGL window surface for a Wayland surface
    pub fn create_window(
        &self,
        wl_surface: &WlSurface,
        width: i32,
        height: i32,
    ) -> Result<EglWindow> {
        // 1. Create Wayland EGL window
        // Extract the ObjectId from the WlSurface proxy
        let surface_id = wl_surface.id();

        // Create wegl surface using ObjectId directly
        let egl_window = wegl::WlEglSurface::new(surface_id, width, height)
            .context("Failed to create wl_egl_window")?;

        // 2. Create EGL window surface
        let egl_surface = unsafe {
            self.instance
                .create_window_surface(
                    self.display,
                    self.config,
                    egl_window.ptr() as egl::NativeWindowType,
                    None,
                )
                .context("Failed to create EGL window surface")?
        };

        tracing::debug!("EGL window surface created: {}x{}", width, height);

        Ok(EglWindow {
            egl_window,
            egl_surface,
            width,
            height,
        })
    }

    /// Make this context current for rendering
    pub fn make_current(&self, window: &EglWindow) -> Result<()> {
        self.instance
            .make_current(
                self.display,
                Some(window.egl_surface),
                Some(window.egl_surface),
                Some(self.context),
            )
            .context("Failed to make EGL context current")?;
        Ok(())
    }

    /// Swap buffers to display rendered frame
    pub fn swap_buffers(&self, window: &EglWindow) -> Result<()> {
        self.instance
            .swap_buffers(self.display, window.egl_surface)
            .context("Failed to swap EGL buffers")?;
        Ok(())
    }
}

impl EglWindow {
    /// Resize the EGL window
    pub fn resize(&mut self, width: i32, height: i32) -> Result<()> {
        if self.width == width && self.height == height {
            return Ok(());
        }

        // Resize the Wayland EGL window
        self.egl_window.resize(width, height, 0, 0);
        self.width = width;
        self.height = height;

        tracing::debug!("EGL window resized to {}x{}", width, height);
        Ok(())
    }
}

impl Drop for EglWindow {
    fn drop(&mut self) {
        tracing::debug!("Dropping EGL window");
        // egl_surface and egl_window will be automatically cleaned up
    }
}

impl Drop for EglContext {
    fn drop(&mut self) {
        tracing::debug!("Dropping EGL context");
        // EGL context and display will be automatically cleaned up by khronos-egl
    }
}

// Safety: EGL contexts are thread-safe
unsafe impl Send for EglContext {}
unsafe impl Sync for EglContext {}

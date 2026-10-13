//! Per-output playback with CPU-prepared Scene resources and a shared EGL surface.

use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use lwe_core::OutputInfo;
use wayland_client::protocol::wl_surface::WlSurface;

use super::{EngineRenderMetrics, WallpaperSource};
use crate::egl::{EglContext, EglWindow};
use crate::mpv::{MpvPlayer, VideoConfig};
use crate::scene::shared_texture::{ResourcePlan, TexturePool};
use crate::scene::{SceneAudio, SceneRenderer};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackState {
    Stopped,
    Playing,
    Paused,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum FrameDemand {
    Idle,
    Continuous,
    Urgent,
}

fn frame_demand(
    state: PlaybackState,
    first_frame: bool,
    redraw: bool,
    resized: bool,
    continuous: bool,
) -> FrameDemand {
    if first_frame || redraw || resized {
        FrameDemand::Urgent
    } else if state == PlaybackState::Playing && continuous {
        FrameDemand::Continuous
    } else {
        FrameDemand::Idle
    }
}

#[derive(Debug)]
struct SceneClock {
    elapsed: Duration,
    started: Option<Instant>,
}

impl SceneClock {
    fn new() -> Self {
        Self {
            elapsed: Duration::ZERO,
            started: Some(Instant::now()),
        }
    }

    fn seconds(&self) -> f64 {
        (self.elapsed + self.started.map(|t| t.elapsed()).unwrap_or_default()).as_secs_f64()
    }

    fn pause(&mut self) {
        if let Some(started) = self.started.take() {
            self.elapsed += started.elapsed();
        }
    }

    fn resume(&mut self) {
        if self.started.is_none() {
            self.started = Some(Instant::now());
        }
    }
}

enum RendererInstance {
    Video(MpvPlayer),
    Scene {
        renderer: Box<SceneRenderer>,
        clock: SceneClock,
        animated: bool,
        drawn: bool,
        audio: SceneAudio,
        audio_error: Option<String>,
    },
}

impl RendererInstance {
    fn allocation_plan(&self) -> Result<ResourcePlan> {
        match self {
            Self::Video(_) => Ok(ResourcePlan::default()),
            Self::Scene { renderer, .. } => renderer.allocation_plan(),
        }
    }

    fn additional_gpu_bytes(&self, width: i32, height: i32) -> Result<u64> {
        match self {
            Self::Video(_) => Ok(0),
            Self::Scene { renderer, .. } => renderer.additional_gpu_bytes(width, height),
        }
    }

    fn continuous(&self) -> bool {
        match self {
            Self::Video(_) => true,
            Self::Scene { animated, .. } => *animated,
        }
    }

    fn create(
        source: &WallpaperSource,
        config: &VideoConfig,
        output: &OutputInfo,
        egl: &EglContext,
        texture_pool: Rc<TexturePool>,
    ) -> Result<Self> {
        match source {
            WallpaperSource::Video { path } => {
                let mut config = config.clone();
                config.source = path.to_string_lossy().into_owned();
                config.volume = 0.0;
                let mut player = MpvPlayer::new(&config, output)?;
                player.init_render_context(egl)?;
                Ok(Self::Video(player))
            }
            WallpaperSource::Scene { prepared, .. } => Ok(Self::Scene {
                renderer: Box::new(prepared.initialize_shared(texture_pool)?),
                clock: SceneClock::new(),
                animated: prepared.summary().animated,
                drawn: false,
                audio: prepared.initialize_audio()?,
                audio_error: None,
            }),
        }
    }

    fn render(
        &mut self,
        width: i32,
        height: i32,
        force: bool,
        resized: bool,
        redraw: bool,
    ) -> Result<bool> {
        match self {
            Self::Video(player) => {
                let ready = player.consume_frame_update(force);
                // libmpv can redraw its previous frame even while paused.
                if !ready && !redraw && !resized {
                    return Ok(false);
                }
                unsafe {
                    gl::ClearColor(0.0, 0.0, 0.0, 1.0);
                    gl::Clear(gl::COLOR_BUFFER_BIT);
                    gl::Viewport(0, 0, width, height);
                }
                player.render(width, height, 0)
            }
            Self::Scene {
                renderer,
                clock,
                animated,
                drawn,
                audio,
                ..
            } => {
                if !audio.poll_ready()? {
                    return Ok(false);
                }
                if *drawn && !*animated && !force && !resized && !redraw {
                    return Ok(false);
                }
                let rendered = if redraw || force || resized {
                    renderer.render_forced(width, height, clock.seconds())?
                } else {
                    renderer.render(width, height, clock.seconds())?
                };
                *drawn |= rendered;
                Ok(rendered)
            }
        }
    }

    fn report_swap(&self) {
        if let Self::Video(player) = self {
            player.report_swap();
        }
    }

    fn pause(&mut self) {
        match self {
            Self::Video(player) => {
                let _ = player.pause();
            }
            Self::Scene {
                renderer,
                clock,
                audio,
                audio_error,
                ..
            } => {
                clock.pause();
                renderer.pause();
                if let Err(error) = audio.set_paused(true) {
                    *audio_error = Some(format!("Cannot pause Scene audio: {error:#}"));
                }
            }
        }
    }

    fn resume(&mut self) {
        match self {
            Self::Video(player) => {
                let _ = player.resume();
            }
            Self::Scene {
                renderer,
                clock,
                audio,
                audio_error,
                ..
            } => {
                clock.resume();
                renderer.resume();
                if let Err(error) = audio.set_paused(false) {
                    *audio_error = Some(format!("Cannot resume Scene audio: {error:#}"));
                }
            }
        }
    }

    fn volume(&mut self, volume: f32) {
        match self {
            Self::Video(player) => {
                let _ = player.set_volume(volume as f64);
            }
            Self::Scene {
                audio, audio_error, ..
            } => {
                if let Err(error) = audio.set_master_volume(volume as f64) {
                    *audio_error = Some(format!("Cannot update Scene audio volume: {error:#}"));
                }
            }
        }
    }

    fn deactivate_audio(&mut self) -> Result<()> {
        if let Self::Scene {
            renderer, audio, ..
        } = self
        {
            audio.deactivate()?;
            renderer.deactivate_analysis();
        }
        Ok(())
    }

    fn freeze_for_rollback(&mut self) {
        match self {
            Self::Video(_) => {
                self.volume(0.0);
                self.pause();
            }
            Self::Scene {
                renderer, clock, ..
            } => {
                clock.pause();
                renderer.pause();
            }
        }
    }

    fn rollback_audio_pause(&mut self, paused: bool) {
        if let Self::Scene {
            audio, audio_error, ..
        } = self
            && let Err(error) = audio.set_paused(paused)
        {
            *audio_error = Some(format!(
                "Cannot update rollback Scene audio pause: {error:#}"
            ));
        }
    }

    fn activate_audio(&mut self, paused: bool, volume: f32) -> Result<()> {
        if let Self::Scene {
            renderer, audio, ..
        } = self
        {
            audio.activate(paused, volume as f64)?;
            renderer.activate_analysis(paused);
        }
        Ok(())
    }

    fn poll_audio_health(&mut self) -> Result<()> {
        if let Self::Scene {
            audio, audio_error, ..
        } = self
        {
            if let Some(error) = audio_error.as_ref() {
                anyhow::bail!("{error}");
            }
            if let Err(error) = audio.poll_ready() {
                *audio_error = Some(format!("Scene audio failed: {error:#}"));
                return Err(error);
            }
        }
        Ok(())
    }
}

/// A renderer and a replaceable candidate share one LWE-owned output surface.
pub struct WallpaperSession {
    output_info: OutputInfo,
    source: WallpaperSource,
    video_config: VideoConfig,
    renderer: Option<RendererInstance>,
    pending: Option<(WallpaperSource, RendererInstance)>,
    rollback: Option<(WallpaperSource, RendererInstance)>,
    egl_window: Option<EglWindow>,
    texture_pool: Option<Rc<TexturePool>>,
    state: PlaybackState,
    manual_paused: bool,
    power_paused: bool,
    volume: f32,
    volume_override: Option<f32>,
    gl_loaded: bool,
    redraw_needed: bool,
    metrics: EngineRenderMetrics,
}

impl WallpaperSession {
    pub fn new(path: PathBuf, output_info: OutputInfo, video_config: VideoConfig) -> Result<Self> {
        Self::from_source(WallpaperSource::Video { path }, output_info, video_config)
    }

    pub fn from_source(
        source: WallpaperSource,
        output_info: OutputInfo,
        video_config: VideoConfig,
    ) -> Result<Self> {
        let volume = video_config.volume.clamp(0.0, 1.0) as f32;
        Ok(Self {
            output_info,
            source,
            video_config,
            renderer: None,
            pending: None,
            rollback: None,
            egl_window: None,
            texture_pool: None,
            state: PlaybackState::Stopped,
            manual_paused: false,
            power_paused: false,
            volume,
            volume_override: None,
            gl_loaded: false,
            redraw_needed: false,
            metrics: EngineRenderMetrics::default(),
        })
    }

    pub(super) fn render_demand(&self, width: i32, height: i32, first_frame: bool) -> FrameDemand {
        let resized = self
            .egl_window
            .as_ref()
            .is_some_and(|window| window.width() != width || window.height() != height);
        let renderer = self
            .pending
            .as_ref()
            .map(|(_, renderer)| renderer)
            .or(self.renderer.as_ref());
        frame_demand(
            self.state,
            first_frame,
            self.redraw_needed,
            resized,
            renderer.is_some_and(RendererInstance::continuous),
        )
    }

    pub(super) fn metrics(&self) -> EngineRenderMetrics {
        let mut metrics = self.metrics;
        metrics.scene_payload_bytes = self.scene_payload_bytes();
        metrics
    }

    pub(crate) fn use_texture_pool(&mut self, pool: Rc<TexturePool>) -> Result<()> {
        anyhow::ensure!(
            self.renderer.is_none() && self.pending.is_none() && self.rollback.is_none(),
            "Cannot change an initialized Scene texture domain"
        );
        self.texture_pool = Some(pool);
        Ok(())
    }

    pub(super) fn resource_plan(&self) -> Result<ResourcePlan> {
        if self.renderer.is_none()
            && let WallpaperSource::Scene { prepared, .. } = &self.source
        {
            return prepared
                .resource_plan_for_output(self.output_info.width, self.output_info.height);
        }
        let mut plan = ResourcePlan::default();
        for renderer in self
            .renderer
            .iter()
            .chain(self.pending.iter().map(|(_, renderer)| renderer))
            .chain(self.rollback.iter().map(|(_, renderer)| renderer))
        {
            plan.merge(&renderer.allocation_plan()?)?;
        }
        Ok(plan)
    }

    fn scene_payload_bytes(&self) -> u64 {
        self.resource_plan()
            .and_then(|plan| plan.total(0))
            .unwrap_or(u64::MAX)
    }

    pub(super) fn reserved_scene_payload_bytes(&self) -> Result<u64> {
        self.resource_plan()?.total(0)
    }

    pub(super) fn additional_scene_payload_bytes(&self, width: i32, height: i32) -> Result<u64> {
        if let Some((_, renderer)) = &self.pending {
            return renderer.additional_gpu_bytes(width, height);
        }
        if let Some(renderer) = &self.renderer {
            return renderer.additional_gpu_bytes(width, height);
        }
        if let WallpaperSource::Scene { prepared, .. } = &self.source {
            return Ok(prepared
                .estimated_gpu_bytes_for_output(width, height)?
                .saturating_sub(self.reserved_scene_payload_bytes()?));
        }
        Ok(0)
    }

    pub(super) fn audio_allocations(&self) -> Vec<(usize, u64)> {
        std::iter::once(&self.source)
            .chain(self.pending.as_ref().map(|(source, _)| source))
            .chain(self.rollback.as_ref().map(|(source, _)| source))
            .flat_map(|source| match source {
                WallpaperSource::Scene { prepared, .. } => {
                    prepared.audio_allocations().collect::<Vec<_>>()
                }
                WallpaperSource::Video { .. } => Vec::new(),
            })
            .collect()
    }

    pub(super) fn decoded_allocations(&self) -> Result<Vec<(usize, u64)>> {
        std::iter::once(&self.source)
            .chain(self.pending.as_ref().map(|(source, _)| source))
            .chain(self.rollback.as_ref().map(|(source, _)| source))
            .filter_map(|source| match source {
                WallpaperSource::Scene { prepared, .. } => Some(
                    prepared
                        .decoded_texture_bytes()
                        .map(|bytes| (Arc::as_ptr(prepared) as usize, bytes)),
                ),
                WallpaperSource::Video { .. } => None,
            })
            .collect()
    }

    pub(super) fn poll_audio_health(&mut self) -> Result<()> {
        if let Some(renderer) = &mut self.renderer {
            renderer.poll_audio_health()?;
        }
        if let Some((_, renderer)) = &mut self.rollback {
            renderer.poll_audio_health()?;
        }
        Ok(())
    }

    fn initialize_resources(
        &mut self,
        egl: &EglContext,
        surface: &WlSurface,
        width: i32,
        height: i32,
    ) -> Result<()> {
        if self.renderer.is_some() {
            return Ok(());
        }
        if self.egl_window.is_none() {
            self.egl_window = Some(egl.create_window(surface, width, height)?);
        }
        egl.make_current(
            self.egl_window
                .as_ref()
                .context("Renderer has no EGL window")?,
        )?;
        if !self.gl_loaded {
            egl.load_gl_functions();
            self.gl_loaded = true;
        }
        if self.texture_pool.is_none() {
            self.texture_pool = Some(TexturePool::new()?);
        }
        self.texture_pool
            .as_ref()
            .expect("texture domain initialized")
            .bind_owner(egl)?;
        let mut renderer = RendererInstance::create(
            &self.source,
            &self.video_config,
            &self.output_info,
            egl,
            Rc::clone(
                self.texture_pool
                    .as_ref()
                    .expect("texture domain initialized"),
            ),
        )?;
        renderer.volume(self.source_volume(&self.source));
        let paused = self.manual_paused || self.power_paused;
        if paused {
            renderer.pause();
        }
        self.renderer = Some(renderer);
        self.state = if paused {
            PlaybackState::Paused
        } else {
            PlaybackState::Playing
        };
        Ok(())
    }

    pub fn render_frame_to_surface(
        &mut self,
        egl: &EglContext,
        surface: &WlSurface,
        width: i32,
        height: i32,
        force: bool,
    ) -> Result<bool> {
        if self.state == PlaybackState::Stopped && self.egl_window.is_some() {
            return Ok(false);
        }
        self.initialize_resources(egl, surface, width, height)?;
        let is_pending = self.pending.is_some();
        let window = self
            .egl_window
            .as_mut()
            .context("Renderer has no EGL window")?;
        let resized = window.width() != width || window.height() != height;
        if self.state != PlaybackState::Playing
            && !is_pending
            && !self.redraw_needed
            && !resized
            && !force
        {
            return Ok(false);
        }
        if resized {
            window.resize(width, height)?;
        }
        egl.make_current(window)?;
        let renderer = if let Some((_, candidate)) = &mut self.pending {
            candidate
        } else {
            self.renderer.as_mut().context("Renderer unavailable")?
        };
        self.metrics.render_attempts = self.metrics.render_attempts.saturating_add(1);
        let frame_started = Instant::now();
        if renderer.render(
            width,
            height,
            force || is_pending,
            resized,
            self.redraw_needed,
        )? {
            self.metrics.draws = self.metrics.draws.saturating_add(1);
            egl.swap_buffers(window)?;
            self.metrics.swaps = self.metrics.swaps.saturating_add(1);
            self.metrics.record_frame_duration(frame_started.elapsed());
            renderer.report_swap();
            self.redraw_needed = false;
            return Ok(true);
        }
        self.metrics.empty_frames = self.metrics.empty_frames.saturating_add(1);
        Ok(false)
    }

    /// Build a candidate with the correct context current; retain the old renderer.
    pub fn load_source(&mut self, source: WallpaperSource, egl: &EglContext) -> Result<()> {
        let window = self
            .egl_window
            .as_ref()
            .context("Cannot replace uninitialized renderer")?;
        egl.make_current(window)?;
        let mut candidate = RendererInstance::create(
            &source,
            &self.video_config,
            &self.output_info,
            egl,
            Rc::clone(
                self.texture_pool
                    .as_ref()
                    .context("Scene texture domain unavailable")?,
            ),
        )?;
        candidate.volume(0.0);
        if self.state == PlaybackState::Paused {
            candidate.pause();
        }
        self.rollback = None;
        self.pending = Some((source, candidate));
        Ok(())
    }

    /// Called after the host accepts the first frame. Rollback remains possible
    /// until the host confirms the commit and saves the assignment.
    pub(crate) fn accept_pending(&mut self) {
        if let Some((source, mut renderer)) = self.pending.take() {
            renderer.volume(self.source_volume(&source));
            let previous_source = std::mem::replace(&mut self.source, source);
            if let Some(mut previous) = self.renderer.replace(renderer) {
                previous.freeze_for_rollback();
                self.rollback = Some((previous_source, previous));
            }
        }
    }

    pub(crate) fn finalize_pending(&mut self, egl: &EglContext) -> Result<()> {
        egl.make_current(
            self.egl_window
                .as_ref()
                .context("Candidate has no EGL window")?,
        )?;
        if let Some((_, previous)) = &mut self.rollback {
            previous.deactivate_audio()?;
        }
        let volume = self.source_volume(&self.source);
        if let Some(renderer) = &mut self.renderer {
            renderer.activate_audio(self.manual_paused || self.power_paused, volume)?;
        }
        self.rollback = None;
        Ok(())
    }

    pub(crate) fn discard_pending(&mut self, egl: &EglContext) -> Result<bool> {
        if self.pending.is_none() && self.rollback.is_none() {
            return Ok(false);
        }
        egl.make_current(
            self.egl_window
                .as_ref()
                .context("Candidate has no EGL window")?,
        )?;
        self.pending = None;
        if let Some((source, mut renderer)) = self.rollback.take() {
            renderer.volume(self.source_volume(&source));
            if self.state == PlaybackState::Playing {
                renderer.resume();
            }
            self.source = source;
            self.renderer = Some(renderer);
        }
        self.redraw_needed = true;
        Ok(true)
    }

    pub fn pause(&mut self) {
        self.manual_paused = true;
        self.synchronize_pause();
    }

    pub fn resume(&mut self) {
        self.manual_paused = false;
        self.synchronize_pause();
    }

    pub(super) fn set_power_paused(&mut self, paused: bool) {
        self.power_paused = paused;
        self.synchronize_pause();
    }

    pub(super) fn is_manual_paused(&self) -> bool {
        self.manual_paused
    }

    /// Policy pause never overwrites the user's intent. The renderer clock and
    /// decoder only resume when both factors allow playback; rollback stays
    /// hidden and is resumed by discard_pending only if effective playback is on.
    fn synchronize_pause(&mut self) {
        if self.manual_paused || self.power_paused {
            if self.state == PlaybackState::Paused {
                return;
            }
            if let Some(renderer) = &mut self.renderer {
                renderer.pause();
            }
            if let Some((_, renderer)) = &mut self.pending {
                renderer.pause();
            }
            if let Some((_, renderer)) = &mut self.rollback {
                renderer.pause();
            }
            self.state = PlaybackState::Paused;
        } else if self.state == PlaybackState::Paused {
            if let Some(renderer) = &mut self.renderer {
                renderer.resume();
            }
            if let Some((_, renderer)) = &mut self.pending {
                renderer.resume();
            }
            if let Some((_, renderer)) = &mut self.rollback {
                renderer.rollback_audio_pause(false);
            }
            self.state = PlaybackState::Playing;
        }
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 1.0);
        self.volume_override = Some(self.volume);
        if let Some(renderer) = &mut self.renderer {
            renderer.volume(self.volume);
        }
        if let Some((_, renderer)) = &mut self.rollback
            && matches!(renderer, RendererInstance::Scene { .. })
        {
            renderer.volume(self.volume);
        }
    }

    fn source_volume(&self, source: &WallpaperSource) -> f32 {
        self.volume_override.unwrap_or(match source {
            WallpaperSource::Scene { .. } => 1.0,
            WallpaperSource::Video { .. } => self.volume,
        })
    }

    pub fn wallpaper_path(&self) -> Option<&str> {
        self.source.path().to_str()
    }
    pub fn state(&self) -> PlaybackState {
        self.state
    }
    pub fn can_reuse_resources(&self) -> bool {
        self.renderer.is_some() && self.egl_window.is_some() && self.state != PlaybackState::Stopped
    }
    pub fn output_name(&self) -> &str {
        &self.output_info.name
    }

    pub fn cleanup_egl(&mut self, egl: &EglContext) -> Result<()> {
        if self.renderer.is_some() || self.pending.is_some() || self.rollback.is_some() {
            egl.make_current(
                self.egl_window
                    .as_ref()
                    .context("Renderer has no EGL window during cleanup")?,
            )?;
        }
        self.pending = None;
        self.rollback = None;
        self.renderer = None;
        self.state = PlaybackState::Stopped;
        if let Some(window) = &self.egl_window {
            egl.destroy_surface(window)?;
        }
        self.egl_window = None;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{FrameDemand, PlaybackState, SceneClock, WallpaperSession, frame_demand};

    fn authored_session() -> WallpaperSession {
        WallpaperSession::new(
            "/authored/video.mp4".into(),
            lwe_core::OutputInfo {
                name: "authored-output".into(),
                width: 32,
                height: 32,
                scale: 1.0,
                position: (0, 0),
                active: true,
                hdr_capabilities: Default::default(),
            },
            crate::mpv::VideoConfig::default(),
        )
        .unwrap()
    }

    #[test]
    fn resume_on_battery_only_clears_manual_pause_until_power_allows_playback() {
        let mut session = authored_session();
        session.pause();
        session.set_power_paused(true);
        session.resume();
        assert!(!session.is_manual_paused());
        assert!(session.power_paused);
        assert_eq!(session.state(), PlaybackState::Paused);
        assert_eq!(session.render_demand(32, 32, false), FrameDemand::Idle);
        assert_eq!(session.render_demand(32, 32, true), FrameDemand::Urgent);
        session.set_power_paused(false);
        assert_eq!(session.state(), PlaybackState::Playing);
    }

    #[test]
    fn battery_unpause_preserves_manual_pause_and_forced_redraw() {
        let mut session = authored_session();
        session.pause();
        session.set_power_paused(true);
        session.set_power_paused(false);
        assert!(session.is_manual_paused());
        assert_eq!(session.state(), PlaybackState::Paused);
        assert_eq!(session.render_demand(32, 32, false), FrameDemand::Idle);
        session.redraw_needed = true;
        assert_eq!(session.render_demand(32, 32, false), FrameDemand::Urgent);
        session.redraw_needed = false;
        session.resume();
        assert_eq!(session.state(), PlaybackState::Playing);
    }

    #[test]
    fn battery_first_frame_does_not_create_a_manual_pause() {
        let mut session = authored_session();
        session.set_power_paused(true);
        assert_eq!(session.state(), PlaybackState::Paused);
        assert_eq!(session.render_demand(32, 32, true), FrameDemand::Urgent);
        // The post-swap policy reinforcement must not behave like user Pause.
        session.set_power_paused(true);
        assert!(!session.is_manual_paused());
        session.set_power_paused(false);
        assert_eq!(session.state(), PlaybackState::Playing);
    }

    #[test]
    fn pause_before_initialization_preserves_the_first_frame_request() {
        let output = lwe_core::OutputInfo {
            name: "test-output".into(),
            width: 32,
            height: 32,
            scale: 1.0,
            position: (0, 0),
            active: true,
            hdr_capabilities: Default::default(),
        };
        let mut session = WallpaperSession::new(
            "/authored/video.mp4".into(),
            output,
            crate::mpv::VideoConfig::default(),
        )
        .unwrap();
        session.pause();
        assert_eq!(session.state(), PlaybackState::Paused);
        assert_eq!(session.render_demand(32, 32, true), FrameDemand::Urgent);
        assert_eq!(session.render_demand(32, 32, false), FrameDemand::Idle);
        session.resume();
        assert_eq!(session.state(), PlaybackState::Playing);
    }

    #[test]
    fn static_or_paused_renderers_do_not_sustain_frame_callbacks() {
        assert_eq!(
            frame_demand(PlaybackState::Playing, false, false, false, false),
            FrameDemand::Idle
        );
        assert_eq!(
            frame_demand(PlaybackState::Paused, false, false, false, true),
            FrameDemand::Idle
        );
        assert_eq!(
            frame_demand(PlaybackState::Playing, false, false, false, true),
            FrameDemand::Continuous
        );
    }

    #[test]
    fn paused_first_frame_resize_and_rollback_are_urgent() {
        for (first_frame, redraw, resized) in [
            (true, false, false),
            (false, true, false),
            (false, false, true),
        ] {
            assert_eq!(
                frame_demand(PlaybackState::Paused, first_frame, redraw, resized, false),
                FrameDemand::Urgent
            );
        }
        assert_eq!(
            frame_demand(PlaybackState::Stopped, true, false, false, false),
            FrameDemand::Urgent
        );
    }

    #[test]
    fn paused_scene_clock_does_not_advance_and_resume_is_idempotent() {
        let mut clock = SceneClock::new();
        clock.pause();
        let elapsed = clock.seconds();
        clock.pause();
        assert_eq!(clock.seconds(), elapsed);
        clock.resume();
        let started = clock.started;
        clock.resume();
        assert_eq!(clock.started, started);
    }
}

//! Render user-owned Scene resources to PNG without touching the desktop.
//! Usage: EGL_PLATFORM=surfaceless cargo run -p lwe-engine --example render_scene -- PROJECT ASSETS OUTPUT TIME
use std::path::Path;

use anyhow::{Context, Result, ensure};
use khronos_egl as egl;
use lwe_engine::scene::PreparedScene;

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    ensure!(
        args.len() == 4,
        "Usage: render_scene PROJECT ASSETS OUTPUT TIME_SECONDS"
    );
    let time: f64 = args[3].parse()?;
    ensure!(
        time.is_finite() && time >= 0.0,
        "Time must be finite and nonnegative"
    );
    let prepared = PreparedScene::prepare(Path::new(&args[0]), Some(Path::new(&args[1])))?;
    eprintln!("Prepared {:?}", prepared.summary());
    let instance = unsafe { egl::DynamicInstance::<egl::EGL1_4>::load_required()? };
    let display = unsafe { instance.get_display(egl::DEFAULT_DISPLAY) }
        .context("No EGL display; use EGL_PLATFORM=surfaceless")?;
    instance.initialize(display)?;
    instance.bind_api(egl::OPENGL_API)?;
    let config = instance
        .choose_first_config(
            display,
            &[
                egl::SURFACE_TYPE,
                egl::PBUFFER_BIT,
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
                egl::NONE,
            ],
        )?
        .context("No EGL pbuffer configuration")?;
    let context = instance.create_context(
        display,
        config,
        None,
        &[
            egl::CONTEXT_MAJOR_VERSION,
            3,
            egl::CONTEXT_MINOR_VERSION,
            0,
            egl::CONTEXT_OPENGL_PROFILE_MASK,
            egl::CONTEXT_OPENGL_CORE_PROFILE_BIT,
            egl::NONE,
        ],
    )?;
    let width: i32 = 960;
    let height: i32 = 540;
    let surface = instance.create_pbuffer_surface(
        display,
        config,
        &[egl::WIDTH, width, egl::HEIGHT, height, egl::NONE],
    )?;
    instance.make_current(display, Some(surface), Some(surface), Some(context))?;
    gl::load_with(|name| {
        instance
            .get_proc_address(name)
            .map(|f| f as *const std::ffi::c_void)
            .unwrap_or(std::ptr::null())
    });
    let mut renderer = prepared.initialize()?;
    ensure!(
        renderer.render(width, height, time)?,
        "Scene produced no frame"
    );
    let mut rgba = vec![0_u8; width as usize * height as usize * 4];
    unsafe {
        gl::ReadPixels(
            0,
            0,
            width,
            height,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            rgba.as_mut_ptr().cast(),
        );
        let error = gl::GetError();
        ensure!(error == gl::NO_ERROR, "OpenGL readback error: {error:#x}");
    }
    let image = image::RgbaImage::from_raw(width as u32, height as u32, rgba)
        .context("Readback dimensions")?;
    image::imageops::flip_vertical(&image).save(&args[2])?;
    renderer.cleanup();
    drop(renderer);
    instance.make_current(display, None, None, None)?;
    instance.destroy_surface(display, surface)?;
    instance.destroy_context(display, context)?;
    instance.terminate(display)?;
    Ok(())
}

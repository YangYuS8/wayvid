//! Authored copy fixtures; no Workshop assets or desktop backend.

use super::*;

fn pixels(target: &RenderTarget) -> Result<Vec<u8>> {
    Ok(frozen_history::FrozenHistory::capture(
        target,
        super::super::timed_runtime::scene_texture_memory_budget(),
    )?
    .pixels()
    .to_vec())
}

fn draw(renderer: &mut SceneRenderer, time: f64) -> Result<Vec<u8>> {
    ensure!(renderer.render(2, 2, time)?, "Copy fixture did not render");
    renderer.report_presented();
    pixels(
        renderer
            .front
            .as_ref()
            .context("Copy fixture has no presented cache")?,
    )
}

#[test]
#[ignore = "requires an opt-in current EGL/OpenGL context"]
fn existing_read_fbo_matches_temporary_fbo_rgba_exactly() -> Result<()> {
    let _context = sharing_tests::Offscreen::new()?;
    let mut scene = sharing_tests::fixture(
        false,
        vec![
            255, 127, 63, 0, 80, 120, 200, 1, 255, 40, 80, 127, 10, 200, 70, 255,
        ],
    );
    scene.layers[0].blend = Blend::Replace;
    scene.summary.animated = true;
    let mut renderer = SceneRenderer::initialize(&scene)?;
    let reused = RenderTarget::new(2, 2)?;
    let temporary = RenderTarget::new(2, 2)?;
    for (frame, tint) in [[1.; 4], [0.75, 0.5, 0.25, 0.625], [0.25, 0.75, 1., 0.5]]
        .into_iter()
        .enumerate()
    {
        renderer.layers[0].tint = tint;
        let expected = draw(&mut renderer, frame as f64 * 0.125)?;
        let source = renderer.front.as_ref().unwrap();
        copy_texture(CopySource::Target(source), &reused)?;
        copy_texture(CopySource::Texture(&source.texture), &temporary)?;
        assert_eq!(pixels(&reused)?, expected, "Reused FBO, frame {frame}");
        assert_eq!(
            pixels(&temporary)?,
            expected,
            "Temporary FBO, frame {frame}"
        );
        assert_eq!(pixels(source)?, expected, "Copy modified its source");
    }
    let snapshot = frozen_history::FrozenHistory::capture(
        renderer.front.as_ref().unwrap(),
        super::super::timed_runtime::scene_texture_memory_budget(),
    )?;
    let restored = snapshot.restore()?;
    copy_texture(CopySource::Target(&restored), &reused)?;
    assert_eq!(pixels(&reused)?, snapshot.pixels());
    Ok(())
}

fn history_fixture() -> PreparedScene {
    let mut scene = sharing_tests::fixture(false, [200, 80, 20, 255].repeat(4));
    scene.layers[0].blend = Blend::Replace;
    scene.summary.animated = true;
    let mut accumulation = super::super::shader::image();
    accumulation.fragment = "#version 130\nin vec2 v_UV; out vec4 lwe_FragColor; uniform sampler2D g_Texture0; uniform sampler2D g_Texture1; void main(){lwe_FragColor=mix(texture(g_Texture1,v_UV),texture(g_Texture0,v_UV),0.5);}".into();
    let render = |shader, textures, target| PreparedPass {
        kind: super::super::PreparedPassKind::Render,
        label: "authored copy output".into(),
        shader: Some(shader),
        textures,
        constants: BTreeMap::new(),
        target,
        blend: Blend::Replace,
    };
    let copy = |source, target: &str| PreparedPass {
        kind: super::super::PreparedPassKind::Copy { source },
        label: "authored target copy".into(),
        shader: None,
        textures: Vec::new(),
        constants: BTreeMap::new(),
        target: Some(target.into()),
        blend: Blend::Replace,
    };
    scene.layers[0].stages.push(PreparedStage {
        targets: BTreeMap::from([
            ("work".into(), 1.),
            ("history".into(), 1.),
            ("saved".into(), 1.),
        ]),
        persistent_targets: std::collections::BTreeSet::from(["history".into(), "saved".into()]),
        passes: vec![
            render(
                accumulation,
                vec![
                    TextureBinding::Input,
                    TextureBinding::History("history".into()),
                ],
                Some("work".into()),
            ),
            copy(TextureBinding::Named("work".into()), "history"),
            copy(TextureBinding::History("history".into()), "saved"),
            render(
                super::super::shader::image(),
                vec![TextureBinding::History("saved".into())],
                None,
            ),
        ],
    });
    scene
}

#[test]
#[ignore = "requires an opt-in current EGL/OpenGL context"]
fn named_and_history_copies_preserve_replay_and_restored_sources() -> Result<()> {
    let _context = sharing_tests::Offscreen::new()?;
    let scene = history_fixture();
    let mut renderer = SceneRenderer::initialize(&scene)?;
    let mut previous = None;
    let mut payload = None;
    for frame in 0..3 {
        let output = draw(&mut renderer, frame as f64 * 0.125)?;
        let stage = &renderer.layers[0].stages[1];
        assert_eq!(pixels(&stage.targets["work"])?, output);
        assert_eq!(pixels(&stage.targets["history"])?, output);
        assert_eq!(pixels(&stage.targets["saved"])?, output);
        if let Some(payload) = payload {
            assert_eq!(renderer.gpu_payload_bytes(), payload);
        }
        payload = Some(renderer.gpu_payload_bytes());
        if let Some(previous) = previous {
            assert_ne!(output, previous, "History stopped advancing");
        }
        previous = Some(output);
    }
    let expected = previous.unwrap();
    let seq = renderer.accepted_seq;
    let time = renderer.accepted_time;
    renderer.pause();
    ensure!(
        renderer.render_forced(4, 4, 1.)?,
        "Paused replay did not render"
    );
    renderer.report_presented();
    assert_eq!(renderer.accepted_seq, seq);
    assert_eq!(renderer.accepted_time, time);
    let stage = &renderer.layers[0].stages[1];
    assert_eq!(pixels(&stage.targets["history"])?, expected);
    assert_eq!(pixels(&stage.targets["saved"])?, expected);
    renderer.freeze_for_rollback()?;
    renderer.thaw(true);
    ensure!(
        renderer.render_forced(2, 2, 1.25)?,
        "Restored replay did not render"
    );
    renderer.report_presented();
    assert_eq!(pixels(renderer.front.as_ref().unwrap())?, expected);
    assert_eq!(renderer.accepted_seq, seq);
    assert_eq!(renderer.accepted_time, time);
    renderer.resume();
    let resumed = draw(&mut renderer, 1.5)?;
    assert_ne!(resumed, expected, "Restored history stopped advancing");
    let stage = &renderer.layers[0].stages[1];
    assert_eq!(pixels(&stage.targets["work"])?, resumed);
    assert_eq!(pixels(&stage.targets["history"])?, resumed);
    assert_eq!(pixels(&stage.targets["saved"])?, resumed);
    assert_eq!(Some(renderer.gpu_payload_bytes()), payload);
    Ok(())
}

#[test]
#[ignore = "requires an opt-in current EGL/OpenGL context"]
fn reused_and_temporary_copy_sources_reject_feedback_and_dimensions_before_gl() -> Result<()> {
    let _context = sharing_tests::Offscreen::new()?;
    let scene = sharing_tests::fixture(false, [90, 120, 150, 255].repeat(4));
    let mut renderer = SceneRenderer::initialize(&scene)?;
    let expected = draw(&mut renderer, 0.)?;
    let source = renderer.front.as_ref().unwrap();
    let wrong_size = RenderTarget::new(1, 1)?;
    let state = GlState::capture();
    for reuse in [false, true] {
        let selected = || {
            if reuse {
                CopySource::Target(source)
            } else {
                CopySource::Texture(&source.texture)
            }
        };
        assert_eq!(
            copy_texture(selected(), source).unwrap_err().to_string(),
            "Scene copy feedback loop"
        );
        assert_eq!(
            copy_texture(selected(), &wrong_size)
                .unwrap_err()
                .to_string(),
            "Scene copy dimensions disagree"
        );
        let after = GlState::capture();
        assert_eq!(after.read_fbo, state.read_fbo);
        assert_eq!(after.draw_fbo, state.draw_fbo);
        check_gl("copy validation retained clean GL state")?;
    }
    assert_eq!(pixels(source)?, expected);
    Ok(())
}

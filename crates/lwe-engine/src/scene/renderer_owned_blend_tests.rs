//! LWE-owned template equality checks; no Workshop assets or desktop backend.

use super::*;

fn fixture(blend: Blend, timed: bool, specialized: bool) -> Result<PreparedScene> {
    let mut scene = sharing_tests::fixture(
        false,
        vec![
            255, 127, 63, 0, 80, 120, 200, 1, 255, 40, 80, 127, 10, 200, 70, 255,
        ],
    );
    scene.textures.push(PreparedTexture {
        static_source: None,
        timed: None,
        image: None,
        text_frame: None,
        _rgba_reservation: None,
        fingerprint: [0; 32],
        width: 2,
        height: 2,
        image_width: 2,
        image_height: 2,
        source_format: 0,
        source_flags: 0,
        rgba: vec![
            10, 200, 70, 0, 128, 2, 255, 64, 1, 30, 255, 128, 50, 90, 130, 255,
        ],
        animation: None,
    });
    let mut shader = if timed {
        super::super::shader::text_material()?
    } else {
        super::super::shader::image_material()
    };
    if !specialized {
        // A comment preserves the baseline GLSL semantics while making this
        // authored copy distinct from the complete canonical owned template.
        shader.vertex.push_str("\n// Baseline template copy\n");
    }
    let pass = &mut scene.layers[0].stages[0].passes[0];
    pass.shader = Some(shader);
    pass.textures = vec![TextureBinding::Static(0), TextureBinding::Static(1)];
    pass.blend = blend;
    scene.layers[0].blend = Blend::Replace;
    scene.summary.animated = true;
    Ok(scene)
}

fn draw(renderer: &mut SceneRenderer, time: f64, tint: [f32; 4]) -> Result<Vec<u8>> {
    renderer.layers[0].tint = tint;
    ensure!(renderer.render(2, 2, time)?, "Fixture did not render");
    renderer.report_presented();
    Ok(frozen_history::FrozenHistory::capture(
        renderer
            .front
            .as_ref()
            .context("Fixture has no presented cache")?,
        super::super::timed_runtime::scene_texture_memory_budget(),
    )?
    .pixels()
    .to_vec())
}

#[test]
#[ignore = "requires an opt-in current EGL/OpenGL context"]
fn owned_blend_templates_match_uniform_rgba_and_sampler_liveness() -> Result<()> {
    let _context = sharing_tests::Offscreen::new()?;
    for timed in [false, true] {
        for blend in [Blend::Replace, Blend::Alpha, Blend::Add] {
            let original = fixture(blend, timed, false)?;
            let candidate = fixture(blend, timed, true)?;
            let original_shader = original.layers[0].stages[0].passes[0]
                .shader
                .as_ref()
                .unwrap()
                .clone();
            let candidate_shader = candidate.layers[0].stages[0].passes[0]
                .shader
                .as_ref()
                .unwrap()
                .clone();
            let original_plan = resource_plan(&original)?;
            let candidate_plan = resource_plan(&candidate)?;
            assert_eq!(original_plan.private_bytes, candidate_plan.private_bytes);
            assert_eq!(original_plan.work, candidate_plan.work);
            assert_eq!(original_plan.textures, candidate_plan.textures);
            let mut original_renderer = SceneRenderer::initialize(&original)?;
            let mut candidate_renderer = SceneRenderer::initialize(&candidate)?;
            let original_program = original_renderer.layers[0].stages[0].passes[0]
                .program
                .as_ref()
                .unwrap();
            let candidate_program = candidate_renderer.layers[0].stages[0].passes[0]
                .program
                .as_ref()
                .unwrap();
            assert!(original_program.location("g_BlendMode") >= 0);
            assert!(candidate_program.location("g_BlendMode") < 0);
            assert!(original_program.location("g_Texture1") >= 0);
            assert_eq!(
                candidate_program.location("g_Texture1") >= 0,
                shader_blend_mode(blend) != 0
            );
            assert!(candidate_program.location("g_Color4") >= 0);
            if timed {
                assert!(candidate_program.location("g_Texture0UVTransform") >= 0);
            }
            for (frame, tint) in [[1.; 4], [0.75, 0.5, 0.25, 0.625]].into_iter().enumerate() {
                let time = frame as f64 * 0.125;
                assert_eq!(
                    draw(&mut candidate_renderer, time, tint)?,
                    draw(&mut original_renderer, time, tint)?,
                    "Mode {}, timed {timed}, frame {frame}",
                    shader_blend_mode(blend),
                );
                assert_eq!(
                    candidate_renderer.accepted_seq,
                    original_renderer.accepted_seq
                );
                assert_eq!(
                    candidate_renderer.accepted_time,
                    original_renderer.accepted_time
                );
            }
            assert_eq!(
                original.layers[0].stages[0].passes[0]
                    .shader
                    .as_ref()
                    .unwrap(),
                &original_shader
            );
            assert_eq!(
                candidate.layers[0].stages[0].passes[0]
                    .shader
                    .as_ref()
                    .unwrap(),
                &candidate_shader
            );
        }
    }
    Ok(())
}

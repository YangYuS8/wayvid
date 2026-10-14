use super::*;

#[test]
fn owned_blend_variants_preserve_all_math_and_canonical_metadata() {
    for original in [image_material(), text_material().unwrap()] {
        for mode in 0..=2 {
            let variant = specialize_owned_image_blend(&original, mode).unwrap();
            let declaration = format!("const int g_BlendMode = {mode};");
            assert_eq!(variant.fragment.matches(&declaration).count(), 1);
            assert_eq!(
                variant
                    .fragment
                    .replace(&declaration, "uniform int g_BlendMode;"),
                original.fragment,
            );
            assert_eq!(variant.vertex, original.vertex);
            assert_eq!(variant.uniforms, original.uniforms);
            assert!(original.fragment.contains("uniform int g_BlendMode;"));
        }
    }
}

#[test]
fn external_or_modified_templates_and_metadata_are_not_specialized() {
    let original = image_material();
    for change in 0..4 {
        let mut authored = original.clone();
        match change {
            0 => authored.vertex.push_str("\n// Authored vertex\n"),
            1 => authored.fragment.push_str("\n// Authored fragment\n"),
            2 => authored.uniforms = text_material().unwrap().uniforms,
            _ => authored.fragment = "uniform int g_BlendMode; void main(){}".into(),
        }
        let unchanged = authored.clone();
        assert!(specialize_owned_image_blend(&authored, 0).is_none());
        assert_eq!(authored, unchanged);
    }
    let mut timed = text_material().unwrap();
    timed.uniforms[0].kind = "vec3".into();
    assert!(specialize_owned_image_blend(&timed, 0).is_none());
    assert!(specialize_owned_image_blend(&image(), 0).is_none());
}

#[test]
fn owned_blend_specialization_rejects_unknown_modes() {
    for mode in [-1, 3, i32::MAX] {
        assert!(specialize_owned_image_blend(&image_material(), mode).is_none());
    }
}

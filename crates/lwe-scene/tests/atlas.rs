use lwe_scene::{SceneAssetError, Texture, TextureInspectionBudget};

fn atlas() -> Vec<u8> {
    let mut bytes = b"TEXV0005\0TEXI0001\0".to_vec();
    for value in [9_u32, 4, 4, 2, 4, 2, 0] {
        bytes.extend(value.to_le_bytes());
    }
    bytes.extend(b"TEXB0002\0");
    for value in [1_u32, 1, 4, 2, 0, 0, 8] {
        bytes.extend(value.to_le_bytes());
    }
    bytes.extend([0_u8, 255, 128, 64, 255, 0, 64, 128]);
    bytes.extend(b"TEXS0002\0");
    bytes.extend(2_u32.to_le_bytes());
    for x in [0.0_f32, 2.0] {
        bytes.extend(0_u32.to_le_bytes());
        for value in [0.0_f32, x, 0.0, 2.0, 0.0, 0.0, 2.0] {
            bytes.extend(value.to_le_bytes());
        }
    }
    bytes
}

#[test]
fn single_image_particle_atlas_retains_frame_uvs_without_static_fallback() {
    let bytes = atlas();
    let texture = Texture::decode_atlas(&bytes).unwrap();
    assert_eq!((texture.width, texture.height), (4, 2));
    assert_eq!(texture.rgba.len(), 32);
    let animation = texture.metadata.animation.unwrap();
    assert_eq!(
        (animation.version, animation.width, animation.height),
        (2, 4, 2)
    );
    assert_eq!(
        animation.frames[1].coordinates,
        [2.0, 0.0, 2.0, 0.0, 0.0, 2.0]
    );
    assert!(matches!(
        Texture::decode(&bytes),
        Err(SceneAssetError::Unsupported { .. })
    ));
}

#[test]
fn particle_atlas_rejects_truncation_invalid_indices_coordinates_and_timing() {
    let bytes = atlas();
    let table = bytes.len() - 64;
    for cut in table..bytes.len() {
        assert!(Texture::inspect(&bytes[..cut]).is_err(), "cut={cut}");
    }
    for (offset, value) in [
        (0, 1_u32),
        (4, f32::NAN.to_bits()),
        (4, 1.0_f32.to_bits()),
        (8, (-1.0_f32).to_bits()),
        (16, 5.0_f32.to_bits()),
        (20, 1.0_f32.to_bits()),
        (28, 0),
    ] {
        let mut invalid = bytes.clone();
        invalid[table + offset..table + offset + 4].copy_from_slice(&value.to_le_bytes());
        assert!(Texture::decode_atlas(&invalid).is_err(), "offset={offset}");
    }
    let project = tempfile::tempdir().unwrap();
    std::fs::write(project.path().join("atlas.tex"), &bytes).unwrap();
    let assets = lwe_scene::SceneAssets::open(project.path(), None).unwrap();
    let resource = assets.resource("atlas.tex").unwrap();
    let mut budget = TextureInspectionBudget {
        remaining_mips: 1,
        remaining_animation_frames: 1,
    };
    assert!(Texture::inspect_resource_bounded(&resource, &mut budget).is_err());
    assert_eq!(budget.remaining_mips, 0);
}

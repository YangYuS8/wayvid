use std::fs;
use std::io::Cursor;
use std::os::unix::fs::symlink;
use std::sync::Arc;

use image::{DynamicImage, ImageFormat, RgbaImage};
use lwe_scene::{SceneAssetError, SceneAssets, Texture};

fn package(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend(8_u32.to_le_bytes());
    bytes.extend(b"PKGV0021");
    bytes.extend((entries.len() as u32).to_le_bytes());
    let mut offset = 0_u32;
    for (name, payload) in entries {
        bytes.extend((name.len() as u32).to_le_bytes());
        bytes.extend(name.as_bytes());
        bytes.extend(offset.to_le_bytes());
        bytes.extend((payload.len() as u32).to_le_bytes());
        offset += payload.len() as u32;
    }
    for (_, payload) in entries {
        bytes.extend(*payload);
    }
    bytes
}

#[test]
fn packaged_reads_have_precedence_and_independent_offsets() {
    let project = tempfile::tempdir().unwrap();
    let assets = tempfile::tempdir().unwrap();
    fs::write(
        project.path().join("scene.pkg"),
        package(&[("scene.json", b"{}"), ("asset", b"abcd")]),
    )
    .unwrap();
    fs::write(project.path().join("scene.json"), b"unpacked").unwrap();
    fs::write(project.path().join("loose"), b"project").unwrap();
    fs::write(assets.path().join("builtin"), b"assets").unwrap();
    let reader = Arc::new(SceneAssets::open(project.path(), Some(assets.path())).unwrap());
    assert_eq!(reader.package_version(), Some("PKGV0021"));
    assert_eq!(reader.package_entries().count(), 2);
    assert_eq!(reader.read("scene.json", 2).unwrap(), b"{}");
    assert_eq!(reader.read("loose", 10).unwrap(), b"project");
    assert_eq!(reader.read("builtin", 10).unwrap(), b"assets");
    assert!(matches!(
        reader.read("asset", 3),
        Err(SceneAssetError::LimitExceeded { .. })
    ));
    let threads: Vec<_> = (0..8)
        .map(|_| {
            let reader = reader.clone();
            std::thread::spawn(move || {
                for _ in 0..100 {
                    assert_eq!(reader.read("scene.json", 2).unwrap(), b"{}");
                    assert_eq!(reader.read("asset", 4).unwrap(), b"abcd");
                }
            })
        })
        .collect();
    for thread in threads {
        thread.join().unwrap();
    }
}

#[test]
fn verified_legacy_directories_preserve_resource_ranges_and_reject_corruption() {
    for version in [b"PKGV0009", b"PKGV0018"] {
        let project = tempfile::tempdir().unwrap();
        let path = project.path().join("scene.pkg");
        let mut good = package(&[("a", b"abcdef"), ("b", b"hello")]);
        good[4..12].copy_from_slice(version);
        fs::write(&path, &good).unwrap();
        let reader = SceneAssets::open(project.path(), None).unwrap();
        assert_eq!(
            reader.package_version(),
            Some(std::str::from_utf8(version).unwrap())
        );
        assert_eq!(reader.read("b", 5).unwrap(), b"hello");
        let resource = reader.resource("a").unwrap();
        assert_eq!(resource.read_range(2, 3, 3).unwrap(), b"cde");
        assert!(resource.read_range(5, 2, 2).is_err());
        for length in 0..good.len() {
            fs::write(&path, &good[..length]).unwrap();
            assert!(SceneAssets::open(project.path(), None).is_err());
        }
        for (position, value) in [(34, 0_u32), (21, u32::MAX)] {
            let mut corrupt = good.clone();
            corrupt[position..position + 4].copy_from_slice(&value.to_le_bytes());
            fs::write(&path, corrupt).unwrap();
            assert!(SceneAssets::open(project.path(), None).is_err());
        }
        for entries in [
            vec![("a", b"1".as_slice()), ("a", b"2".as_slice())],
            vec![("../a", b"1".as_slice())],
        ] {
            let mut corrupt = package(&entries);
            corrupt[4..12].copy_from_slice(version);
            fs::write(&path, corrupt).unwrap();
            assert!(SceneAssets::open(project.path(), None).is_err());
        }
        good[4..12].copy_from_slice(b"PKGV0010");
        fs::write(&path, good).unwrap();
        assert!(matches!(
            SceneAssets::open(project.path(), None),
            Err(SceneAssetError::Unsupported { .. })
        ));
    }
}

#[test]
fn loose_resources_reject_traversal_symlink_escape_and_non_files() {
    let project = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    fs::write(outside.path().join("secret"), b"secret").unwrap();
    symlink(outside.path(), project.path().join("escape")).unwrap();
    symlink(outside.path().join("secret"), project.path().join("link")).unwrap();
    fs::create_dir(project.path().join("directory")).unwrap();
    let reader = SceneAssets::open(project.path(), None).unwrap();
    for name in [
        "../secret",
        "/secret",
        "a/../secret",
        "a//b",
        "a\\b",
        "C:secret",
        "./file",
        "a\0b",
    ] {
        assert!(matches!(
            reader.read(name, 100),
            Err(SceneAssetError::UnsafePath { .. })
        ));
    }
    assert!(reader.read("escape/secret", 100).is_err());
    assert!(reader.read("link", 100).is_err());
    assert!(reader.read("directory", 100).is_err());
    assert!(matches!(
        reader.read("missing", 100),
        Err(SceneAssetError::NotFound { .. })
    ));
    assert!(!reader.exists("link"));
}

#[test]
fn malformed_packages_cannot_fall_back_to_loose_scene() {
    let project = tempfile::tempdir().unwrap();
    fs::write(project.path().join("scene.json"), b"{}").unwrap();
    let good = package(&[("a", b"abcdef"), ("b", b"hello")]);
    for length in 0..good.len() {
        // Removing directory bytes or a payload byte must fail during indexing.
        fs::write(project.path().join("scene.pkg"), &good[..length]).unwrap();
        assert!(
            SceneAssets::open(project.path(), None).is_err(),
            "length {length}"
        );
    }
    fs::write(
        project.path().join("scene.pkg"),
        package(&[("a", b"1"), ("a", b"2")]),
    )
    .unwrap();
    assert!(SceneAssets::open(project.path(), None).is_err());
    fs::write(project.path().join("scene.pkg"), package(&[("../a", b"1")])).unwrap();
    assert!(SceneAssets::open(project.path(), None).is_err());
    let mut overlap = good.clone();
    // Header 16 bytes, first entry 13 bytes, second name-length+name 5 bytes.
    overlap[34..38].copy_from_slice(&0_u32.to_le_bytes());
    fs::write(project.path().join("scene.pkg"), overlap).unwrap();
    assert!(SceneAssets::open(project.path(), None).is_err());
    let mut outside = good;
    outside[21..25].copy_from_slice(&u32::MAX.to_le_bytes());
    fs::write(project.path().join("scene.pkg"), outside).unwrap();
    assert!(SceneAssets::open(project.path(), None).is_err());
}

fn encoded_image(format: ImageFormat) -> Vec<u8> {
    let rgba = RgbaImage::from_raw(2, 1, vec![10, 20, 30, 255, 200, 100, 0, 128]).unwrap();
    let image = DynamicImage::ImageRgba8(rgba);
    let image = if format == ImageFormat::Jpeg {
        DynamicImage::ImageRgb8(image.to_rgb8())
    } else {
        image
    };
    let mut output = Cursor::new(Vec::new());
    image.write_to(&mut output, format).unwrap();
    output.into_inner()
}

fn tex(payload: &[u8], body: u32, compression: bool, decompressed_size: u32) -> Vec<u8> {
    let mut bytes = b"TEXV0005\0TEXI0001\0".to_vec();
    // Format 0, flags 2, storage 4x2, logical 2x1, final unknown image field 0.
    for value in [0_u32, 2, 4, 2, 2, 1, 0] {
        bytes.extend(value.to_le_bytes());
    }
    bytes.extend(format!("TEXB{body:04}\0").as_bytes());
    bytes.extend(1_u32.to_le_bytes());
    if body >= 3 {
        bytes.extend(13_u32.to_le_bytes());
    }
    if body == 4 {
        bytes.extend(0_u32.to_le_bytes());
    }
    for value in [
        1_u32,
        2,
        1,
        u32::from(compression),
        decompressed_size,
        payload.len() as u32,
    ] {
        bytes.extend(value.to_le_bytes());
    }
    bytes.extend(payload);
    bytes
}

#[test]
fn png_jpeg_and_lz4_images_decode_with_explicit_dimensions() {
    let png = encoded_image(ImageFormat::Png);
    for body in [3, 4] {
        let bytes = tex(&png, body, false, 0);
        let image = Texture::decode(&bytes).unwrap();
        assert_eq!((image.width, image.height), (2, 1));
        assert_eq!(
            (image.metadata.storage_width, image.metadata.storage_height),
            (4, 2)
        );
        assert_eq!(
            (image.metadata.image_width, image.metadata.image_height),
            (2, 1)
        );
        assert_eq!(image.metadata.format, 0);
        assert_eq!(image.metadata.flags, 2);
        assert_eq!(image.rgba, [10, 20, 30, 255, 200, 100, 0, 128]);
    }
    let lz4 = lz4_flex::block::compress(&png);
    let image = Texture::decode(&tex(&lz4, 4, true, png.len() as u32)).unwrap();
    assert_eq!(image.rgba.len(), 8);
    let jpeg = Texture::decode(&tex(&encoded_image(ImageFormat::Jpeg), 3, false, 0)).unwrap();
    assert_eq!((jpeg.width, jpeg.height, jpeg.rgba.len()), (2, 1, 8));
}

#[test]
fn native_sized_raw_channels_are_preserved_after_lz4() {
    for (format, raw, expected) in [
        (
            0_u32,
            vec![10, 20, 30, 40, 50, 60, 70, 80],
            vec![10, 20, 30, 40, 50, 60, 70, 80],
        ),
        (
            8,
            vec![10, 20, 30, 40],
            vec![10, 20, 0, 255, 30, 40, 0, 255],
        ),
        (9, vec![10, 20], vec![10, 0, 0, 255, 20, 0, 0, 255]),
    ] {
        for compressed in [false, true] {
            let payload = if compressed {
                lz4_flex::block::compress(&raw)
            } else {
                raw.clone()
            };
            let mut bytes = tex(
                &payload,
                4,
                compressed,
                if compressed { raw.len() as u32 } else { 0 },
            );
            bytes[18..22].copy_from_slice(&format.to_le_bytes());
            bytes[59..63].copy_from_slice(&u32::MAX.to_le_bytes());
            let image = Texture::decode(&bytes).unwrap();
            assert_eq!(image.rgba, expected);
        }
    }
    let mut short = tex(&[1, 2, 3], 4, false, 0);
    short[18..22].copy_from_slice(&8_u32.to_le_bytes());
    short[59..63].copy_from_slice(&u32::MAX.to_le_bytes());
    assert!(matches!(
        Texture::decode(&short),
        Err(SceneAssetError::InvalidTexture { .. })
    ));
}

#[test]
fn verified_builtin_texb1_framing_decodes_raw_pixels() {
    let mut bytes = b"TEXV0005\0TEXI0001\0".to_vec();
    for value in [0_u32, 0, 2, 1, 2, 1, 0] {
        bytes.extend(value.to_le_bytes());
    }
    bytes.extend(b"TEXB0001\0");
    for value in [1_u32, 1, 2, 1, 8] {
        bytes.extend(value.to_le_bytes());
    }
    bytes.extend([255, 255, 255, 255, 252, 252, 252, 252]);
    let texture = Texture::decode(&bytes).unwrap();
    assert_eq!(texture.metadata.body_version, 1);
    assert_eq!(texture.rgba, [255, 255, 255, 255, 252, 252, 252, 252]);
    assert!(Texture::decode(&bytes[..bytes.len() - 1]).is_err());
}

#[test]
fn texb2_lz4_raw_framing_decodes_without_container_field() {
    let raw = [10, 20, 30, 40, 50, 60, 70, 80];
    let bytes = tex(&lz4_flex::block::compress(&raw), 2, true, 8);
    let texture = Texture::decode(&bytes).unwrap();
    assert_eq!(texture.metadata.body_version, 2);
    assert_eq!(texture.metadata.container_format, -1);
    assert_eq!(texture.rgba, raw);
}

#[test]
fn bc3_uses_exact_blocks_and_crops_partial_edge_blocks() {
    // Opaque red DXT5: alpha endpoint 255/index0, RGB565 red/index0.
    let block = [255, 0, 0, 0, 0, 0, 0, 0, 0, 248, 0, 0, 0, 0, 0, 0];
    let mut bytes = tex(&block, 4, false, 0);
    bytes[18..22].copy_from_slice(&4_u32.to_le_bytes());
    bytes[59..63].copy_from_slice(&u32::MAX.to_le_bytes());
    let texture = Texture::decode(&bytes).unwrap();
    assert_eq!((texture.width, texture.height), (2, 1));
    assert_eq!(texture.rgba, [255, 0, 0, 255, 255, 0, 0, 255]);
    let mut wrong = bytes.clone();
    wrong.push(0);
    wrong[87..91].copy_from_slice(&17_u32.to_le_bytes());
    assert!(Texture::decode(&wrong).is_err());
}

#[test]
fn texture_bounds_and_decode_limits_are_checked_before_allocation() {
    let good = tex(&encoded_image(ImageFormat::Png), 4, false, 0);
    for length in 0..good.len() {
        assert!(Texture::decode(&good[..length]).is_err(), "length {length}");
    }
    let bomb = tex(b"x", 4, true, u32::MAX);
    assert!(matches!(
        Texture::decode(&bomb),
        Err(SceneAssetError::LimitExceeded { .. })
    ));
    assert!(Texture::decode(&tex(b"not lz4", 4, true, 8)).is_err());
    let mut oversized = good.clone();
    oversized[26..30].copy_from_slice(&u32::MAX.to_le_bytes());
    assert!(Texture::decode(&oversized).is_err());
    let mut mismatch = good;
    // TEXB4 image mip width begins at 71.
    mismatch[71..75].copy_from_slice(&1_u32.to_le_bytes());
    assert!(Texture::decode(&mismatch).is_err());
}

#[test]
fn animation_and_video_are_described_without_silent_static_fallback() {
    let mut animated = tex(&encoded_image(ImageFormat::Png), 4, false, 0);
    animated.extend(b"TEXS0003\0");
    for value in [1_u32, 2, 1, 0] {
        animated.extend(value.to_le_bytes());
    }
    animated.extend(0.1_f32.to_le_bytes());
    for value in [0.0_f32, 0.0, 2.0, 0.0, 0.0, 1.0] {
        animated.extend(value.to_le_bytes());
    }
    let metadata = Texture::inspect(&animated).unwrap();
    assert_eq!(metadata.animation.unwrap().frames.len(), 1);
    assert!(matches!(
        Texture::decode(&animated),
        Err(SceneAssetError::Unsupported { .. })
    ));
    let video = tex(b"\0\0\0\x18ftypmp42\0\0\0\0mp41isom", 4, false, 0);
    assert!(Texture::inspect(&video).unwrap().video);
    assert!(matches!(
        Texture::decode(&video),
        Err(SceneAssetError::Unsupported { .. })
    ));
    let mut unknown = tex(b"raw pixels", 4, false, 0);
    unknown[18..22].copy_from_slice(&99_u32.to_le_bytes());
    unknown[59..63].copy_from_slice(&u32::MAX.to_le_bytes());
    assert!(matches!(
        Texture::decode(&unknown),
        Err(SceneAssetError::Unsupported { .. })
    ));
}

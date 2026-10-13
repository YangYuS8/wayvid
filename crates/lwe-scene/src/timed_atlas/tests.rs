use std::fs;
use std::path::PathBuf;

use super::*;
use crate::SceneAssets;

fn blocks() -> Vec<u8> {
    // Six blocks: both alpha modes plus equality, varied RGB565 endpoints in
    // both orders and all selectors. Pixels within every block are nonuniform.
    let mut raw = Vec::new();
    for i in 0..6_u16 {
        let (a, b) = match i % 3 {
            0 => (233, 19),
            1 => (11, 199),
            _ => (77, 77),
        };
        let mut block = [0_u8; 16];
        block[0] = a;
        block[1] = b;
        let alpha = (0..16).fold(0_u64, |bits, pixel| {
            bits | ((((pixel + i as usize) % 8) as u64) << (pixel * 3))
        });
        block[2..8].copy_from_slice(&alpha.to_le_bytes()[..6]);
        let (c0, c1) = if i % 2 == 0 {
            (0x9b73 + i, 0x2125 + i)
        } else {
            (0x1234 + i, 0xdabe - i)
        };
        block[8..10].copy_from_slice(&c0.to_le_bytes());
        block[10..12].copy_from_slice(&c1.to_le_bytes());
        let colors = (0..16).fold(0_u32, |bits, pixel| {
            bits | ((((pixel + i as usize) % 4) as u32) << (pixel * 2))
        });
        block[12..16].copy_from_slice(&colors.to_le_bytes());
        raw.extend(block);
    }
    raw
}

fn authored(raw: &[u8], compressed: bool, frames: &[(f32, [f32; 6])]) -> Vec<u8> {
    let payload = if compressed {
        lz4_flex::block::compress(raw)
    } else {
        raw.to_vec()
    };
    let mut bytes = b"TEXV0005\0TEXI0001\0".to_vec();
    for value in [4_u32, 7, 10, 7, 10, 7, 0] {
        bytes.extend(value.to_le_bytes());
    }
    bytes.extend(b"TEXB0004\0");
    for value in [
        1_u32,
        u32::MAX,
        0,
        1,
        10,
        7,
        u32::from(compressed),
        if compressed { raw.len() as u32 } else { 0 },
        payload.len() as u32,
    ] {
        bytes.extend(value.to_le_bytes());
    }
    bytes.extend(payload);
    bytes.extend(b"TEXS0003\0");
    for value in [frames.len() as u32, 2, 2] {
        bytes.extend(value.to_le_bytes());
    }
    for (duration, coordinates) in frames {
        bytes.extend(0_u32.to_le_bytes());
        bytes.extend(duration.to_le_bytes());
        for value in coordinates {
            bytes.extend(value.to_le_bytes());
        }
    }
    bytes
}

fn sample(compressed: bool) -> Vec<u8> {
    authored(&blocks(), compressed, &[(0.1, [0., 0., 2., 0., 0., 2.])])
}

fn source(bytes: &[u8], budget: &AtlasMemoryBudget) -> Result<TimedAtlasSource> {
    let mut encoded = OwnedBytes::zeroed(bytes.len(), budget)?;
    encoded.bytes.copy_from_slice(bytes);
    TimedAtlasSource::from_owned(
        encoded,
        ResourceOrigin::Project,
        &mut TextureInspectionBudget::default(),
        budget,
    )
}

fn budget() -> AtlasMemoryBudget {
    AtlasMemoryBudget::new(8 * 1024 * 1024).unwrap()
}

fn crop(full: &[u8], stride_width: u32, rect: PixelRect) -> Vec<u8> {
    let mut out = Vec::new();
    for row in rect.y..rect.y + rect.height {
        let begin = (row as usize * stride_width as usize + rect.x as usize) * 4;
        out.extend_from_slice(&full[begin..begin + rect.width as usize * 4]);
    }
    out
}

#[test]
fn all_block_phases_multiblock_rows_and_partial_edges_match_retained_dds() {
    let memory = budget();
    let atlas = source(&sample(true), &memory).unwrap();
    let oracle = crate::texture::decode_bc3_oracle(&blocks(), 10, 7).unwrap();
    let mut comparisons = 0;
    for y in 0..7 {
        for x in 0..10 {
            for width in [1, 3, 4, 5, 10 - x] {
                for height in [1, 3, 4, 5, 7 - y] {
                    if x + width > 10 || y + height > 7 {
                        continue;
                    }
                    let rect = PixelRect {
                        x,
                        y,
                        width,
                        height,
                    };
                    let out = atlas.decode_region(rect).unwrap();
                    assert_eq!(out.rgba(), crop(&oracle, 10, rect), "{rect:?}");
                    assert_eq!(out.rgba().len(), width as usize * height as usize * 4);
                    assert_eq!(out.capacity_bytes(), out.rgba().len());
                    assert_eq!(out.layout().block_origin(), [x / 4, y / 4]);
                    assert_eq!(
                        out.layout().block_dimensions(),
                        [
                            (x + width).div_ceil(4) - x / 4,
                            (y + height).div_ceil(4) - y / 4
                        ]
                    );
                    comparisons += 1;
                }
            }
        }
    }
    assert!(comparisons > 500);
    assert_eq!(
        memory.usage().charged_bytes,
        atlas.retained_capacity_bytes()
    );
    drop(atlas);
    assert_eq!(memory.usage().charged_bytes, 0);
}

#[test]
fn every_rgb565_endpoint_and_both_alpha_modes_match_dds_rounding() {
    // Endpoint expansion checks every 16-bit color, rather than assuming bit
    // replication matches the existing decoder. Alpha selectors cover all 8.
    let mut raw = Vec::new();
    for color in 0..=u16::MAX {
        let mut block = [0_u8; 16];
        block[0] = (color & 255) as u8;
        block[1] = (color >> 8) as u8;
        block[2..8].copy_from_slice(&0x00fa_c688_fac6_u64.to_le_bytes()[..6]);
        block[8..10].copy_from_slice(&color.to_le_bytes());
        block[10..12].copy_from_slice(&(!color).to_le_bytes());
        block[12..16].copy_from_slice(&0xe4e4_e4e4_u32.to_le_bytes());
        raw.extend(block);
    }
    // 256 x 256 blocks => 1024 x 1024 pixels, a small authored oracle.
    let layout = Bc3RegionLayout::checked(
        1024,
        1024,
        PixelRect {
            x: 0,
            y: 0,
            width: 1024,
            height: 1024,
        },
    )
    .unwrap();
    let mut rgba = vec![0; layout.rgba_bytes()];
    decode_region_into(&raw, &layout, &mut rgba).unwrap();
    assert_eq!(
        rgba,
        crate::texture::decode_bc3_oracle(&raw, 1024, 1024).unwrap()
    );
}

#[test]
fn layout_rejects_zero_oob_overflow_and_oversized_extents() {
    for rect in [
        PixelRect {
            x: 0,
            y: 0,
            width: 0,
            height: 1,
        },
        PixelRect {
            x: 0,
            y: 0,
            width: 1,
            height: 0,
        },
        PixelRect {
            x: 9,
            y: 6,
            width: 2,
            height: 1,
        },
        PixelRect {
            x: 9,
            y: 6,
            width: 1,
            height: 2,
        },
        PixelRect {
            x: u32::MAX,
            y: 0,
            width: 1,
            height: 1,
        },
        PixelRect {
            x: 0,
            y: u32::MAX,
            width: 1,
            height: 1,
        },
    ] {
        assert!(Bc3RegionLayout::checked(10, 7, rect).is_err());
    }
    for dimensions in [
        (0, 7),
        (10, 0),
        (16_385, 4),
        (16_384, 16_384),
        (u32::MAX, 1),
    ] {
        assert!(
            Bc3RegionLayout::checked(
                dimensions.0,
                dimensions.1,
                PixelRect {
                    x: 0,
                    y: 0,
                    width: 1,
                    height: 1
                }
            )
            .is_err()
        );
    }
    let layout = Bc3RegionLayout::checked(
        10,
        7,
        PixelRect {
            x: 3,
            y: 3,
            width: 5,
            height: 4,
        },
    )
    .unwrap();
    assert_eq!(layout.source_bytes(), 96);
    assert_eq!(layout.source_block_row_stride(), 48);
    let mut out = vec![0; layout.rgba_bytes()];
    assert!(decode_region_into(&blocks()[..95], &layout, &mut out).is_err());
    let mut too_long = blocks();
    too_long.push(0);
    assert!(decode_region_into(&too_long, &layout, &mut out).is_err());
    assert!(decode_region_into(&blocks(), &layout, &mut out[..79]).is_err());
}

#[test]
fn whole_frame_table_and_known_flags_are_required_before_lz4_decode() {
    let good = sample(true);
    let animation = good
        .windows(9)
        .position(|bytes| bytes == b"TEXS0003\0")
        .unwrap();
    let frame = animation + 21;
    for (offset, value) in [
        (18, 0_u32),
        (22, 2),
        (22, 0xffff),
        (63, 1),
        (frame, 1),
        (frame + 4, 0_f32.to_bits()),
        (frame + 4, f32::NAN.to_bits()),
        (frame + 8, (-1_f32).to_bits()),
        (frame + 8, 0.5_f32.to_bits()),
        (frame + 16, 0_f32.to_bits()),
        (frame + 20, 1_f32.to_bits()),
        (frame + 24, 1_f32.to_bits()),
        (frame + 28, 3_f32.to_bits()),
    ] {
        let mut bad = good.clone();
        bad[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
        let memory = budget();
        assert!(
            source(&bad, &memory).is_err(),
            "accepted modified field {offset}"
        );
        assert_eq!(memory.usage().charged_bytes, 0);
    }
    // Last frame invalid: validating only frame0 would miss this.
    let bad = authored(
        &blocks(),
        true,
        &[
            (0.1, [0., 0., 2., 0., 0., 2.]),
            (0.1, [9., 6., 2., 0., 0., 2.]),
        ],
    );
    let memory = budget();
    assert!(source(&bad, &memory).is_err());
    assert_eq!(memory.usage().charged_bytes, 0);
    for length in 0..good.len() {
        let memory = budget();
        assert!(
            source(&good[..length], &memory).is_err(),
            "truncation {length}"
        );
        assert_eq!(memory.usage().charged_bytes, 0);
    }
    let mut trailing = good;
    trailing.push(0);
    assert!(source(&trailing, &budget()).is_err());
}

#[test]
fn exact_raw_and_declared_lz4_lengths_are_checked_and_failures_release_budget() {
    for length in [95, 97] {
        let mut raw = blocks();
        raw.resize(length, 0);
        assert!(
            source(
                &authored(&raw, false, &[(0.1, [0., 0., 2., 0., 0., 2.])]),
                &budget()
            )
            .is_err()
        );
    }
    for declared in [95_u32, 97, u32::MAX] {
        let mut bytes = sample(true);
        bytes[83..87].copy_from_slice(&declared.to_le_bytes());
        let memory = budget();
        assert!(source(&bytes, &memory).is_err());
        assert_eq!(memory.usage().charged_bytes, 0);
    }
    let mut malformed = sample(true);
    let stored_size = u32::from_le_bytes(malformed[87..91].try_into().unwrap()) as usize;
    malformed[91..91 + stored_size].fill(255);
    let memory = budget();
    assert!(source(&malformed, &memory).is_err());
    assert_eq!(memory.usage().charged_bytes, 0);
    let raw = source(&sample(false), &memory).unwrap();
    assert_eq!(raw.raw_capacity(), 0); // raw is a borrowed range of sealed bytes.
    assert_eq!(
        raw.decode_frame(0).unwrap().rgba(),
        crop(
            &crate::texture::decode_bc3_oracle(&blocks(), 10, 7).unwrap(),
            10,
            raw.frames()[0].rect
        )
    );
}

#[test]
fn valid_but_unmodeled_mips_images_and_animation_versions_stay_rejected() {
    let good = sample(false);
    let animation = 91 + blocks().len();
    let mut extra_mip = Vec::new();
    for value in [2_u32, 2, 0, 0, 16] {
        extra_mip.extend(value.to_le_bytes());
    }
    extra_mip.extend(&blocks()[..16]);
    let mut multiple_mips = good.clone();
    multiple_mips[67..71].copy_from_slice(&2_u32.to_le_bytes());
    multiple_mips.splice(animation..animation, extra_mip);
    assert_eq!(Texture::inspect(&multiple_mips).unwrap().mips.len(), 2);
    assert!(matches!(
        source(&multiple_mips, &budget()),
        Err(SceneAssetError::Unsupported { .. })
    ));

    let mut multiple_images = good.clone();
    multiple_images[55..59].copy_from_slice(&2_u32.to_le_bytes());
    multiple_images.splice(animation..animation, good[67..animation].iter().copied());
    assert_eq!(Texture::inspect(&multiple_images).unwrap().images, 2);
    assert!(matches!(
        source(&multiple_images, &budget()),
        Err(SceneAssetError::Unsupported { .. })
    ));

    let mut particle = good[..animation].to_vec();
    particle.extend(b"TEXS0002\0");
    particle.extend(1_u32.to_le_bytes());
    particle.extend(0_u32.to_le_bytes());
    particle.extend(0_f32.to_le_bytes());
    for value in [0_f32, 0., 2., 0., 0., 2.] {
        particle.extend(value.to_le_bytes());
    }
    assert!(Texture::decode_atlas(&particle).is_ok());
    assert!(matches!(
        source(&particle, &budget()),
        Err(SceneAssetError::Unsupported { .. })
    ));

    for (offset, value) in [
        (26, 12_u32),
        (animation + 13, 3),
        (animation + 9, 0),
        (animation + 9, 4097),
    ] {
        let mut changed = good.clone();
        changed[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
        assert!(source(&changed, &budget()).is_err());
    }
    assert!(
        Bc3RegionLayout::checked(
            8192,
            8192,
            PixelRect {
                x: 8191,
                y: 8191,
                width: 1,
                height: 1
            }
        )
        .is_ok()
    );
    assert!(
        Bc3RegionLayout::checked(
            8193,
            8192,
            PixelRect {
                x: 0,
                y: 0,
                width: 1,
                height: 1
            }
        )
        .is_err()
    );
}

#[test]
fn opaque_image_info_is_retained_in_sealed_identity_without_invented_semantics() {
    let bytes = sample(false);
    let mut opaque = bytes.clone();
    opaque[42..46].copy_from_slice(&4_290_032_704_u32.to_le_bytes());
    let memory = budget();
    let original = source(&bytes, &memory).unwrap();
    let changed = source(&opaque, &memory).unwrap();
    assert_eq!(changed.metadata().image_info_extra, 4_290_032_704);
    assert_ne!(changed.encoded_sha256(), original.encoded_sha256());
    assert_eq!(
        changed.decode_frame(0).unwrap().rgba(),
        original.decode_frame(0).unwrap().rgba()
    );
}

#[test]
fn timeline_uses_original_durations_and_strict_half_open_boundaries() {
    let bytes = authored(
        &blocks(),
        true,
        &[
            (0.125, [0., 0., 2., 0., 0., 2.]),
            (0.25, [4., 0., 2., 0., 0., 2.]),
            (0.5, [8., 0., 2., 0., 0., 2.]),
        ],
    );
    let atlas = source(&bytes, &budget()).unwrap();
    assert_eq!(atlas.cycle_seconds(), 0.875);
    for (time, index) in [
        (0., 0),
        (0.125 - f64::EPSILON, 0),
        (0.125, 1),
        (0.375 - f64::EPSILON, 1),
        (0.375, 2),
        (0.875 - f64::EPSILON, 2),
        (0.875, 0),
        (875_000.125, 1),
    ] {
        assert_eq!(atlas.frame_at(time).unwrap(), index, "{time}");
    }
    for time in [-1., f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(atlas.frame_at(time).is_err());
    }
    assert!(atlas.decode_frame(3).is_err());
    let collapsed = authored(
        &blocks(),
        false,
        &[
            (f32::MAX, [0., 0., 2., 0., 0., 2.]),
            (0.1, [4., 0., 2., 0., 0., 2.]),
        ],
    );
    assert!(source(&collapsed, &budget()).is_err());
}

#[test]
fn neighborhood_reads_original_neighbor_pixels_and_clips_only_at_atlas_edge() {
    let bytes = authored(
        &blocks(),
        false,
        &[
            (0.1, [4., 3., 2., 0., 0., 2.]),
            (0.1, [0., 0., 2., 0., 0., 2.]),
            (0.1, [8., 5., 2., 0., 0., 2.]),
        ],
    );
    let atlas = source(&bytes, &budget()).unwrap();
    let full = crate::texture::decode_bc3_oracle(&blocks(), 10, 7).unwrap();
    for (index, rect) in [
        (
            0,
            PixelRect {
                x: 3,
                y: 2,
                width: 4,
                height: 4,
            },
        ),
        (
            1,
            PixelRect {
                x: 0,
                y: 0,
                width: 3,
                height: 3,
            },
        ),
        (
            2,
            PixelRect {
                x: 7,
                y: 4,
                width: 3,
                height: 3,
            },
        ),
    ] {
        let layout = atlas.frame_neighborhood(index, 1).unwrap();
        assert_eq!(layout.rect(), rect);
        assert_eq!(
            atlas.decode_region(rect).unwrap().rgba(),
            crop(&full, 10, rect)
        );
    }
    assert!(atlas.frame_neighborhood(0, u32::MAX).is_err());
}

#[test]
fn source_and_region_clones_hold_real_allocations_until_last_owner() {
    let memory = budget();
    let bytes = sample(true);
    let atlas = source(&bytes, &memory).unwrap();
    let retained = atlas.retained_capacity_bytes();
    assert_eq!(memory.usage().charged_bytes, retained);
    let shared = atlas.clone();
    assert_eq!(memory.usage().charged_bytes, retained);
    let second = source(&bytes, &memory).unwrap();
    assert_eq!(atlas.encoded_sha256(), second.encoded_sha256());
    assert_eq!(memory.usage().charged_bytes, retained * 2);
    drop(second);
    let out = atlas.decode_frame(0).unwrap();
    let out_capacity = out.capacity_bytes();
    let out_shared = out.clone();
    assert_eq!(memory.usage().charged_bytes, retained + out_capacity);
    drop(atlas);
    drop(shared);
    assert_eq!(memory.usage().charged_bytes, out_capacity);
    drop(out);
    assert_eq!(memory.usage().charged_bytes, out_capacity);
    drop(out_shared);
    assert_eq!(memory.usage().charged_bytes, 0);
}

#[test]
fn exact_decode_reservation_cap_and_failed_jobs_preserve_existing_owners() {
    let memory = budget();
    let atlas = source(&sample(false), &memory).unwrap();
    let previous = atlas.decode_frame(0).unwrap();
    let pixels = previous.rgba().to_vec();
    let layout = atlas.frame_layout(0).unwrap();
    let needed = layout.decode_reservation().unwrap();
    let used = memory.usage().charged_bytes;
    let padding = memory.reserve(memory.limit - used - needed + 1).unwrap();
    assert!(matches!(
        atlas.decode_frame(0),
        Err(SceneAssetError::LimitExceeded { .. })
    ));
    assert!(!memory.usage().active_decode_job);
    assert_eq!(memory.usage().charged_bytes, used + padding.bytes);
    assert_eq!(previous.rgba(), pixels);
    drop(padding);
    let padding = memory.reserve(memory.limit - used - needed).unwrap();
    let accepted = atlas.decode_frame(0).unwrap();
    assert_eq!(accepted.rgba(), previous.rgba());
    assert_eq!(memory.usage().peak_charged_bytes, memory.limit);
    drop(accepted);
    drop(padding);
    let _job = memory.begin_decode().unwrap();
    assert!(matches!(
        atlas.decode_frame(0),
        Err(SceneAssetError::Unsupported { .. })
    ));
    assert_eq!(previous.rgba(), pixels);
    assert!(AtlasMemoryBudget::new(MAX_ASSET_BYTES + 1).is_err());
    assert!(AtlasMemoryBudget::new(0).is_err());
}

#[test]
fn descriptor_sealed_bytes_hash_and_framing_share_one_read_after_path_replacement() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("atlas.tex");
    let before = sample(true);
    fs::write(&path, &before).unwrap();
    let assets = SceneAssets::open(dir.path(), None).unwrap();
    let descriptor = assets.resource("atlas.tex").unwrap();
    let mut changed = before.clone();
    let duration = frame_duration_offset(&changed);
    changed[duration..duration + 4].copy_from_slice(&0.25_f32.to_le_bytes());
    let replacement = dir.path().join("replacement");
    fs::write(&replacement, &changed).unwrap();
    fs::rename(replacement, &path).unwrap();
    let memory = budget();
    let mut inspection = TextureInspectionBudget::default();
    let old = TimedAtlasSource::from_resource(&descriptor, &mut inspection, &memory).unwrap();
    assert_eq!(old.encoded_bytes(), before);
    assert_eq!(
        old.encoded_sha256(),
        <[u8; 32]>::from(Sha256::digest(&before))
    );
    assert_eq!(old.origin(), ResourceOrigin::Project);
    assert_eq!(inspection.remaining_mips, 1024 * 32 - 1);
    assert_eq!(inspection.remaining_animation_frames, 4095);
    let fresh = TimedAtlasSource::from_resource(
        &assets.resource("atlas.tex").unwrap(),
        &mut inspection,
        &memory,
    )
    .unwrap();
    assert_eq!(fresh.encoded_bytes(), changed);
    assert_ne!(old.encoded_sha256(), fresh.encoded_sha256());
    assert_eq!(fresh.cycle_seconds(), 0.25);
    assert!(Texture::decode(old.encoded_bytes()).is_err());
    assert!(Texture::decode_atlas(old.encoded_bytes()).is_err());
}

fn frame_duration_offset(bytes: &[u8]) -> usize {
    bytes.windows(9).position(|v| v == b"TEXS0003\0").unwrap() + 25
}

#[test]
fn framing_and_resource_limits_fail_before_large_allocations() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("atlas.tex");
    fs::write(&path, sample(true)).unwrap();
    let assets = SceneAssets::open(dir.path(), None).unwrap();
    let descriptor = assets.resource("atlas.tex").unwrap();
    let memory = budget();
    for mut inspection in [
        TextureInspectionBudget {
            remaining_mips: 0,
            remaining_animation_frames: 1,
        },
        TextureInspectionBudget {
            remaining_mips: 1,
            remaining_animation_frames: 0,
        },
    ] {
        assert!(TimedAtlasSource::from_resource(&descriptor, &mut inspection, &memory).is_err());
        assert_eq!(memory.usage().charged_bytes, 0);
    }
    let file = fs::File::create(dir.path().join("large.tex")).unwrap();
    file.set_len(MAX_TEXTURE_BYTES as u64 + 1).unwrap();
    let before = memory.usage();
    assert!(matches!(
        TimedAtlasSource::from_resource(
            &assets.resource("large.tex").unwrap(),
            &mut TextureInspectionBudget::default(),
            &memory
        ),
        Err(SceneAssetError::LimitExceeded { .. })
    ));
    assert_eq!(memory.usage(), before);
    assert!(assets.resource("../atlas.tex").is_err());
}

/// Private original-project oracle. No author assets enter the repository, and
/// no renderer/Wayland/backend/desktop state is touched. One atlas at a time.
#[test]
#[ignore = "requires private original 3420277613 assets and explicit CPU-only opt-in"]
fn original_two_atlases_sixteen_frames_and_neighbor_pixels_match_dds() {
    assert_eq!(std::env::var("LWE_TIMED_ATLAS_ORACLE").as_deref(), Ok("1"));
    let project = PathBuf::from(std::env::var_os("LWE_TIMED_ATLAS_PROJECT").expect("project path"));
    let assets_dir = std::env::var_os("LWE_TIMED_ATLAS_ASSETS").map(PathBuf::from);
    let assets = SceneAssets::open(&project, assets_dir.as_deref()).unwrap();
    let paths = ["scene.pkg", "project.json"];
    let before: Vec<_> = paths
        .iter()
        .map(|path| Sha256::digest(fs::read(project.join(path)).unwrap()).to_vec())
        .collect();
    assert_eq!(
        hex(&before[0]),
        "c55b26c8b5eedba6029a71239198d9001089b6e3c01fa689b3c40eeb945e53a6"
    );
    assert_eq!(
        hex(&before[1]),
        "9f83b7b648e318a493c62b7132880ed1cd0031cff0c4a824c54077c4a4142f21"
    );
    let memory = AtlasMemoryBudget::new(MAX_ASSET_BYTES).unwrap();
    let mut results = Vec::new();
    for (name, expected_hash) in [
        (
            "materials/Glitch4cut.tex",
            "5dc96f132337b95450258f20458c6d1df0bf9b5c907eb5aa805604443d16d6ec",
        ),
        (
            "materials/SwitchGlitch4cut.tex",
            "7a379e64dbe74eddd52e1dfce6dc0dc2b0d9741b25103d01bf45d4f17b61ef04",
        ),
    ] {
        let descriptor = assets.resource(name).unwrap();
        println!(
            "original framing {name}: {:?}",
            Texture::inspect_resource(&descriptor).unwrap()
        );
        let atlas = TimedAtlasSource::from_resource(
            &descriptor,
            &mut TextureInspectionBudget::default(),
            &memory,
        )
        .unwrap();
        if !expected_hash.is_empty() {
            assert_eq!(hex(&atlas.encoded_sha256()), expected_hash);
        }
        assert_eq!(atlas.origin(), ResourceOrigin::Package);
        assert_eq!(atlas.frames().len(), 8);
        assert_eq!(atlas.raw_capacity(), 52_488_000);
        assert_eq!(atlas.metadata().animation.as_ref().unwrap().version, 3);
        // This deliberately large test-only DDS reference is outside the
        // production ledger. Its separate Vec capacity is recorded explicitly.
        let raw = match &atlas.0.raw {
            RawMip::Decompressed(bytes) => &bytes.bytes,
            _ => panic!("original is compressed"),
        };
        let oracle = crate::texture::decode_bc3_oracle(raw, 8100, 6480).unwrap();
        assert_eq!(oracle.len(), 209_952_000);
        let source_capacity = atlas.retained_capacity_bytes();
        let mut frames = Vec::new();
        for index in 0..8 {
            let rect = atlas.frames()[index].rect;
            assert_eq!(
                rect,
                PixelRect {
                    x: (index as u32 % 3) * 2700,
                    y: (index as u32 / 3) * 2160,
                    width: 2700,
                    height: 2160
                }
            );
            assert_eq!(atlas.frames()[index].duration_seconds, 0.1_f32 as f64);
            let decoded = atlas.decode_frame(index).unwrap();
            assert_crop_eq(decoded.rgba(), &oracle, 8100, rect);
            let frame_hash = hex(&Sha256::digest(decoded.rgba()));
            let frame_capacity = decoded.capacity_bytes();
            drop(decoded);
            let neighbors = atlas.frame_neighborhood(index, 1).unwrap().rect();
            let decoded = atlas.decode_region(neighbors).unwrap();
            assert_crop_eq(decoded.rgba(), &oracle, 8100, neighbors);
            let neighborhood_hash = hex(&Sha256::digest(decoded.rgba()));
            frames.push(serde_json::json!({"index":index,"rect":[rect.x,rect.y,rect.width,rect.height],"rgba_capacity":frame_capacity,"rgba_sha256":frame_hash,"neighborhood_rect":[neighbors.x,neighbors.y,neighbors.width,neighbors.height],"neighborhood_capacity":decoded.capacity_bytes(),"neighborhood_sha256":neighborhood_hash}));
            drop(decoded);
            assert_eq!(memory.usage().charged_bytes, source_capacity);
        }
        let metadata = atlas.metadata();
        results.push(serde_json::json!({"resource":name,"encoded_sha256":hex(&atlas.encoded_sha256()),"encoded_capacity":atlas.encoded_capacity(),"raw_capacity":atlas.raw_capacity(),"source_retained_capacity":source_capacity,"cycle_seconds":atlas.cycle_seconds(),"dds_reference_capacity_outside_production_ledger":oracle.capacity(),"framing":{"version":metadata.version,"body_version":metadata.body_version,"format":metadata.format,"flags":metadata.flags,"storage_dimensions":[metadata.storage_width,metadata.storage_height],"image_dimensions":[metadata.image_width,metadata.image_height],"image_info_extra_opaque":metadata.image_info_extra,"container_format":metadata.container_format,"body_extra":metadata.body_extra,"images":metadata.images,"mip_count":metadata.mips.len(),"mip_dimensions":[metadata.mips[0].width,metadata.mips[0].height],"animation_version":metadata.animation.as_ref().unwrap().version},"frames":frames}));
        drop(oracle);
        drop(atlas);
        assert_eq!(memory.usage().charged_bytes, 0);
    }
    let after: Vec<_> = paths
        .iter()
        .map(|path| Sha256::digest(fs::read(project.join(path)).unwrap()).to_vec())
        .collect();
    assert_eq!(before, after);
    let report = serde_json::json!({"schema":"lwe.timed_atlas_cpu_oracle.v1","project":project,"package_sha256_before_after":hex(&before[0]),"project_json_sha256_before_after":hex(&before[1]),"results":results,"production_cpu_peak_charged_bytes":memory.usage().peak_charged_bytes,"codec_workspace_reservation":size_of::<Bc3Scratch>(),"proof":"CPU-only exact equality against retained image DDS; no Windows, native sampling, GL, desktop or342 admission"});
    if let Some(output) = std::env::var_os("LWE_TIMED_ATLAS_ORACLE_REPORT") {
        fs::write(output, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
    }
    println!("{}", serde_json::to_string_pretty(&report).unwrap());
}

fn assert_crop_eq(region: &[u8], full: &[u8], width: u32, rect: PixelRect) {
    for row in 0..rect.height as usize {
        let start = ((rect.y as usize + row) * width as usize + rect.x as usize) * 4;
        let count = rect.width as usize * 4;
        assert_eq!(
            &region[row * count..(row + 1) * count],
            &full[start..start + count],
            "ROI {rect:?} row {row}"
        );
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

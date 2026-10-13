use std::io::{Cursor, Read};

use image::{ImageDecoder, ImageFormat, ImageReader, codecs::dds::DdsDecoder};

use crate::{
    MAX_ASSET_BYTES, MAX_TEXTURE_BYTES, MAX_TEXTURE_DIMENSION, Result, SceneAssetError,
    SceneResource, limit,
};

const MAX_IMAGES: u32 = 1024;
const MAX_MIPS: u32 = 32;
const MAX_ANIMATION_FRAMES: u32 = 4096;

/// Aggregate framing declarations consume this budget before loops/allocations,
/// including when a later field is malformed.
#[derive(Debug)]
pub struct TextureInspectionBudget {
    pub remaining_mips: usize,
    pub remaining_animation_frames: usize,
}

impl Default for TextureInspectionBudget {
    fn default() -> Self {
        Self {
            remaining_mips: MAX_IMAGES as usize * MAX_MIPS as usize,
            remaining_animation_frames: MAX_ANIMATION_FRAMES as usize,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MipMetadata {
    pub image: u32,
    pub level: u32,
    pub width: u32,
    pub height: u32,
    pub compressed: bool,
    pub decompressed_size: u32,
    pub stored_size: u32,
    pub offset: usize,
    pub video: bool,
}

/// Frame image and pixel-space affine UVs: origin.xy, horizontal.xy, vertical.xy.
/// TEXS0002 particle atlases use zero duration; TEXS0003 carries timed frames.
#[derive(Debug, Clone, PartialEq)]
pub struct AnimationFrame {
    pub image: u32,
    pub duration: f32,
    pub coordinates: [f32; 6],
}

#[derive(Debug, Clone, PartialEq)]
pub struct AnimationMetadata {
    pub version: u32,
    pub width: u32,
    pub height: u32,
    pub frames: Vec<AnimationFrame>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextureMetadata {
    pub version: u32,
    pub body_version: u32,
    /// The first TEXI u32 is the format; the second is flags.
    pub format: u32,
    pub flags: u32,
    pub storage_width: u32,
    pub storage_height: u32,
    pub image_width: u32,
    pub image_height: u32,
    pub image_info_extra: u32,
    pub container_format: i32,
    pub body_extra: Option<u32>,
    pub images: u32,
    pub mips: Vec<MipMetadata>,
    pub animation: Option<AnimationMetadata>,
    pub video: bool,
}

#[derive(Debug, Clone)]
pub struct Texture {
    /// Dimensions of this RGBA buffer, which can differ from TEXI storage dimensions.
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
    pub metadata: TextureMetadata,
}

impl Texture {
    /// Inspect bounded framing without allocating decoded images or video frames.
    pub fn inspect(bytes: &[u8]) -> Result<TextureMetadata> {
        parse_source(Source::Bytes(bytes))
    }

    /// Inspect one sealed byte buffer with the caller's aggregate framing budget.
    pub fn inspect_bounded(
        bytes: &[u8],
        budget: &mut TextureInspectionBudget,
    ) -> Result<TextureMetadata> {
        parse_source_bounded(Source::Bytes(bytes), budget)
    }

    /// Inspect framing via small positional reads, skipping mip payloads. This
    /// does not decode pixels or validate compressed/image/video payloads.
    pub fn inspect_resource(resource: &SceneResource) -> Result<TextureMetadata> {
        parse_source(Source::Resource(resource))
    }

    pub fn inspect_resource_bounded(
        resource: &SceneResource,
        budget: &mut TextureInspectionBudget,
    ) -> Result<TextureMetadata> {
        parse_source_bounded(Source::Resource(resource), budget)
    }

    /// Decode a static first mip. Animated and video textures are explicit errors.
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        Self::decode_inner(bytes, false)
    }

    /// Decode a single-image particle atlas while retaining its validated frame
    /// table. Consumers must select the frame UVs; this is never a static-image
    /// fallback for an animated image layer.
    pub fn decode_atlas(bytes: &[u8]) -> Result<Self> {
        Self::decode_inner(bytes, true)
    }

    fn decode_inner(bytes: &[u8], allow_atlas: bool) -> Result<Self> {
        let metadata = parse(bytes)?;
        if let Some(animation) = &metadata.animation {
            if !allow_atlas || animation.version != 2 {
                return Err(unsupported("TEXS animated texture playback"));
            }
            if metadata.images != 1 || animation.frames.iter().any(|frame| frame.image != 0) {
                return Err(unsupported("multi-image particle atlas"));
            }
            for frame in &animation.frames {
                let [x, y, width, shear_y, shear_x, height] = frame.coordinates;
                if frame.duration != 0.0
                    || shear_y != 0.0
                    || shear_x != 0.0
                    || width <= 0.0
                    || height <= 0.0
                    || x < 0.0
                    || y < 0.0
                    || x + width > metadata.storage_width as f32
                    || y + height > metadata.storage_height as f32
                {
                    return Err(unsupported("nonrectangular or timed particle atlas"));
                }
            }
        }
        if metadata.video {
            return Err(unsupported("video texture playback"));
        }
        if metadata.images != 1 {
            return Err(unsupported("multi-image texture"));
        }
        if metadata.body_extra.is_some_and(|extra| extra != 0) {
            return Err(unsupported("nonzero TEXB0004 extra field"));
        }
        let mip = &metadata.mips[0];
        let stored = &bytes[mip.offset..mip.offset + mip.stored_size as usize];
        let decompressed;
        let data = if mip.compressed {
            decompressed = lz4_flex::block::decompress(stored, mip.decompressed_size as usize)
                .map_err(|e| invalid(format!("invalid LZ4 mip: {e}")))?;
            if decompressed.len() != mip.decompressed_size as usize {
                return Err(invalid("LZ4 mip size differs from declaration"));
            }
            decompressed.as_slice()
        } else {
            stored
        };
        let format = if data.starts_with(b"\x89PNG\r\n\x1a\n") {
            Some(ImageFormat::Png)
        } else if data.starts_with(b"\xff\xd8\xff") {
            Some(ImageFormat::Jpeg)
        } else {
            None
        };
        if data.get(4..8) == Some(b"ftyp") {
            return Err(unsupported("video texture playback"));
        }
        let (width, height, rgba) = if let Some(format) = format {
            decode_image(data, format, mip.width, mip.height)?
        } else {
            if metadata.container_format != -1 {
                return Err(invalid(
                    "encoded texture has an unrecognized image signature",
                ));
            }
            (
                mip.width,
                mip.height,
                decode_raw(data, metadata.format, mip.width, mip.height)?,
            )
        };
        if metadata.animation.is_some()
            && (width != metadata.storage_width || height != metadata.storage_height)
        {
            return Err(unsupported(
                "particle atlas storage dimensions differ from decoded mip",
            ));
        }
        Ok(Self {
            width,
            height,
            rgba,
            metadata,
        })
    }
}

fn decode_raw(bytes: &[u8], format: u32, width: u32, height: u32) -> Result<Vec<u8>> {
    // Native FUN_14013a520 establishes byte counts 0=4, 8=2, 9=1.
    // Preserve the native channels, expanding absent channels as an RGBA sampler
    // would. The resource layer does not apply a grayscale/color interpretation.
    if format == 4 {
        return decode_bc3(bytes, width, height);
    }
    let channels = match format {
        0 => 4,
        8 => 2,
        9 => 1,
        _ => return Err(unsupported(format!("raw texture format {format}"))),
    };
    let pixels = width as usize * height as usize;
    if bytes.len() != pixels * channels {
        return Err(invalid(
            "raw texture byte count differs from format and dimensions",
        ));
    }
    if channels == 4 {
        return Ok(bytes.to_vec());
    }
    let mut rgba = Vec::with_capacity(pixels * 4);
    for native in bytes.chunks_exact(channels) {
        rgba.extend_from_slice(&[native[0], native.get(1).copied().unwrap_or(0), 0, 255]);
    }
    Ok(rgba)
}

fn decode_bc3(bytes: &[u8], width: u32, height: u32) -> Result<Vec<u8>> {
    // Native format4 storage is confirmed by dxt5/dxt5n asset sidecars.
    // Use image's DDS decoder rather than implementing the compression codec.
    let padded_width = width.div_ceil(4) * 4;
    let padded_height = height.div_ceil(4) * 4;
    dimensions(padded_width, padded_height)?;
    let expected = padded_width as usize / 4 * (padded_height as usize / 4) * 16;
    if bytes.len() != expected {
        return Err(invalid(
            "BC3 byte count differs from block-aligned dimensions",
        ));
    }
    // DDS_HEADER offsets include the four-byte DDS signature.
    let mut header = [0_u8; 128];
    header[..4].copy_from_slice(b"DDS ");
    for (offset, value) in [
        (4, 124_u32),
        (8, 0x0008_1007), // CAPS, HEIGHT, WIDTH, PIXELFORMAT, LINEARSIZE.
        (12, padded_height),
        (16, padded_width),
        (20, expected as u32),
        (76, 32),      // DDS_PIXELFORMAT size.
        (80, 4),       // DDPF_FOURCC.
        (108, 0x1000), // DDSCAPS_TEXTURE.
    ] {
        header[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }
    header[84..88].copy_from_slice(b"DXT5");
    let decoder = DdsDecoder::new(Cursor::new(header).chain(Cursor::new(bytes)))
        .map_err(|e| invalid(format!("invalid BC3 texture: {e}")))?;
    if decoder.dimensions() != (padded_width, padded_height)
        || decoder.total_bytes() != padded_width as u64 * padded_height as u64 * 4
    {
        return Err(invalid(
            "BC3 decoder output does not match declared dimensions",
        ));
    }
    let mut rgba = vec![0; decoder.total_bytes() as usize];
    decoder
        .read_image(&mut rgba)
        .map_err(|e| invalid(format!("invalid BC3 texture: {e}")))?;
    // The mature decoder requires 4x4 dimensions; crop padded edge blocks in
    // place to keep allocation bounded and preserve the declared mip extent.
    let source_stride = padded_width as usize * 4;
    let target_stride = width as usize * 4;
    for row in 0..height as usize {
        rgba.copy_within(
            row * source_stride..row * source_stride + target_stride,
            row * target_stride,
        );
    }
    rgba.truncate(width as usize * height as usize * 4);
    Ok(rgba)
}

#[cfg(test)]
pub(crate) fn decode_bc3_oracle(bytes: &[u8], width: u32, height: u32) -> Result<Vec<u8>> {
    decode_bc3(bytes, width, height)
}

fn parse(bytes: &[u8]) -> Result<TextureMetadata> {
    parse_source(Source::Bytes(bytes))
}

fn parse_source(source: Source<'_>) -> Result<TextureMetadata> {
    parse_source_bounded(source, &mut TextureInspectionBudget::default())
}

fn parse_source_bounded(
    source: Source<'_>,
    budget: &mut TextureInspectionBudget,
) -> Result<TextureMetadata> {
    let length = source.len();
    if length > MAX_ASSET_BYTES as u64 {
        return Err(limit("texture file", length, MAX_ASSET_BYTES as u64));
    }
    let mut reader = Reader { source, offset: 0 };
    reader.tag(b"TEXV0005\0")?;
    reader.tag(b"TEXI0001\0")?;
    let format = reader.u32()?;
    let flags = reader.u32()?;
    let storage_width = reader.u32()?;
    let storage_height = reader.u32()?;
    let image_width = reader.u32()?;
    let image_height = reader.u32()?;
    let image_info_extra = reader.u32()?;
    dimensions(storage_width, storage_height)?;
    dimensions(image_width, image_height)?;
    let body_version = match reader.read(9)?.as_slice() {
        b"TEXB0001\0" => 1,
        b"TEXB0002\0" => 2,
        b"TEXB0003\0" => 3,
        b"TEXB0004\0" => 4,
        _ => return Err(unsupported("texture body version")),
    };
    let images = reader.u32()?;
    if images == 0 || images > MAX_IMAGES {
        return Err(invalid("invalid texture image count"));
    }
    // TEXB0001 is verified against the built-in util/white.tex: it has no
    // image-format field and mip headers contain only width, height and size.
    let container_format = if body_version < 3 {
        -1
    } else {
        reader.u32()? as i32
    };
    let body_extra = if body_version == 4 {
        Some(reader.u32()?)
    } else {
        None
    };
    let mut mips = Vec::new();
    let mut video = false;
    for image in 0..images {
        let levels = reader.u32()?;
        if levels == 0 || levels > MAX_MIPS {
            return Err(invalid("invalid mip count"));
        }
        if levels as usize > budget.remaining_mips {
            return Err(limit(
                "texture inspection mips",
                levels as u64,
                budget.remaining_mips as u64,
            ));
        }
        budget.remaining_mips -= levels as usize;
        for level in 0..levels {
            let width = reader.u32()?;
            let height = reader.u32()?;
            dimensions(width, height)?;
            let compressed = if body_version == 1 {
                false
            } else {
                match reader.u32()? {
                    0 => false,
                    1 => true,
                    _ => return Err(unsupported("texture compression marker")),
                }
            };
            let decompressed_size = if body_version == 1 { 0 } else { reader.u32()? };
            let stored_size = reader.u32()?;
            if stored_size == 0 {
                return Err(invalid("empty texture mip"));
            }
            if compressed && decompressed_size == 0 {
                return Err(invalid("compressed mip has zero decompressed size"));
            }
            if decompressed_size as usize > MAX_TEXTURE_BYTES {
                return Err(limit(
                    "decompressed mip",
                    decompressed_size as u64,
                    MAX_TEXTURE_BYTES as u64,
                ));
            }
            let offset = reader.offset;
            let payload = reader.read((stored_size as usize).min(8))?;
            reader.skip(stored_size as usize - payload.len())?;
            let is_video = payload.get(4..8) == Some(b"ftyp");
            video |= is_video;
            mips.push(MipMetadata {
                image,
                level,
                width,
                height,
                compressed,
                decompressed_size,
                stored_size,
                offset,
                video: is_video,
            });
        }
    }
    let animation = if (reader.offset as u64) < length {
        let version = match reader.read(9)?.as_slice() {
            b"TEXS0002\0" => 2,
            b"TEXS0003\0" => 3,
            _ => return Err(unsupported("texture animation version")),
        };
        let count = reader.u32()?;
        if count == 0 || count > MAX_ANIMATION_FRAMES {
            return Err(invalid("invalid texture animation frame count"));
        }
        if count as usize > budget.remaining_animation_frames {
            return Err(limit(
                "texture inspection animation frames",
                count as u64,
                budget.remaining_animation_frames as u64,
            ));
        }
        budget.remaining_animation_frames -= count as usize;
        let (width, height) = if version == 3 {
            (reader.u32()?, reader.u32()?)
        } else {
            (storage_width, storage_height)
        };
        dimensions(width, height)?;
        let mut frames = Vec::with_capacity(count as usize);
        for _ in 0..count {
            let image = reader.u32()?;
            if image >= images {
                return Err(invalid("animation image index outside texture"));
            }
            let duration = reader.f32()?;
            if !duration.is_finite() || duration < 0.0 || (version == 3 && duration == 0.0) {
                return Err(invalid("invalid animation frame duration"));
            }
            let mut coordinates = [0.0; 6];
            for coordinate in &mut coordinates {
                *coordinate = reader.f32()?;
                if !coordinate.is_finite() {
                    return Err(invalid("non-finite animation frame coordinate"));
                }
            }
            frames.push(AnimationFrame {
                image,
                duration,
                coordinates,
            });
        }
        Some(AnimationMetadata {
            version,
            width,
            height,
            frames,
        })
    } else {
        None
    };
    if reader.offset as u64 != length {
        return Err(invalid("unrecognized data after texture framing"));
    }
    Ok(TextureMetadata {
        version: 5,
        body_version,
        format,
        flags,
        storage_width,
        storage_height,
        image_width,
        image_height,
        image_info_extra,
        container_format,
        body_extra,
        images,
        mips,
        animation,
        video,
    })
}

fn dimensions(width: u32, height: u32) -> Result<()> {
    if width == 0 || height == 0 {
        return Err(invalid("zero texture dimension"));
    }
    if width > MAX_TEXTURE_DIMENSION || height > MAX_TEXTURE_DIMENSION {
        return Err(limit(
            "texture dimension",
            width.max(height) as u64,
            MAX_TEXTURE_DIMENSION as u64,
        ));
    }
    let rgba_size = width as u64 * height as u64 * 4;
    if rgba_size > MAX_TEXTURE_BYTES as u64 {
        return Err(limit("RGBA texture", rgba_size, MAX_TEXTURE_BYTES as u64));
    }
    Ok(())
}

fn decode_image(
    bytes: &[u8],
    format: ImageFormat,
    width: u32,
    height: u32,
) -> Result<(u32, u32, Vec<u8>)> {
    let mut reader = ImageReader::with_format(Cursor::new(bytes), format);
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(width);
    limits.max_image_height = Some(height);
    limits.max_alloc = Some(MAX_TEXTURE_BYTES as u64);
    reader.limits(limits);
    let image = reader
        .decode()
        .map_err(|e| invalid(format!("invalid encoded image: {e}")))?;
    if image.width() != width || image.height() != height {
        return Err(invalid("encoded image dimensions differ from mip framing"));
    }
    Ok((width, height, image.into_rgba8().into_raw()))
}

fn invalid(reason: impl Into<String>) -> SceneAssetError {
    SceneAssetError::InvalidTexture {
        reason: reason.into(),
    }
}

fn unsupported(feature: impl Into<String>) -> SceneAssetError {
    SceneAssetError::Unsupported {
        feature: feature.into(),
    }
}

enum Source<'a> {
    Bytes(&'a [u8]),
    Resource(&'a SceneResource),
}

impl Source<'_> {
    fn len(&self) -> u64 {
        match self {
            Self::Bytes(bytes) => bytes.len() as u64,
            Self::Resource(resource) => resource.len(),
        }
    }
    fn read(&self, offset: usize, count: usize) -> Result<Vec<u8>> {
        match self {
            Self::Bytes(bytes) => bytes
                .get(offset..offset + count)
                .map(<[u8]>::to_vec)
                .ok_or_else(|| invalid("truncated texture field")),
            Self::Resource(resource) => resource.read_range(offset as u64, count, 32),
        }
    }
}

struct Reader<'a> {
    source: Source<'a>,
    offset: usize,
}

impl Reader<'_> {
    fn skip(&mut self, count: usize) -> Result<()> {
        let end = self
            .offset
            .checked_add(count)
            .ok_or_else(|| invalid("texture length overflow"))?;
        if end as u64 > self.source.len() {
            return Err(invalid("truncated texture field"));
        }
        self.offset = end;
        Ok(())
    }

    fn read(&mut self, count: usize) -> Result<Vec<u8>> {
        let offset = self.offset;
        self.skip(count)?;
        let bytes = self.source.read(offset, count)?;
        Ok(bytes)
    }

    fn tag(&mut self, expected: &[u8]) -> Result<()> {
        if self.read(expected.len())? != expected {
            return Err(unsupported(format!(
                "expected {}",
                String::from_utf8_lossy(expected)
            )));
        }
        Ok(())
    }

    fn u32(&mut self) -> Result<u32> {
        Ok(u32::from_le_bytes(
            self.read(4)?.try_into().expect("four bytes"),
        ))
    }

    fn f32(&mut self) -> Result<f32> {
        Ok(f32::from_bits(self.u32()?))
    }
}

//! Validated immutable imported images. File selection and persistence belong
//! to the shell; this reader accepts bytes, never paths or URLs from scripts.
use crate::{AtlasMemoryBudget, AtlasMemoryReservation, Result, SceneAssetError};
use image::{DynamicImage, ImageDecoder, ImageFormat, Limits};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    io::Cursor,
    sync::{Arc, Mutex, OnceLock, Weak},
};

const MAX_ENCODED: usize = 32 * 1024 * 1024;
const MAX_DIMENSION: u32 = 8192;
const MAX_RGBA: u64 = 128 * 1024 * 1024;

#[derive(Debug)]
struct ImageInner {
    rgba: Vec<u8>,
    width: u32,
    height: u32,
    encoded_sha256: [u8; 32],
    fingerprint: [u8; 32],
    // The Vec drops before its accounting lease.
    _lease: AtlasMemoryReservation,
}

#[derive(Debug, Clone)]
pub struct ImageSource(Arc<ImageInner>);

type ImageCache = BTreeMap<(usize, [u8; 32]), Weak<ImageInner>>;
static IMAGES: OnceLock<Mutex<ImageCache>> = OnceLock::new();

impl ImageSource {
    /// The caller accounts for its encoded buffer before reading it. This
    /// reservation additionally covers codec copies, padded component planes,
    /// conversion/orientation outputs and bounded metadata. It is a conservative
    /// working-set policy, not process RSS or allocator instrumentation.
    pub fn decode(bytes: &[u8], budget: &AtlasMemoryBudget) -> Result<Self> {
        if bytes.is_empty() || bytes.len() > MAX_ENCODED {
            return Err(invalid("imported image encoded size exceeds limit"));
        }
        let (format, width, height) = header(bytes)?;
        if width == 0
            || height == 0
            || width > MAX_DIMENSION
            || height > MAX_DIMENSION
            || u64::from(width) * u64::from(height) * 4 > MAX_RGBA
        {
            return Err(invalid("imported image dimensions exceed limit"));
        }
        let encoded_sha256: [u8; 32] = Sha256::digest(bytes).into();
        let key = (budget.domain_id(), encoded_sha256);
        let cache = IMAGES.get_or_init(|| Mutex::new(BTreeMap::new()));
        if let Some(image) = cache
            .lock()
            .map_err(|_| invalid("image cache is unavailable"))?
            .get(&key)
            .and_then(Weak::upgrade)
        {
            return Ok(Self(image));
        }
        let padded = usize::try_from(
            u64::from(width.div_ceil(16) * 16) * u64::from(height.div_ceil(16) * 16),
        )
        .map_err(|_| invalid("image size overflow"))?;
        let reserve = padded
            .checked_mul(32)
            .and_then(|n| n.checked_add(bytes.len() * 4))
            .and_then(|n| n.checked_add(1024 * 1024))
            .ok_or_else(|| invalid("image size overflow"))?;
        let mut lease = budget.reserve_bytes(reserve)?;
        let mut limits = Limits::default();
        limits.max_image_width = Some(MAX_DIMENSION);
        limits.max_image_height = Some(MAX_DIMENSION);
        limits.max_alloc = Some(reserve as u64);
        let (rgba, width, height, orientation) = match format {
            ImageFormat::Png => decode(
                image::codecs::png::PngDecoder::with_limits(Cursor::new(bytes), limits.clone())
                    .map_err(codec)?,
                limits,
                width,
                height,
            )?,
            ImageFormat::Jpeg => decode(
                image::codecs::jpeg::JpegDecoder::new(Cursor::new(bytes)).map_err(codec)?,
                limits,
                width,
                height,
            )?,
            _ => unreachable!("validated image format"),
        };
        lease.resize(rgba.capacity())?;
        let mut digest = Sha256::new();
        digest.update(b"lwe-managed-image-rgba8-top-down-v1");
        digest.update(encoded_sha256);
        digest.update(width.to_le_bytes());
        digest.update(height.to_le_bytes());
        digest.update([orientation]);
        digest.update(&rgba);
        let image = Arc::new(ImageInner {
            rgba,
            width,
            height,
            encoded_sha256,
            fingerprint: digest.finalize().into(),
            _lease: lease,
        });
        let mut entries = cache
            .lock()
            .map_err(|_| invalid("image cache is unavailable"))?;
        // A concurrent decode may have published the same immutable input.
        if let Some(existing) = entries.get(&key).and_then(Weak::upgrade) {
            return Ok(Self(existing));
        }
        entries.retain(|_, value| value.strong_count() > 0);
        if entries.len() < 256 {
            entries.insert(key, Arc::downgrade(&image));
        }
        Ok(Self(image))
    }
    pub fn width(&self) -> u32 {
        self.0.width
    }
    pub fn height(&self) -> u32 {
        self.0.height
    }
    pub fn rgba(&self) -> &[u8] {
        &self.0.rgba
    }
    pub fn capacity_bytes(&self) -> usize {
        self.0.rgba.capacity()
    }
    pub fn fingerprint(&self) -> [u8; 32] {
        self.0.fingerprint
    }
    pub fn encoded_sha256(&self) -> [u8; 32] {
        self.0.encoded_sha256
    }
    pub fn allocation_id(&self) -> usize {
        Arc::as_ptr(&self.0) as usize
    }
}

fn decode<D: ImageDecoder>(
    mut decoder: D,
    limits: Limits,
    width: u32,
    height: u32,
) -> Result<(Vec<u8>, u32, u32, u8)> {
    decoder.set_limits(limits).map_err(codec)?;
    if decoder.dimensions() != (width, height) {
        return Err(invalid("image header dimensions changed"));
    }
    if !matches!(
        decoder.color_type(),
        image::ColorType::L8
            | image::ColorType::La8
            | image::ColorType::Rgb8
            | image::ColorType::Rgba8
    ) {
        return Err(invalid("imported image requires 8-bit RGB or grayscale"));
    }
    let orientation = decoder.orientation().map_err(codec)?;
    let mut image = DynamicImage::from_decoder(decoder).map_err(codec)?;
    image.apply_orientation(orientation);
    let width = image.width();
    let height = image.height();
    Ok((
        image.into_rgba8().into_raw(),
        width,
        height,
        orientation as u8,
    ))
}

/// Inspect only the bounded TIFF declarations needed by the RGBA8/sRGB path.
/// Values stay borrowed and the directory queue uses fixed stack storage.
fn validate_exif(bytes: &[u8]) -> Result<()> {
    const MAX_DIRECTORIES: usize = 16;
    const MAX_ENTRIES: usize = 1024;
    if bytes.len() < 8 || bytes.len() > 65536 {
        return Err(invalid("invalid or oversized image EXIF"));
    }
    let little = match &bytes[..2] {
        b"II" => true,
        b"MM" => false,
        _ => return Err(invalid("invalid EXIF byte order")),
    };
    let short = |offset: usize| -> Result<u16> {
        let value = bytes
            .get(offset..offset.saturating_add(2))
            .and_then(|value| value.try_into().ok())
            .ok_or_else(|| invalid("truncated EXIF value"))?;
        Ok(if little {
            u16::from_le_bytes(value)
        } else {
            u16::from_be_bytes(value)
        })
    };
    let long = |offset: usize| -> Result<u32> {
        let value = bytes
            .get(offset..offset.saturating_add(4))
            .and_then(|value| value.try_into().ok())
            .ok_or_else(|| invalid("truncated EXIF value"))?;
        Ok(if little {
            u32::from_le_bytes(value)
        } else {
            u32::from_be_bytes(value)
        })
    };
    if short(2)? != 42 {
        return Err(invalid("unsupported EXIF TIFF header"));
    }
    let mut directories = [(0u32, false); MAX_DIRECTORIES];
    let mut length = 0;
    let enqueue = |directories: &mut [(u32, bool); MAX_DIRECTORIES],
                   length: &mut usize,
                   offset: u32,
                   interoperability: bool|
     -> Result<()> {
        if offset < 8 || offset as usize >= bytes.len() {
            return Err(invalid("EXIF directory offset is outside the source"));
        }
        if directories[..*length].iter().any(|item| item.0 == offset) {
            return Err(invalid("EXIF directory cycle or alias"));
        }
        if *length == MAX_DIRECTORIES {
            return Err(invalid("EXIF directory count exceeds limit"));
        }
        directories[*length] = (offset, interoperability);
        *length += 1;
        Ok(())
    };
    enqueue(&mut directories, &mut length, long(4)?, false)?;
    let mut index = 0;
    let mut entries = 0usize;
    while index < length {
        let (directory, interoperability) = directories[index];
        index += 1;
        let offset = directory as usize;
        let count = usize::from(short(offset)?);
        entries = entries
            .checked_add(count)
            .filter(|total| *total <= MAX_ENTRIES && count <= 256)
            .ok_or_else(|| invalid("EXIF entry count exceeds limit"))?;
        let end = offset
            .checked_add(2 + count * 12 + 4)
            .filter(|end| *end <= bytes.len())
            .ok_or_else(|| invalid("truncated EXIF directory"))?;
        for entry in 0..count {
            let entry = offset + 2 + entry * 12;
            let tag = short(entry)?;
            let kind = short(entry + 2)?;
            let count = long(entry + 4)? as usize;
            let unit = match kind {
                1 | 2 | 6 | 7 => 1,
                3 | 8 => 2,
                4 | 9 | 11 => 4,
                13 if tag == 0x014a => 4,
                5 | 10 | 12 => 8,
                _ => return Err(invalid("unsupported EXIF value type")),
            };
            let size = count
                .checked_mul(unit)
                .ok_or_else(|| invalid("EXIF value size overflow"))?;
            let value = if size <= 4 {
                entry + 8
            } else {
                let value = long(entry + 8)? as usize;
                if value < 8 {
                    return Err(invalid("EXIF value offset is outside the source"));
                }
                value
            };
            let payload = bytes
                .get(value..value.saturating_add(size))
                .ok_or_else(|| invalid("EXIF value offset is outside the source"))?;
            match tag {
                0x8773 => return Err(invalid("EXIF ICC profile requires color conversion")),
                0xa001 => {
                    if kind != 3 || count != 1 || short(value)? != 1 {
                        return Err(invalid("EXIF color space requires color conversion"));
                    }
                }
                0x0001 if interoperability => {
                    if kind != 2 || payload != b"R98\0" {
                        return Err(invalid("EXIF interoperability requires color conversion"));
                    }
                }
                0x0112 => {
                    if kind != 3 || count != 1 || !(1..=8).contains(&short(value)?) {
                        return Err(invalid("invalid EXIF orientation"));
                    }
                }
                0x8769 | 0x8825 | 0xa005 => {
                    if kind != 4 || count != 1 {
                        return Err(invalid("invalid EXIF directory pointer"));
                    }
                    enqueue(&mut directories, &mut length, long(value)?, tag == 0xa005)?;
                }
                0x014a => {
                    if !matches!(kind, 4 | 13) || count == 0 || count > MAX_DIRECTORIES {
                        return Err(invalid("invalid EXIF subdirectory pointers"));
                    }
                    for pointer in payload.as_chunks::<4>().0 {
                        let pointer = if little {
                            u32::from_le_bytes(*pointer)
                        } else {
                            u32::from_be_bytes(*pointer)
                        };
                        enqueue(&mut directories, &mut length, pointer, false)?;
                    }
                }
                _ => {}
            }
        }
        let next = long(end - 4)?;
        if next != 0 {
            enqueue(&mut directories, &mut length, next, interoperability)?;
        }
    }
    Ok(())
}

fn header(bytes: &[u8]) -> Result<(ImageFormat, u32, u32)> {
    if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        if bytes.len() < 33 || &bytes[12..16] != b"IHDR" || bytes[24] != 8 {
            return Err(invalid("invalid or non-8-bit PNG"));
        }
        let width = u32::from_be_bytes(bytes[16..20].try_into().unwrap());
        let height = u32::from_be_bytes(bytes[20..24].try_into().unwrap());
        let mut offset = 8usize;
        let mut chunks = 0;
        let mut seen_exif = false;
        let mut seen_data = false;
        while offset < bytes.len() {
            chunks += 1;
            if chunks > 4096 {
                return Err(invalid("PNG chunk count exceeds limit"));
            }
            let prefix = bytes
                .get(offset..offset + 8)
                .ok_or_else(|| invalid("truncated PNG chunk"))?;
            let length = u32::from_be_bytes(prefix[..4].try_into().unwrap()) as usize;
            let end = offset
                .checked_add(length)
                .and_then(|n| n.checked_add(12))
                .filter(|end| *end <= bytes.len())
                .ok_or_else(|| invalid("truncated PNG payload"))?;
            match &prefix[4..8] {
                b"acTL" | b"fcTL" | b"fdAT" => {
                    return Err(invalid("animated PNG images are unsupported"));
                }
                b"iCCP" | b"cICP" => {
                    return Err(invalid(
                        "profiled PNG images require a color conversion path",
                    ));
                }
                b"mDCv" | b"cLLi" => {
                    return Err(invalid("HDR PNG metadata requires a separate color path"));
                }
                b"cHRM" => {
                    let primaries = [31270u32, 32900, 64000, 33000, 30000, 60000, 15000, 6000];
                    if length != 32
                        || !bytes[offset + 8..offset + 8 + length]
                            .as_chunks::<4>()
                            .0
                            .iter()
                            .zip(primaries)
                            .all(|(actual, expected)| actual == &expected.to_be_bytes())
                    {
                        return Err(invalid("PNG chromaticities require color conversion"));
                    }
                }
                b"gAMA"
                    if length != 4
                        || bytes.get(offset + 8..offset + 12) != Some(&45455u32.to_be_bytes()) =>
                {
                    return Err(invalid("PNG gamma requires color conversion"));
                }
                b"eXIf" => {
                    if seen_exif || seen_data {
                        return Err(invalid("duplicate or late PNG EXIF is unsupported"));
                    }
                    validate_exif(&bytes[offset + 8..offset + 8 + length])?;
                    seen_exif = true;
                }
                b"IDAT" => seen_data = true,
                b"IEND" => {
                    if length != 0 || end != bytes.len() {
                        return Err(invalid("PNG trailing payload"));
                    }
                    return Ok((ImageFormat::Png, width, height));
                }
                _ => {}
            }
            offset = end;
        }
        return Err(invalid("PNG has no complete end chunk"));
    }
    if bytes.starts_with(&[0xff, 0xd8]) {
        let mut offset = 2usize;
        let mut size = None;
        let mut markers = 0;
        let mut in_scan = false;
        let mut saw_scan = false;
        let mut seen_exif = false;
        while offset < bytes.len() {
            if in_scan {
                while offset < bytes.len() && bytes[offset] != 0xff {
                    offset += 1;
                }
                if offset == bytes.len() {
                    break;
                }
            }
            if bytes[offset] != 0xff {
                return Err(invalid("invalid JPEG marker"));
            }
            while bytes.get(offset) == Some(&0xff) {
                offset += 1;
            }
            let marker = *bytes
                .get(offset)
                .ok_or_else(|| invalid("truncated JPEG marker"))?;
            offset += 1;
            if in_scan && (marker == 0 || matches!(marker, 0xd0..=0xd7)) {
                continue;
            }
            markers += 1;
            if markers > 4096 {
                return Err(invalid("JPEG metadata marker count exceeds limit"));
            }
            in_scan = false;
            if marker == 0xd9 {
                if !saw_scan || offset != bytes.len() {
                    return Err(invalid("JPEG has no complete single image"));
                }
                let (width, height) = size.ok_or_else(|| invalid("unsupported JPEG frame"))?;
                return Ok((ImageFormat::Jpeg, width, height));
            }
            if marker == 0xd8 || marker == 0 {
                return Err(invalid("invalid or multi-image JPEG"));
            }
            if marker == 0x01 {
                continue;
            }
            let length_bytes = bytes
                .get(offset..offset + 2)
                .ok_or_else(|| invalid("truncated JPEG segment"))?;
            let length = u16::from_be_bytes(length_bytes.try_into().unwrap()) as usize;
            if length < 2 {
                return Err(invalid("invalid JPEG segment length"));
            }
            let data = bytes
                .get(
                    offset + 2
                        ..offset
                            .checked_add(length)
                            .ok_or_else(|| invalid("JPEG size overflow"))?,
                )
                .ok_or_else(|| invalid("truncated JPEG segment"))?;
            if marker == 0xe1 && data.starts_with(b"Exif\0\0") {
                if seen_exif || saw_scan {
                    return Err(invalid("duplicate or late JPEG EXIF is unsupported"));
                }
                validate_exif(&data[6..])?;
                seen_exif = true;
            }
            if (marker == 0xe2
                && [
                    b"ICC_PROFILE\0".as_slice(),
                    b"MPF\0",
                    b"urn:iso:std:iso:ts:21496:-1\0",
                ]
                .iter()
                .any(|prefix| data.starts_with(prefix)))
                || (marker == 0xe1 && data.starts_with(b"http://ns.adobe.com/"))
            {
                return Err(invalid(
                    "profiled, multi-image or HDR JPEG metadata requires a separate decoding path",
                ));
            }
            if matches!(marker, 0xc0..=0xc2) {
                if data.len() < 6 || data[0] != 8 || !matches!(data[5], 1 | 3) {
                    return Err(invalid("JPEG requires 8-bit RGB or grayscale components"));
                }
                size = Some((
                    u16::from_be_bytes([data[3], data[4]]) as u32,
                    u16::from_be_bytes([data[1], data[2]]) as u32,
                ));
            }
            offset += length;
            if marker == 0xda {
                if size.is_none() {
                    return Err(invalid("unsupported JPEG frame"));
                }
                in_scan = true;
                saw_scan = true;
            }
        }
        return Err(invalid("JPEG has no complete end marker"));
    }
    Err(invalid("only static PNG and JPEG images are supported"))
}
fn invalid(reason: impl Into<String>) -> SceneAssetError {
    SceneAssetError::InvalidTexture {
        reason: reason.into(),
    }
}
fn codec(error: image::ImageError) -> SceneAssetError {
    invalid(format!("invalid imported image: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn png() -> Vec<u8> {
        let mut bytes = Cursor::new(Vec::new());
        image::RgbaImage::from_pixel(2, 3, image::Rgba([1, 2, 3, 255]))
            .write_to(&mut bytes, ImageFormat::Png)
            .unwrap();
        bytes.into_inner()
    }
    fn exif_fixture(
        little: bool,
        color_space: Option<u16>,
        interoperability: Option<&[u8; 4]>,
        icc: bool,
    ) -> Vec<u8> {
        let count = usize::from(color_space.is_some())
            + usize::from(interoperability.is_some())
            + usize::from(icc);
        let exif = 26usize;
        let interop = exif + 2 + count * 12 + 4;
        let mut bytes = vec![0; interop + if interoperability.is_some() { 18 } else { 0 }];
        bytes[..2].copy_from_slice(if little { b"II" } else { b"MM" });
        let short = |value: u16| {
            if little {
                value.to_le_bytes()
            } else {
                value.to_be_bytes()
            }
        };
        let long = |value: u32| {
            if little {
                value.to_le_bytes()
            } else {
                value.to_be_bytes()
            }
        };
        bytes[2..4].copy_from_slice(&short(42));
        bytes[4..8].copy_from_slice(&long(8));
        bytes[8..10].copy_from_slice(&short(1));
        let write_entry = |bytes: &mut [u8], offset, tag, kind, count, value: [u8; 4]| {
            bytes[offset..offset + 2].copy_from_slice(&short(tag));
            bytes[offset + 2..offset + 4].copy_from_slice(&short(kind));
            bytes[offset + 4..offset + 8].copy_from_slice(&long(count));
            bytes[offset + 8..offset + 12].copy_from_slice(&value);
        };
        write_entry(&mut bytes, 10, 0x8769, 4, 1, long(exif as u32));
        bytes[exif..exif + 2].copy_from_slice(&short(count as u16));
        let mut entry = exif + 2;
        if let Some(value) = color_space {
            let value = short(value);
            write_entry(&mut bytes, entry, 0xa001, 3, 1, [value[0], value[1], 0, 0]);
            entry += 12;
        }
        if let Some(value) = interoperability {
            write_entry(&mut bytes, entry, 0xa005, 4, 1, long(interop as u32));
            entry += 12;
            bytes[interop..interop + 2].copy_from_slice(&short(1));
            write_entry(&mut bytes, interop + 2, 1, 2, 4, *value);
        }
        if icc {
            write_entry(&mut bytes, entry, 0x8773, 7, 1, [1, 0, 0, 0]);
        }
        bytes
    }
    fn jpeg_exif(tiff: &[u8]) -> Vec<u8> {
        let mut encoded = Cursor::new(Vec::new());
        image::RgbImage::from_pixel(3, 2, image::Rgb([1, 2, 3]))
            .write_to(&mut encoded, ImageFormat::Jpeg)
            .unwrap();
        let mut app1 = vec![255, 225];
        app1.extend_from_slice(&((tiff.len() + 8) as u16).to_be_bytes());
        app1.extend_from_slice(b"Exif\0\0");
        app1.extend_from_slice(tiff);
        let mut bytes = encoded.into_inner();
        bytes.splice(2..2, app1);
        bytes
    }
    fn png_exif(tiff: &[u8]) -> Vec<u8> {
        let mut chunk = Vec::new();
        chunk.extend_from_slice(&(tiff.len() as u32).to_be_bytes());
        chunk.extend_from_slice(b"eXIf");
        chunk.extend_from_slice(tiff);
        let mut crc = u32::MAX;
        for byte in &chunk[4..] {
            crc ^= u32::from(*byte);
            for _ in 0..8 {
                crc = (crc >> 1) ^ (0xedb88320 & 0u32.wrapping_sub(crc & 1));
            }
        }
        chunk.extend_from_slice(&(!crc).to_be_bytes());
        let mut bytes = png();
        bytes.splice(33..33, chunk);
        bytes
    }
    #[test]
    fn exif_srgb_or_absent_color_declarations_allow_both_byte_orders() {
        for little in [true, false] {
            for (color, interoperability) in [(None, None), (Some(1), Some(b"R98\0"))] {
                let tiff = exif_fixture(little, color, interoperability, false);
                validate_exif(&tiff).unwrap();
                let budget = AtlasMemoryBudget::new(8 * 1024 * 1024).unwrap();
                for bytes in [jpeg_exif(&tiff), png_exif(&tiff)] {
                    let image = ImageSource::decode(&bytes, &budget).unwrap();
                    assert!(image.width() > 0);
                    drop(image);
                    assert_eq!(budget.usage().charged_bytes, 0);
                }
            }
        }
    }
    #[test]
    fn nested_exif_non_srgb_and_icc_are_rejected_before_decode_reservation() {
        for little in [true, false] {
            for tiff in [
                exif_fixture(little, Some(2), None, false),
                exif_fixture(little, Some(65535), None, false),
                exif_fixture(little, Some(1), Some(b"R03\0"), false),
                exif_fixture(little, None, None, true),
            ] {
                let budget = AtlasMemoryBudget::new(8 * 1024 * 1024).unwrap();
                for bytes in [jpeg_exif(&tiff), png_exif(&tiff)] {
                    let error = ImageSource::decode(&bytes, &budget).unwrap_err();
                    assert!(error.to_string().contains("color conversion"));
                    assert_eq!(budget.usage().charged_bytes, 0);
                }
            }
        }
    }
    #[test]
    fn malformed_exif_offsets_cycles_and_counts_fail_closed_without_reservation() {
        for little in [true, false] {
            let long = |value: u32| {
                if little {
                    value.to_le_bytes()
                } else {
                    value.to_be_bytes()
                }
            };
            let short = |value: u16| {
                if little {
                    value.to_le_bytes()
                } else {
                    value.to_be_bytes()
                }
            };
            let mut invalid_sources = Vec::new();
            let mut root = exif_fixture(little, None, None, false);
            root[4..8].copy_from_slice(&long(u32::MAX));
            invalid_sources.push(root);
            let mut cycle = exif_fixture(little, None, None, false);
            cycle[18..22].copy_from_slice(&long(8));
            invalid_sources.push(cycle);
            let mut count = exif_fixture(little, None, None, false);
            count[8..10].copy_from_slice(&short(257));
            invalid_sources.push(count);
            let mut payload = exif_fixture(little, Some(1), None, false);
            payload[28..30].copy_from_slice(&short(0x9286));
            payload[30..32].copy_from_slice(&short(7));
            payload[32..36].copy_from_slice(&long(5));
            payload[36..40].copy_from_slice(&long(u32::MAX));
            invalid_sources.push(payload);
            let mut unknown_directory = exif_fixture(little, Some(1), None, false);
            unknown_directory[28..30].copy_from_slice(&short(0x7e00));
            unknown_directory[30..32].copy_from_slice(&short(13));
            unknown_directory[36..40].copy_from_slice(&long(8));
            invalid_sources.push(unknown_directory);
            let mut nested = exif_fixture(little, None, Some(b"R98\0"), false);
            nested[36..40].copy_from_slice(&long(8));
            invalid_sources.push(nested);
            let mut chain = vec![0; 8 + 17 * 6];
            chain[..2].copy_from_slice(if little { b"II" } else { b"MM" });
            chain[2..4].copy_from_slice(&short(42));
            chain[4..8].copy_from_slice(&long(8));
            for index in 0..16 {
                let next = 8 + (index + 1) * 6;
                chain[8 + index * 6 + 2..8 + index * 6 + 6].copy_from_slice(&long(next as u32));
            }
            invalid_sources.push(chain);
            let directory_size = 2 + 256 * 12 + 4;
            let mut many_entries = vec![0; 8 + 5 * directory_size];
            many_entries[..2].copy_from_slice(if little { b"II" } else { b"MM" });
            many_entries[2..4].copy_from_slice(&short(42));
            many_entries[4..8].copy_from_slice(&long(8));
            for directory in 0..5 {
                let offset = 8 + directory * directory_size;
                many_entries[offset..offset + 2].copy_from_slice(&short(256));
                for entry in 0..256 {
                    let entry = offset + 2 + entry * 12;
                    many_entries[entry + 2..entry + 4].copy_from_slice(&short(7));
                }
                if directory < 4 {
                    let next = offset + directory_size;
                    many_entries[next - 4..next].copy_from_slice(&long(next as u32));
                }
            }
            invalid_sources.push(many_entries);
            let mut subdirectory_cycle = exif_fixture(little, None, None, false);
            subdirectory_cycle[10..12].copy_from_slice(&short(0x014a));
            subdirectory_cycle[18..22].copy_from_slice(&long(8));
            invalid_sources.push(subdirectory_cycle);
            let budget = AtlasMemoryBudget::new(8 * 1024 * 1024).unwrap();
            for tiff in invalid_sources {
                for bytes in [jpeg_exif(&tiff), png_exif(&tiff)] {
                    assert!(ImageSource::decode(&bytes, &budget).is_err());
                    assert_eq!(budget.usage().charged_bytes, 0);
                }
            }
        }
    }
    #[test]
    fn late_or_duplicate_exif_cannot_silently_skip_author_orientation() {
        let tiff = exif_fixture(true, Some(1), Some(b"R98\0"), false);
        let jpeg = jpeg_exif(&tiff);
        let segment = &jpeg[2..2 + tiff.len() + 10];
        let mut duplicate_jpeg = jpeg.clone();
        duplicate_jpeg.splice(2..2, segment.iter().copied());
        let mut late_jpeg = jpeg.clone();
        let segment: Vec<_> = late_jpeg.drain(2..2 + tiff.len() + 10).collect();
        let end = late_jpeg.len() - 2;
        late_jpeg.splice(end..end, segment);
        let png = png_exif(&tiff);
        let chunk = &png[33..33 + tiff.len() + 12];
        let mut duplicate_png = png.clone();
        duplicate_png.splice(33..33, chunk.iter().copied());
        let mut late_png = png;
        let chunk: Vec<_> = late_png.drain(33..33 + tiff.len() + 12).collect();
        let end = late_png.len() - 12;
        late_png.splice(end..end, chunk);
        let budget = AtlasMemoryBudget::new(8 * 1024 * 1024).unwrap();
        for bytes in [duplicate_jpeg, late_jpeg, duplicate_png, late_png] {
            let error = ImageSource::decode(&bytes, &budget).unwrap_err();
            assert!(error.to_string().contains("duplicate or late"));
            assert_eq!(budget.usage().charged_bytes, 0);
        }
    }
    #[test]
    fn retained_image_clones_and_failures_keep_budget_ownership() {
        let budget = AtlasMemoryBudget::new(8 * 1024 * 1024).unwrap();
        let bytes = png();
        let source = ImageSource::decode(&bytes, &budget).unwrap();
        assert_eq!((source.width(), source.height()), (2, 3));
        let used = budget.usage().charged_bytes;
        assert_eq!(used, source.capacity_bytes());
        let repeated = ImageSource::decode(&bytes, &budget).unwrap();
        assert_eq!(repeated.allocation_id(), source.allocation_id());
        assert_eq!(budget.usage().charged_bytes, used);
        drop(repeated);
        let independent = AtlasMemoryBudget::new(8 * 1024 * 1024).unwrap();
        let other = ImageSource::decode(&bytes, &independent).unwrap();
        assert_ne!(other.allocation_id(), source.allocation_id());
        drop(other);
        assert_eq!(independent.usage().charged_bytes, 0);
        let clone = source.clone();
        drop(source);
        assert_eq!(budget.usage().charged_bytes, used);
        assert!(ImageSource::decode(b"bad", &budget).is_err());
        assert_eq!(budget.usage().charged_bytes, used);
        drop(clone);
        assert_eq!(budget.usage().charged_bytes, 0);
    }
    #[test]
    fn headers_reject_bombs_animation_profiles_and_truncation_before_decode() {
        let bytes = png();
        let budget = AtlasMemoryBudget::new(8 * 1024 * 1024).unwrap();
        let mut bomb = bytes.clone();
        bomb[16..20].copy_from_slice(&u32::MAX.to_be_bytes());
        assert!(ImageSource::decode(&bomb, &budget).is_err());
        assert_eq!(budget.usage().charged_bytes, 0);
        assert!(ImageSource::decode(&bytes[..bytes.len() - 1], &budget).is_err());
        for tag in [b"acTL", b"iCCP", b"cICP"] {
            let mut bad = bytes.clone();
            let offset = bad.len() - 12;
            bad.splice(
                offset..offset,
                [0, 0, 0, 0, tag[0], tag[1], tag[2], tag[3], 0, 0, 0, 0],
            );
            assert!(ImageSource::decode(&bad, &budget).is_err());
            assert_eq!(budget.usage().charged_bytes, 0);
        }
        let small = AtlasMemoryBudget::new(100).unwrap();
        assert!(ImageSource::decode(&bytes, &small).is_err());
        assert_eq!(small.usage().charged_bytes, 0);
    }
    #[test]
    fn jpeg_rgb_and_corrupt_segment_lengths_are_bounded() {
        let mut encoded = Cursor::new(Vec::new());
        image::RgbImage::from_pixel(3, 2, image::Rgb([255, 0, 0]))
            .write_to(&mut encoded, ImageFormat::Jpeg)
            .unwrap();
        let budget = AtlasMemoryBudget::new(8 * 1024 * 1024).unwrap();
        let source = ImageSource::decode(encoded.get_ref(), &budget).unwrap();
        assert_eq!(source.rgba().len(), 24);
        assert_eq!((source.width(), source.height()), (3, 2));
        assert!(
            source
                .rgba()
                .as_chunks::<4>()
                .0
                .iter()
                .all(|p| p[0] > 240 && p[1] < 10 && p[3] == 255)
        );
        assert!(ImageSource::decode(&[255, 216, 255, 224, 0, 1], &budget).is_err());
    }

    #[test]
    fn jpeg_rejects_profile_multi_image_and_gainmap_segments_after_scan_data() {
        let mut encoded = Cursor::new(Vec::new());
        image::RgbImage::from_pixel(8, 8, image::Rgb([1, 2, 3]))
            .write_to(&mut encoded, ImageFormat::Jpeg)
            .unwrap();
        let budget = AtlasMemoryBudget::new(8 * 1024 * 1024).unwrap();
        for prefix in [
            b"ICC_PROFILE\0".as_slice(),
            b"MPF\0",
            b"urn:iso:std:iso:ts:21496:-1\0",
        ] {
            let mut bytes = encoded.get_ref().clone();
            let mut marker = vec![0xff, 0xe2];
            marker.extend_from_slice(&((prefix.len() + 2) as u16).to_be_bytes());
            marker.extend_from_slice(prefix);
            let end = bytes.len() - 2;
            bytes.splice(end..end, marker);
            let error = ImageSource::decode(&bytes, &budget).unwrap_err();
            assert!(
                error
                    .to_string()
                    .contains("profiled, multi-image or HDR JPEG metadata")
            );
            assert_eq!(budget.usage().charged_bytes, 0);
        }
        let mut multiple = encoded.get_ref().clone();
        multiple.extend_from_slice(encoded.get_ref());
        assert!(ImageSource::decode(&multiple, &budget).is_err());
    }

    #[test]
    fn entropy_byte_stuffing_does_not_consume_the_metadata_segment_budget() {
        let mut encoded = Cursor::new(Vec::new());
        image::RgbImage::from_pixel(8, 8, image::Rgb([1, 2, 3]))
            .write_to(&mut encoded, ImageFormat::Jpeg)
            .unwrap();
        let mut bytes = encoded.into_inner();
        let end = bytes.len() - 2;
        bytes.splice(end..end, [255, 0].repeat(4097));
        assert_eq!(header(&bytes).unwrap(), (ImageFormat::Jpeg, 8, 8));
        // Header validation is not a claim that altered entropy pixels decode.
    }

    #[test]
    fn jpeg_exif_orientation_is_applied_before_publishing_dimensions_and_pixels() {
        let mut encoded = Cursor::new(Vec::new());
        image::RgbImage::from_fn(3, 2, |x, y| {
            image::Rgb([(x * 60) as u8, (y * 120) as u8, 80])
        })
        .write_to(&mut encoded, ImageFormat::Jpeg)
        .unwrap();
        let budget = AtlasMemoryBudget::new(8 * 1024 * 1024).unwrap();
        let original = ImageSource::decode(encoded.get_ref(), &budget).unwrap();
        for orientation in 2u8..=8 {
            // Self-authored little-endian TIFF with a single inline orientation tag.
            let mut payload = b"Exif\0\0II\x2a\0\x08\0\0\0\x01\0\x12\x01\x03\0\x01\0\0\0".to_vec();
            payload.extend_from_slice(&[orientation, 0, 0, 0, 0, 0, 0, 0]);
            let mut bytes = encoded.get_ref().clone();
            let mut app1 = vec![255, 225];
            app1.extend_from_slice(&((payload.len() + 2) as u16).to_be_bytes());
            app1.extend_from_slice(&payload);
            bytes.splice(2..2, app1);
            let oriented = ImageSource::decode(&bytes, &budget).unwrap();
            assert_eq!(
                (oriented.width(), oriented.height()),
                if orientation >= 5 { (2, 3) } else { (3, 2) }
            );
            for y in 0..2 {
                for x in 0..3 {
                    let (dx, dy) = match orientation {
                        2 => (2 - x, y),
                        3 => (2 - x, 1 - y),
                        4 => (x, 1 - y),
                        5 => (y, x),
                        6 => (1 - y, x),
                        7 => (1 - y, 2 - x),
                        8 => (y, 2 - x),
                        _ => unreachable!(),
                    };
                    let src = (y * 3 + x) * 4;
                    let dst = (dy * oriented.width() as usize + dx) * 4;
                    assert_eq!(
                        &oriented.rgba()[dst..dst + 4],
                        &original.rgba()[src..src + 4],
                        "orientation {orientation}"
                    );
                }
            }
            assert_ne!(oriented.fingerprint(), original.fingerprint());
        }
    }
}

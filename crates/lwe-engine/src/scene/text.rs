//! Bounded CPU text surfaces from descriptor-resolved font bytes. Layout uses
//! authored extents and font metrics; this is not Windows text-renderer parity.
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, Weak};

use anyhow::{Context, Result, ensure};
use fontdue::{Font, FontSettings};
use lwe_scene::{AtlasMemoryReservation, ResourceOrigin, SceneResource};
use sha2::{Digest, Sha256};

const MAX_FONT_BYTES: usize = 4 * 1024 * 1024;
const MAX_GLYPHS: u16 = 4096;
const MAX_MAPPING_VISITS: usize = 8192;
const MAX_OUTLINE_SEGMENTS: usize = 65_536;
/// Fontdue owns private geometry/maps without an allocation-capacity accessor.
/// This conservative logical reservation is separate from encoded/font bitmap
/// capacity, not a measurement or guarantee of allocator/RSS consumption.
const FONT_WORK_RESERVATION: usize = 64 * 1024 * 1024;
const MAX_GLYPH_PIXELS: usize = 1024 * 1024;

#[derive(Debug, Clone, Copy)]
enum Alignment {
    Start,
    Center,
    End,
}
fn align(value: &str, vertical: bool) -> Result<Alignment> {
    match value {
        "center" => Ok(Alignment::Center),
        "left" if !vertical => Ok(Alignment::Start),
        "right" if !vertical => Ok(Alignment::End),
        "top" if vertical => Ok(Alignment::Start),
        "bottom" if vertical => Ok(Alignment::End),
        _ => anyhow::bail!("Unsupported authored text alignment"),
    }
}
fn aligned(alignment: Alignment, available: f32, used: f32) -> f32 {
    match alignment {
        Alignment::Start => 0.0,
        Alignment::Center => (available - used) * 0.5,
        Alignment::End => available - used,
    }
}

struct EncodedFont {
    // Preserve buffer-before-lease destruction even on a preflight/parser error.
    bytes: Vec<u8>,
    reservation: AtlasMemoryReservation,
}

struct FontOwner {
    font: Font,
    _encoded: Vec<u8>,
    glyph_count: u16,
    outline_segments: usize,
    mapping_visits: usize,
    // Drop font and encoded source before releasing their reservation.
    _reservation: AtlasMemoryReservation,
}
impl std::fmt::Debug for FontOwner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FontOwner")
            .field("glyph_count", &self.glyph_count)
            .field("outline_segments", &self.outline_segments)
            .field("mapping_visits", &self.mapping_visits)
            .finish_non_exhaustive()
    }
}
static FONTS: Mutex<BTreeMap<[u8; 32], Weak<FontOwner>>> = Mutex::new(BTreeMap::new());

#[derive(Debug, Clone)]
pub(crate) struct PreparedText {
    font: Arc<FontOwner>,
    pub width: u32,
    pub height: u32,
    pub point_size: f32,
    pub padding: u32,
    pub font_sha256: [u8; 32],
    pub font_origin: ResourceOrigin,
    horizontal: Alignment,
    vertical: Alignment,
}

#[derive(Debug)]
struct FrameInner {
    width: u32,
    height: u32,
    fingerprint: [u8; 32],
    rgba: Vec<u8>,
    _reservation: AtlasMemoryReservation,
}
#[derive(Debug, Clone)]
pub(crate) struct TextFrame(Arc<FrameInner>);
impl TextFrame {
    pub fn allocation_id(&self) -> usize {
        Arc::as_ptr(&self.0) as usize
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
    pub fn fingerprint(&self) -> [u8; 32] {
        self.0.fingerprint
    }
    pub fn capacity_bytes(&self) -> usize {
        self.0.rgba.capacity()
    }
}

#[derive(Default)]
struct OutlineBound {
    segments: usize,
    invalid: bool,
    minimum: [f32; 2],
    maximum: [f32; 2],
    seen: bool,
}
impl OutlineBound {
    fn point(&mut self, x: f32, y: f32) {
        self.segments = self.segments.saturating_add(1);
        if !x.is_finite() || !y.is_finite() {
            self.invalid = true;
            return;
        }
        if !self.seen {
            self.minimum = [x, y];
            self.maximum = [x, y];
            self.seen = true;
        }
        for (i, v) in [x, y].into_iter().enumerate() {
            self.minimum[i] = self.minimum[i].min(v);
            self.maximum[i] = self.maximum[i].max(v);
        }
    }
}
impl ttf_parser::OutlineBuilder for OutlineBound {
    fn move_to(&mut self, x: f32, y: f32) {
        self.point(x, y);
    }
    fn line_to(&mut self, x: f32, y: f32) {
        self.point(x, y);
    }
    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.point(x1, y1);
        self.point(x, y);
    }
    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.point(x1, y1);
        self.point(x2, y2);
        self.point(x, y);
    }
    fn close(&mut self) {}
}
fn be16(data: &[u8], offset: usize) -> Result<u16> {
    Ok(u16::from_be_bytes(
        data.get(offset..offset + 2)
            .context("Truncated font table")?
            .try_into()?,
    ))
}
fn be32(data: &[u8], offset: usize) -> Result<u32> {
    Ok(u32::from_be_bytes(
        data.get(offset..offset + 4)
            .context("Truncated font table")?
            .try_into()?,
    ))
}

fn preflight(bytes: &[u8]) -> Result<(u16, usize, usize)> {
    ensure!(
        !bytes.is_empty() && bytes.len() <= MAX_FONT_BYTES,
        "Font source exceeds bounded byte size"
    );
    ensure!(
        bytes.starts_with(b"OTTO") || bytes.starts_with(&[0, 1, 0, 0]),
        "Only single TrueType/OpenType fonts are supported"
    );
    let table_count = be16(bytes, 4)? as usize;
    ensure!(
        table_count > 0 && table_count <= 64,
        "Font table count exceeds budget"
    );
    for index in 0..table_count {
        let header = 12 + index * 16;
        let offset = be32(bytes, header + 8)? as usize;
        let length = be32(bytes, header + 12)? as usize;
        ensure!(
            offset
                .checked_add(length)
                .is_some_and(|end| end <= bytes.len()),
            "Font table is outside sealed source"
        );
    }
    let face =
        ttf_parser::Face::parse(bytes, 0).map_err(|_| anyhow::anyhow!("Invalid OpenType font"))?;
    let glyphs = face.number_of_glyphs();
    ensure!(
        glyphs > 0 && glyphs <= MAX_GLYPHS,
        "Font glyph count exceeds budget"
    );
    let cmap = face
        .raw_face()
        .table(ttf_parser::Tag::from_bytes(b"cmap"))
        .context("Font requires bounded character mappings")?;
    let count = be16(cmap, 2)? as usize;
    ensure!(
        count > 0 && count <= 8,
        "Font cmap table count exceeds budget"
    );
    // Format12/13 ranges can describe billions of codepoints in a tiny file.
    // Bound those ranges before asking ttf-parser/fontdue to enumerate them.
    for index in 0..count {
        let offset = be32(cmap, 4 + index * 8 + 4)? as usize;
        let sub = cmap.get(offset..).context("Font cmap outside table")?;
        let format = be16(sub, 0)?;
        ensure!(
            matches!(format, 0 | 4 | 6 | 12 | 13),
            "Unmodeled cmap format"
        );
        if matches!(format, 12 | 13) {
            let groups = be32(sub, 12)? as usize;
            ensure!(groups <= 4096, "Font cmap range count exceeds budget");
            let mut covered = 0usize;
            for group in 0..groups {
                let start = be32(sub, 16 + group * 12)?;
                let end = be32(sub, 20 + group * 12)?;
                ensure!(
                    start <= end && end <= 0x10ffff && !(start <= 0xdfff && end >= 0xd800),
                    "Invalid Unicode font range"
                );
                covered = covered
                    .checked_add((end - start + 1) as usize)
                    .context("Font cmap count overflow")?;
                ensure!(
                    covered <= MAX_MAPPING_VISITS,
                    "Font cmap range exceeds budget"
                );
            }
        }
    }
    let mut visits = 0usize;
    let mut invalid = false;
    for table in face
        .tables()
        .cmap
        .context("Font cmap unavailable")?
        .subtables
    {
        table.codepoints(|codepoint| {
            if table
                .glyph_index(codepoint)
                .is_some_and(|glyph| glyph.0 != 0)
            {
                visits = visits.saturating_add(1);
                invalid |= char::from_u32(codepoint).is_none()
                    || table
                        .glyph_index(codepoint)
                        .is_some_and(|glyph| glyph.0 >= glyphs);
            }
        });
    }
    ensure!(
        !invalid && visits <= MAX_MAPPING_VISITS,
        "Font character mappings exceed bounded valid Unicode"
    );
    let scale = 128.0 / f32::from(face.units_per_em());
    let mut total = 0usize;
    for id in 0..glyphs {
        let mut outline = OutlineBound::default();
        face.outline_glyph(ttf_parser::GlyphId(id), &mut outline);
        ensure!(
            !outline.invalid && outline.segments <= 2048,
            "Font glyph outline exceeds budget"
        );
        if outline.seen {
            ensure!(
                outline
                    .minimum
                    .iter()
                    .chain(&outline.maximum)
                    .all(|coordinate| (coordinate * scale).is_finite()
                        && (coordinate * scale).abs() <= 4096.0),
                "Font glyph absolute coordinates exceed budget"
            );
            ensure!(
                (outline.maximum[0] - outline.minimum[0]) * scale <= 1024.0
                    && (outline.maximum[1] - outline.minimum[1]) * scale <= 1024.0,
                "Font glyph coordinate extent exceeds budget"
            );
        }
        total = total
            .checked_add(outline.segments)
            .context("Font outline count overflow")?;
        ensure!(
            total <= MAX_OUTLINE_SEGMENTS,
            "Font outlines exceed total work budget"
        );
    }
    Ok((glyphs, total, visits))
}

impl PreparedText {
    #[allow(clippy::too_many_arguments)]
    pub fn from_resource(
        resource: &SceneResource,
        width: u32,
        height: u32,
        point_size: f32,
        padding: u32,
        horizontal: &str,
        vertical: &str,
    ) -> Result<Self> {
        ensure!(
            resource.len() <= MAX_FONT_BYTES as u64,
            "Font source exceeds bounded byte size"
        );
        let mut reservation =
            super::scene_texture_memory_budget().reserve_bytes(resource.len() as usize)?;
        let bytes = resource.read_range(0, resource.len() as usize, MAX_FONT_BYTES)?;
        reservation.resize(bytes.capacity())?;
        Self::from_owned(
            EncodedFont { bytes, reservation },
            resource.origin(),
            width,
            height,
            point_size,
            padding,
            horizontal,
            vertical,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn from_owned(
        mut source: EncodedFont,
        origin: ResourceOrigin,
        width: u32,
        height: u32,
        point_size: f32,
        padding: u32,
        horizontal: &str,
        vertical: &str,
    ) -> Result<Self> {
        ensure!(
            width > 0
                && height > 0
                && width <= 2048
                && height <= 2048
                && u64::from(width) * u64::from(height) * 4 <= 16 * 1024 * 1024,
            "Text extent exceeds bounded surface size"
        );
        ensure!(
            point_size.is_finite() && (1.0..=128.0).contains(&point_size),
            "Text point size exceeds budget"
        );
        ensure!(
            padding <= width / 2 && padding <= height / 2,
            "Text padding outside authored extent"
        );
        let horizontal = align(horizontal, false)?;
        let vertical = align(vertical, true)?;
        let fingerprint: [u8; 32] = Sha256::digest(&source.bytes).into();
        let mut cache = FONTS
            .lock()
            .map_err(|_| anyhow::anyhow!("Font cache unavailable"))?;
        cache.retain(|_, value| value.strong_count() != 0);
        let font = if let Some(font) = cache.get(&fingerprint).and_then(Weak::upgrade) {
            drop(source);
            font
        } else {
            ensure!(cache.len() < 256, "Too many retained font identities");
            source.reservation.resize(
                source
                    .bytes
                    .capacity()
                    .checked_add(FONT_WORK_RESERVATION)
                    .context("Font reservation overflow")?,
            )?;
            let (glyph_count, outline_segments, mapping_visits) = preflight(&source.bytes)?;
            let font = std::panic::catch_unwind(|| {
                Font::from_bytes(
                    source.bytes.as_slice(),
                    FontSettings {
                        load_substitutions: false,
                        ..FontSettings::default()
                    },
                )
            })
            .map_err(|_| anyhow::anyhow!("Font parser stopped unexpectedly"))?
            .map_err(|_| anyhow::anyhow!("Cannot rasterize resolved font"))?;
            let owner = Arc::new(FontOwner {
                font,
                _encoded: source.bytes,
                glyph_count,
                outline_segments,
                mapping_visits,
                _reservation: source.reservation,
            });
            cache.insert(fingerprint, Arc::downgrade(&owner));
            owner
        };
        Ok(Self {
            font,
            width,
            height,
            point_size,
            padding,
            font_sha256: fingerprint,
            font_origin: origin,
            horizontal,
            vertical,
        })
    }

    pub fn rasterize(&self, value: &str) -> Result<TextFrame> {
        ensure!(
            value.len() <= 4096 && value.chars().count() <= 256 && value.split('\n').count() <= 8,
            "Text content exceeds layout budget"
        );
        ensure!(
            value.chars().all(|c| c == '\n' || !c.is_control()),
            "Unsupported control character in text"
        );
        let bytes = self.width as usize * self.height as usize * 4;
        let mut reservation = super::scene_texture_memory_budget().reserve_bytes(bytes)?;
        let mut rgba = Vec::new();
        rgba.try_reserve_exact(bytes)
            .context("Cannot allocate bounded text surface")?;
        reservation.resize(rgba.capacity())?;
        rgba.resize(bytes, 0);
        let metrics = self
            .font
            .font
            .horizontal_line_metrics(self.point_size)
            .context("Font lacks horizontal metrics")?;
        ensure!(
            metrics.new_line_size.is_finite()
                && metrics.new_line_size > 0.0
                && metrics.new_line_size <= 1024.0
                && metrics.ascent.is_finite(),
            "Invalid font line metrics"
        );
        let lines = value.split('\n').count();
        let used_height = metrics.new_line_size * lines as f32;
        let available_height = self.height as f32 - 2.0 * self.padding as f32;
        let start_y = self.padding as f32 + aligned(self.vertical, available_height, used_height);
        for (row, line) in value.split('\n').enumerate() {
            let mut advance = 0.0f32;
            let mut previous = None;
            for character in line.chars() {
                ensure!(
                    self.font.font.chars().contains_key(&character),
                    "Resolved font lacks a requested character"
                );
                if let Some(previous) = previous {
                    advance += self
                        .font
                        .font
                        .horizontal_kern(previous, character, self.point_size)
                        .unwrap_or(0.0);
                }
                advance += self
                    .font
                    .font
                    .metrics(character, self.point_size)
                    .advance_width;
                previous = Some(character);
            }
            ensure!(
                advance.is_finite() && advance.abs() <= 262144.0,
                "Text line advance exceeds bounds"
            );
            let mut x = self.padding as f32
                + aligned(
                    self.horizontal,
                    self.width as f32 - 2.0 * self.padding as f32,
                    advance,
                );
            let baseline = start_y + row as f32 * metrics.new_line_size + metrics.ascent;
            let mut previous = None;
            for character in line.chars() {
                if let Some(previous) = previous {
                    x += self
                        .font
                        .font
                        .horizontal_kern(previous, character, self.point_size)
                        .unwrap_or(0.0);
                }
                let bounds = self.font.font.metrics(character, self.point_size);
                ensure!(
                    bounds.width <= 1024
                        && bounds.height <= 1024
                        && bounds
                            .width
                            .checked_mul(bounds.height)
                            .is_some_and(|n| n <= MAX_GLYPH_PIXELS),
                    "Glyph bitmap exceeds work budget"
                );
                // Pinned fontdue 0.9.3 keeps its f32 coverage canvas alive while
                // allocating the u8 result: (pixels + 3)*4 plus pixels bytes.
                // Its disabled SIMD feature avoids the alternate padded bitmap.
                let pixels = bounds.width * bounds.height;
                let scratch_bytes = pixels
                    .checked_add(3)
                    .and_then(|n| n.checked_mul(std::mem::size_of::<f32>()))
                    .and_then(|n| n.checked_add(pixels))
                    .context("Glyph scratch reservation overflow")?;
                let _scratch = super::scene_texture_memory_budget().reserve_bytes(scratch_bytes)?;
                let (bounds, bitmap) = self.font.font.rasterize(character, self.point_size);
                ensure!(
                    bitmap.len() == bounds.width * bounds.height && bitmap.capacity() <= pixels,
                    "Glyph bitmap allocation exceeds reservation"
                );
                let left = x.floor() as i64 + i64::from(bounds.xmin);
                let top = baseline.floor() as i64 - i64::from(bounds.ymin) - bounds.height as i64;
                for by in 0..bounds.height {
                    let py = top + by as i64;
                    if py < 0 || py >= self.height as i64 {
                        continue;
                    }
                    for bx in 0..bounds.width {
                        let px = left + bx as i64;
                        if px < 0 || px >= self.width as i64 {
                            continue;
                        }
                        let coverage = bitmap[by * bounds.width + bx];
                        let to = (py as usize * self.width as usize + px as usize) * 4;
                        rgba[to..to + 3].fill(255);
                        let old = u16::from(rgba[to + 3]);
                        rgba[to + 3] =
                            (u16::from(coverage) + old * (255 - u16::from(coverage)) / 255) as u8;
                    }
                }
                x += bounds.advance_width;
                previous = Some(character);
            }
        }
        let mut digest = Sha256::new();
        digest.update(self.font_sha256);
        // Resource namespace is retained in the prepared surface identity even
        // when identical encoded font bytes share immutable font geometry.
        digest.update([match self.font_origin {
            ResourceOrigin::Package => 0,
            ResourceOrigin::Project => 1,
            ResourceOrigin::Builtin => 2,
        }]);
        digest.update(self.width.to_le_bytes());
        digest.update(self.height.to_le_bytes());
        digest.update(&rgba);
        Ok(TextFrame(Arc::new(FrameInner {
            width: self.width,
            height: self.height,
            fingerprint: digest.finalize().into(),
            rgba,
            _reservation: reservation,
        })))
    }
    #[cfg(test)]
    pub fn font_stats(&self) -> (u16, usize, usize, usize) {
        (
            self.font.glyph_count,
            self.font.outline_segments,
            self.font.mapping_visits,
            self.font._encoded.capacity(),
        )
    }
    /// Deduplication identity and charged logical bytes. The charge combines
    /// encoded Vec capacity with the conservative private-font workspace;
    /// it does not measure Fontdue's inaccessible allocator capacities or RSS.
    pub fn font_allocation(&self) -> (usize, u64) {
        (
            Arc::as_ptr(&self.font) as usize,
            self.font._reservation.bytes() as u64,
        )
    }
    #[cfg(test)]
    pub fn font_reservation_bytes(&self) -> usize {
        self.font._reservation.bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unmodeled_fonts_and_unbounded_layout_metadata_are_rejected() {
        assert!(preflight(&[]).is_err());
        assert!(preflight(b"wOFFmalformed").is_err());
        assert!(align("justify", false).is_err());
        assert!(align("left", true).is_err());
    }
    #[test]
    #[ignore = "requires private resolved original font and explicit CPU-only opt-in"]
    fn resolved_original_font_renders_bounded_framerate_surface() {
        assert_eq!(std::env::var("LWE_TEXT_FONT_ORACLE").as_deref(), Ok("1"));
        let project = std::path::PathBuf::from(
            std::env::var_os("LWE_TEXT_FONT_PROJECT").expect("private project"),
        );
        let assets = std::path::PathBuf::from(
            std::env::var_os("LWE_TEXT_FONT_ASSETS").expect("private assets"),
        );
        let vfs = lwe_scene::SceneAssets::open(&project, Some(&assets)).unwrap();
        let baseline = super::super::scene_texture_memory_budget()
            .usage()
            .charged_bytes;
        let text = PreparedText::from_resource(
            &vfs.resource("fonts/Atami-Regular.otf").unwrap(),
            153,
            100,
            20.0,
            32,
            "center",
            "center",
        )
        .unwrap();
        assert_eq!(text.font_origin, ResourceOrigin::Builtin);
        let before = super::super::scene_texture_memory_budget()
            .usage()
            .charged_bytes;
        let second = PreparedText::from_resource(
            &vfs.resource("fonts/Atami-Regular.otf").unwrap(),
            153,
            100,
            20.0,
            32,
            "center",
            "center",
        )
        .unwrap();
        assert_eq!(text.font_allocation(), second.font_allocation());
        assert_eq!(
            super::super::scene_texture_memory_budget()
                .usage()
                .charged_bytes,
            before
        );
        // A rejected encoded source releases both source capacity and its lease.
        let mut bad = Vec::new();
        bad.extend_from_slice(b"wOFFinvalid");
        let reservation = super::super::scene_texture_memory_budget()
            .reserve_bytes(bad.capacity())
            .unwrap();
        assert!(
            PreparedText::from_owned(
                EncodedFont {
                    bytes: bad,
                    reservation
                },
                ResourceOrigin::Builtin,
                153,
                100,
                20.0,
                32,
                "center",
                "center"
            )
            .is_err()
        );
        assert_eq!(
            super::super::scene_texture_memory_budget()
                .usage()
                .charged_bytes,
            before
        );
        let empty = text.rasterize("").unwrap();
        assert!(empty.rgba().iter().all(|v| *v == 0));
        drop(empty);
        let pixels = text.rasterize("FPS: 30\n Avg: 30").unwrap();
        assert_eq!(pixels.rgba().len(), 153 * 100 * 4);
        assert!(
            pixels
                .rgba()
                .as_chunks::<4>()
                .0
                .iter()
                .any(|pixel| pixel[3] > 0)
        );
        assert!(text.rasterize(&"x".repeat(257)).is_err());
        assert!(text.rasterize("\0").is_err());
        let report = serde_json::json!({
            "ok":true,"kind":"resolved_original_font_cpu_only",
            "font_origin":format!("{:?}",text.font_origin),
            "font_sha256":text.font_sha256.iter().map(|v|format!("{v:02x}")).collect::<String>(),
            "glyphs":text.font_stats().0,"outline_control_point_visits":text.font_stats().1,"cmap_mapping_visits":text.font_stats().2,
            "encoded_capacity":text.font_stats().3,"logical_font_reservation":text.font_reservation_bytes(),
            "extent":[text.width,text.height],"pointsize":text.point_size,"padding":text.padding,
            "shared_two_generation_font_allocation":true,"failed_source_charge_released":true,
            "frame_capacity":pixels.capacity_bytes(),"covered_pixels":pixels.rgba().as_chunks::<4>().0.iter().filter(|p|p[3]>0).count(),
            "frame_sha256":pixels.fingerprint().iter().map(|v|format!("{v:02x}")).collect::<String>(),
            "bounds":"FontDue private geometry is a conservative logical reservation, not exact allocation capacities or RSS. No Windows text layout or desktop acceptance."
        });
        if let Some(path) = std::env::var_os("LWE_TEXT_FONT_REPORT") {
            std::fs::write(path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
        }
        println!(
            "font statistics {:?}, retained logical reservation {}, frame capacity {}",
            text.font_stats(),
            text.font_reservation_bytes(),
            pixels.capacity_bytes()
        );
        drop(pixels);
        assert_eq!(
            super::super::scene_texture_memory_budget()
                .usage()
                .charged_bytes,
            before
        );
        drop(second);
        drop(text);
        assert_eq!(
            super::super::scene_texture_memory_budget()
                .usage()
                .charged_bytes,
            baseline
        );
    }
}

//! TEX texture format parser for Wallpaper Engine
//!
//! Wallpaper Engine uses a custom .tex format with the following structure:
//! - TEXV0005: Container header
//! - TEXI0001: Texture info (format, dimensions, flags)
//! - TEXB0001/0002/0003/0004: Body with mipmap data (optionally LZ4 compressed)
//!
//! Reference: Almamu/linux-wallpaperengine

use anyhow::{anyhow, Result};
use tracing::{debug, trace, warn};

/// Texture format constants from Wallpaper Engine
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum TextureFormat {
    ARGB8888 = 0,
    RGB888 = 1,
    RGB565 = 2,
    DXT5 = 4,
    DXT3 = 6,
    DXT1 = 7,
    RG88 = 8,
    R8 = 9,
    RG1616f = 10,
    R16f = 11,
    BC7 = 12,
    RGBa1010102 = 13,
    RGBA16161616f = 14,
    RGB161616f = 15,
    Unknown = 0xFFFFFFFF,
}

impl TextureFormat {
    fn from_u32(v: u32) -> Self {
        match v {
            0 => Self::ARGB8888,
            1 => Self::RGB888,
            2 => Self::RGB565,
            4 => Self::DXT5,
            6 => Self::DXT3,
            7 => Self::DXT1,
            8 => Self::RG88,
            9 => Self::R8,
            10 => Self::RG1616f,
            11 => Self::R16f,
            12 => Self::BC7,
            13 => Self::RGBa1010102,
            14 => Self::RGBA16161616f,
            15 => Self::RGB161616f,
            _ => Self::Unknown,
        }
    }

    /// Get bytes per pixel for uncompressed formats
    pub fn bytes_per_pixel(&self) -> Option<u32> {
        match self {
            Self::ARGB8888 => Some(4),
            Self::RGB888 => Some(3),
            Self::RGB565 => Some(2),
            Self::RG88 => Some(2),
            Self::R8 => Some(1),
            Self::RG1616f => Some(4),
            Self::R16f => Some(2),
            Self::RGBa1010102 => Some(4),
            Self::RGBA16161616f => Some(8),
            Self::RGB161616f => Some(6),
            _ => None, // Compressed formats
        }
    }

    /// Check if format is block compressed (DXT/BC)
    pub fn is_compressed(&self) -> bool {
        matches!(self, Self::DXT1 | Self::DXT3 | Self::DXT5 | Self::BC7)
    }

    /// Get block size for compressed formats (always 4x4)
    pub fn block_size(&self) -> Option<(u32, u32)> {
        if self.is_compressed() {
            Some((4, 4))
        } else {
            None
        }
    }

    /// Get bytes per block for compressed formats
    pub fn bytes_per_block(&self) -> Option<u32> {
        match self {
            Self::DXT1 => Some(8),               // 8 bytes per 4x4 block
            Self::DXT3 | Self::DXT5 => Some(16), // 16 bytes per 4x4 block
            Self::BC7 => Some(16),
            _ => None,
        }
    }
}

/// Texture flags
#[derive(Debug, Clone, Copy, Default)]
pub struct TextureFlags {
    pub no_interpolation: bool,
    pub clamp_uvs: bool,
    pub is_gif: bool,
    pub clamp_uvs_border: bool,
}

impl TextureFlags {
    fn from_u32(v: u32) -> Self {
        Self {
            no_interpolation: (v & 1) != 0,
            clamp_uvs: (v & 2) != 0,
            is_gif: (v & 4) != 0,
            clamp_uvs_border: (v & 8) != 0,
        }
    }
}

/// Container version
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerVersion {
    TEXB0001,
    TEXB0002,
    TEXB0003,
    TEXB0004,
}

/// A single mipmap level
#[derive(Debug, Clone)]
pub struct TextureMipmap {
    pub width: u32,
    pub height: u32,
    /// RGBA8 pixel data (converted from source format)
    pub data: Vec<u8>,
}

/// Parsed texture
#[derive(Debug)]
pub struct Texture {
    pub format: TextureFormat,
    pub flags: TextureFlags,
    /// Original texture width (may be power of 2)
    pub texture_width: u32,
    /// Original texture height (may be power of 2)
    pub texture_height: u32,
    /// Actual image width
    pub width: u32,
    /// Actual image height
    pub height: u32,
    /// Mipmap levels (level 0 is full resolution)
    pub mipmaps: Vec<TextureMipmap>,
}

impl Texture {
    /// Parse a .tex file from raw bytes
    pub fn parse(data: &[u8]) -> Result<Self> {
        let mut pos = 0usize;

        // Check TEXV header
        if data.len() < 18 {
            return Err(anyhow!("TEX file too small"));
        }

        if &data[pos..pos + 8] != b"TEXV0005" {
            return Err(anyhow!(
                "Invalid TEX header: expected TEXV0005, got {:?}",
                &data[pos..pos + 8]
            ));
        }
        pos += 9; // Including null terminator

        // Check TEXI header
        if &data[pos..pos + 8] != b"TEXI0001" {
            return Err(anyhow!(
                "Invalid TEX sub-header: expected TEXI0001, got {:?}",
                &data[pos..pos + 8]
            ));
        }
        pos += 9;

        // Read texture info
        let format_val = read_u32(&data, &mut pos)?;
        let flags_val = read_u32(&data, &mut pos)?;
        let texture_width = read_u32(&data, &mut pos)?;
        let texture_height = read_u32(&data, &mut pos)?;
        let width = read_u32(&data, &mut pos)?;
        let height = read_u32(&data, &mut pos)?;

        let format = TextureFormat::from_u32(format_val);
        let flags = TextureFlags::from_u32(flags_val);

        debug!(
            "TEX info: format={:?}, flags={:?}, tex_size={}x{}, size={}x{}",
            format, flags, texture_width, texture_height, width, height
        );

        // Skip to TEXB (there might be padding/unknown data)
        let texb_pos =
            find_magic(&data[pos..], b"TEXB").ok_or_else(|| anyhow!("TEXB section not found"))?;
        pos += texb_pos;

        // Parse container version
        let container_version = if &data[pos..pos + 8] == b"TEXB0004" {
            ContainerVersion::TEXB0004
        } else if &data[pos..pos + 8] == b"TEXB0003" {
            ContainerVersion::TEXB0003
        } else if &data[pos..pos + 8] == b"TEXB0002" {
            ContainerVersion::TEXB0002
        } else if &data[pos..pos + 8] == b"TEXB0001" {
            ContainerVersion::TEXB0001
        } else {
            return Err(anyhow!(
                "Unknown TEXB version: {:?}",
                String::from_utf8_lossy(&data[pos..pos + 8])
            ));
        };
        pos += 9; // Including null

        debug!("Container version: {:?}", container_version);

        // Read image count (frames for animated textures, usually 1)
        let image_count = read_u32(&data, &mut pos)?;
        debug!("Image/frame count: {}", image_count);

        // TEXB0004 has a different structure:
        // After image_count, there are always 3 extra u32 fields:
        // - unknown1 (various values observed: 13, etc)
        // - unknown2 (usually 0)
        // - mip_count (actual number of mipmaps)
        // Then for each mip: width, height, compression, uncompressed_size, compressed_size, data
        let mip_count = if container_version == ContainerVersion::TEXB0004 {
            let unknown1 = read_u32(&data, &mut pos)?;
            let unknown2 = read_u32(&data, &mut pos)?;
            let count = read_u32(&data, &mut pos)?;
            debug!(
                "TEXB0004: unknown1={}, unknown2={}, mip_count={}",
                unknown1, unknown2, count
            );
            count
        } else {
            image_count
        };

        // For TEXB0003, there's an additional format byte
        let _free_image_format = if container_version == ContainerVersion::TEXB0003 {
            let fmt = data.get(pos).copied().unwrap_or(0);
            pos += 1;
            Some(fmt)
        } else {
            None
        };

        // Parse mipmaps
        let mut mipmaps = Vec::new();

        for mip_level in 0..mip_count {
            let mip_width = read_u32(&data, &mut pos)?;
            let mip_height = read_u32(&data, &mut pos)?;

            // TEXB0001 uses a simpler format, newer versions have compression info
            let (is_compressed, uncompressed_size, compressed_size) = match container_version {
                ContainerVersion::TEXB0001 => {
                    // TEXB0001: just raw data, size calculated from format
                    let size = calculate_data_size(format, mip_width, mip_height);
                    (false, size, size)
                }
                ContainerVersion::TEXB0002
                | ContainerVersion::TEXB0003
                | ContainerVersion::TEXB0004 => {
                    let compression_flag = read_u32(&data, &mut pos)?;
                    let uncompressed = read_u32(&data, &mut pos)?;
                    let compressed = read_u32(&data, &mut pos)?;
                    debug!(
                        "Mip {}: {}x{}, comp_flag={}, uncompressed={}, compressed={}",
                        mip_level,
                        mip_width,
                        mip_height,
                        compression_flag,
                        uncompressed,
                        compressed
                    );
                    (compression_flag == 1, uncompressed, compressed)
                }
            };

            trace!(
                "Mip {}: {}x{}, compressed={}, sizes={}/{}",
                mip_level,
                mip_width,
                mip_height,
                is_compressed,
                compressed_size,
                uncompressed_size
            );

            // Handle -1 (0xFFFFFFFF) as uncompressed marker
            let actual_compressed = if compressed_size == 0xFFFFFFFF {
                false
            } else {
                is_compressed
            };

            let data_size = if actual_compressed {
                compressed_size as usize
            } else {
                uncompressed_size as usize
            };

            // Sanity check: prevent ridiculously large allocations
            // Max reasonable texture size: 16384x16384x4 = ~1GB
            const MAX_TEXTURE_SIZE: usize = 16384 * 16384 * 4;
            if data_size > MAX_TEXTURE_SIZE || uncompressed_size > MAX_TEXTURE_SIZE as u32 {
                warn!(
                    "Mipmap {} has unreasonable size: data_size={}, uncompressed={}. Skipping.",
                    mip_level, data_size, uncompressed_size
                );
                break;
            }

            if pos + data_size > data.len() {
                warn!(
                    "Not enough data for mipmap {}: need {} bytes, have {}",
                    mip_level,
                    data_size,
                    data.len() - pos
                );
                break;
            }

            let raw_data = &data[pos..pos + data_size];
            pos += data_size;

            // Decompress if needed
            let texture_data = if actual_compressed {
                decompress_lz4(raw_data, uncompressed_size as usize)?
            } else {
                raw_data.to_vec()
            };

            // Convert to RGBA8
            let rgba_data = convert_to_rgba(format, mip_width, mip_height, &texture_data)?;

            mipmaps.push(TextureMipmap {
                width: mip_width,
                height: mip_height,
                data: rgba_data,
            });
        }

        Ok(Self {
            format,
            flags,
            texture_width,
            texture_height,
            width,
            height,
            mipmaps,
        })
    }

    /// Get the primary (level 0) mipmap data as RGBA8
    pub fn rgba_data(&self) -> Option<&[u8]> {
        self.mipmaps.first().map(|m| m.data.as_slice())
    }
}

/// Read u32 little-endian
fn read_u32(data: &[u8], pos: &mut usize) -> Result<u32> {
    if *pos + 4 > data.len() {
        return Err(anyhow!("Unexpected end of data reading u32 at {}", pos));
    }
    let val = u32::from_le_bytes([data[*pos], data[*pos + 1], data[*pos + 2], data[*pos + 3]]);
    *pos += 4;
    Ok(val)
}

/// Find magic bytes in data
fn find_magic(data: &[u8], magic: &[u8]) -> Option<usize> {
    data.windows(magic.len()).position(|w| w == magic)
}

/// Calculate expected data size for a format
fn calculate_data_size(format: TextureFormat, width: u32, height: u32) -> u32 {
    if let Some(bpp) = format.bytes_per_pixel() {
        width * height * bpp
    } else if let Some(bytes_per_block) = format.bytes_per_block() {
        // Block compressed: round up to block boundaries
        let blocks_x = (width + 3) / 4;
        let blocks_y = (height + 3) / 4;
        blocks_x * blocks_y * bytes_per_block
    } else {
        width * height * 4 // Default to RGBA
    }
}

/// Decompress LZ4 data
fn decompress_lz4(compressed: &[u8], uncompressed_size: usize) -> Result<Vec<u8>> {
    // Try lz4_flex block decompression first
    match lz4_flex::block::decompress(compressed, uncompressed_size) {
        Ok(decompressed) => {
            if decompressed.len() != uncompressed_size {
                warn!(
                    "LZ4 decompression size mismatch: expected {}, got {}",
                    uncompressed_size,
                    decompressed.len()
                );
            }
            // Pad or truncate to expected size
            let mut output = vec![0u8; uncompressed_size];
            let copy_len = decompressed.len().min(uncompressed_size);
            output[..copy_len].copy_from_slice(&decompressed[..copy_len]);
            Ok(output)
        }
        Err(e) => {
            // Fall back: try treating data as uncompressed
            warn!("LZ4 decompression failed ({}), treating as uncompressed", e);
            let mut output = vec![0u8; uncompressed_size];
            let copy_len = compressed.len().min(uncompressed_size);
            output[..copy_len].copy_from_slice(&compressed[..copy_len]);
            Ok(output)
        }
    }
}

/// Convert texture data to RGBA8 format
fn convert_to_rgba(format: TextureFormat, width: u32, height: u32, data: &[u8]) -> Result<Vec<u8>> {
    let pixel_count = (width as usize).saturating_mul(height as usize);
    let expected_size = match format {
        TextureFormat::ARGB8888 => pixel_count * 4,
        TextureFormat::RGB888 => pixel_count * 3,
        TextureFormat::RGB565 | TextureFormat::RG88 | TextureFormat::R16f => pixel_count * 2,
        TextureFormat::R8 => pixel_count,
        TextureFormat::RG1616f | TextureFormat::RGBa1010102 => pixel_count * 4,
        TextureFormat::RGBA16161616f => pixel_count * 8,
        TextureFormat::RGB161616f => pixel_count * 6,
        TextureFormat::DXT1 => ((width + 3) / 4) as usize * ((height + 3) / 4) as usize * 8,
        TextureFormat::DXT3 | TextureFormat::DXT5 | TextureFormat::BC7 => {
            ((width + 3) / 4) as usize * ((height + 3) / 4) as usize * 16
        }
        _ => pixel_count * 4,
    };

    trace!(
        "Converting {}x{} {:?}: input {} bytes, expected {} bytes",
        width,
        height,
        format,
        data.len(),
        expected_size
    );

    let mut rgba = vec![0u8; pixel_count.saturating_mul(4)];

    match format {
        TextureFormat::ARGB8888 => {
            // ARGB -> RGBA
            for i in 0..pixel_count {
                let src = i * 4;
                let dst = i * 4;
                if src + 4 <= data.len() {
                    rgba[dst] = data[src + 1]; // R
                    rgba[dst + 1] = data[src + 2]; // G
                    rgba[dst + 2] = data[src + 3]; // B
                    rgba[dst + 3] = data[src]; // A
                }
            }
        }
        TextureFormat::RGB888 => {
            for i in 0..pixel_count {
                let src = i * 3;
                let dst = i * 4;
                if src + 3 <= data.len() {
                    rgba[dst] = data[src];
                    rgba[dst + 1] = data[src + 1];
                    rgba[dst + 2] = data[src + 2];
                    rgba[dst + 3] = 255;
                }
            }
        }
        TextureFormat::RGB565 => {
            for i in 0..pixel_count {
                let src = i * 2;
                let dst = i * 4;
                if src + 2 <= data.len() {
                    let pixel = u16::from_le_bytes([data[src], data[src + 1]]);
                    rgba[dst] = ((pixel >> 11) as u8) << 3; // R
                    rgba[dst + 1] = ((pixel >> 5) as u8 & 0x3F) << 2; // G
                    rgba[dst + 2] = (pixel as u8 & 0x1F) << 3; // B
                    rgba[dst + 3] = 255;
                }
            }
        }
        TextureFormat::R8 => {
            for i in 0..pixel_count {
                if i < data.len() {
                    let v = data[i];
                    rgba[i * 4] = v;
                    rgba[i * 4 + 1] = v;
                    rgba[i * 4 + 2] = v;
                    rgba[i * 4 + 3] = 255;
                }
            }
        }
        TextureFormat::RG88 => {
            for i in 0..pixel_count {
                let src = i * 2;
                let dst = i * 4;
                if src + 2 <= data.len() {
                    rgba[dst] = data[src]; // R
                    rgba[dst + 1] = data[src + 1]; // G
                    rgba[dst + 2] = 0;
                    rgba[dst + 3] = 255;
                }
            }
        }
        TextureFormat::DXT1 => {
            decode_dxt1(data, width, height, &mut rgba)?;
        }
        TextureFormat::DXT3 => {
            decode_dxt3(data, width, height, &mut rgba)?;
        }
        TextureFormat::DXT5 => {
            decode_dxt5(data, width, height, &mut rgba)?;
        }
        TextureFormat::BC7 => {
            // BC7 is complex, for now just fill with magenta to indicate unsupported
            warn!("BC7 format not yet supported, using placeholder");
            for i in 0..pixel_count {
                rgba[i * 4] = 255;
                rgba[i * 4 + 1] = 0;
                rgba[i * 4 + 2] = 255;
                rgba[i * 4 + 3] = 255;
            }
        }
        _ => {
            warn!("Unsupported texture format {:?}, using placeholder", format);
            for i in 0..pixel_count {
                rgba[i * 4] = 128;
                rgba[i * 4 + 1] = 128;
                rgba[i * 4 + 2] = 128;
                rgba[i * 4 + 3] = 255;
            }
        }
    }

    Ok(rgba)
}

// DXT1 decoding
fn decode_dxt1(data: &[u8], width: u32, height: u32, output: &mut [u8]) -> Result<()> {
    let blocks_x = (width + 3) / 4;
    let blocks_y = (height + 3) / 4;

    for by in 0..blocks_y {
        for bx in 0..blocks_x {
            let block_idx = (by * blocks_x + bx) as usize;
            let block_offset = block_idx * 8;

            if block_offset + 8 > data.len() {
                continue;
            }

            let block = &data[block_offset..block_offset + 8];
            decode_dxt1_block(block, bx * 4, by * 4, width, height, output);
        }
    }

    Ok(())
}

fn decode_dxt1_block(block: &[u8], bx: u32, by: u32, width: u32, height: u32, output: &mut [u8]) {
    let c0 = u16::from_le_bytes([block[0], block[1]]);
    let c1 = u16::from_le_bytes([block[2], block[3]]);

    let r0 = ((c0 >> 11) & 0x1F) as u8;
    let g0 = ((c0 >> 5) & 0x3F) as u8;
    let b0 = (c0 & 0x1F) as u8;

    let r1 = ((c1 >> 11) & 0x1F) as u8;
    let g1 = ((c1 >> 5) & 0x3F) as u8;
    let b1 = (c1 & 0x1F) as u8;

    // Expand to 8-bit
    let colors: [[u8; 4]; 4] = if c0 > c1 {
        [
            [r0 << 3 | r0 >> 2, g0 << 2 | g0 >> 4, b0 << 3 | b0 >> 2, 255],
            [r1 << 3 | r1 >> 2, g1 << 2 | g1 >> 4, b1 << 3 | b1 >> 2, 255],
            [
                ((2 * (r0 as u16) + r1 as u16) / 3) as u8 * 8,
                ((2 * (g0 as u16) + g1 as u16) / 3) as u8 * 4,
                ((2 * (b0 as u16) + b1 as u16) / 3) as u8 * 8,
                255,
            ],
            [
                ((r0 as u16 + 2 * r1 as u16) / 3) as u8 * 8,
                ((g0 as u16 + 2 * g1 as u16) / 3) as u8 * 4,
                ((b0 as u16 + 2 * b1 as u16) / 3) as u8 * 8,
                255,
            ],
        ]
    } else {
        [
            [r0 << 3 | r0 >> 2, g0 << 2 | g0 >> 4, b0 << 3 | b0 >> 2, 255],
            [r1 << 3 | r1 >> 2, g1 << 2 | g1 >> 4, b1 << 3 | b1 >> 2, 255],
            [
                (((r0 as u16 + r1 as u16) / 2) as u8) << 3,
                (((g0 as u16 + g1 as u16) / 2) as u8) << 2,
                (((b0 as u16 + b1 as u16) / 2) as u8) << 3,
                255,
            ],
            [0, 0, 0, 0], // Transparent
        ]
    };

    let indices = u32::from_le_bytes([block[4], block[5], block[6], block[7]]);

    for py in 0..4 {
        for px in 0..4 {
            let x = bx + px;
            let y = by + py;
            if x >= width || y >= height {
                continue;
            }

            let idx = ((indices >> ((py * 4 + px) * 2)) & 0x3) as usize;
            let pixel_offset = ((y * width + x) * 4) as usize;

            if pixel_offset + 3 < output.len() {
                output[pixel_offset] = colors[idx][0];
                output[pixel_offset + 1] = colors[idx][1];
                output[pixel_offset + 2] = colors[idx][2];
                output[pixel_offset + 3] = colors[idx][3];
            }
        }
    }
}

fn decode_dxt3(data: &[u8], width: u32, height: u32, output: &mut [u8]) -> Result<()> {
    let blocks_x = (width + 3) / 4;
    let blocks_y = (height + 3) / 4;

    for by in 0..blocks_y {
        for bx in 0..blocks_x {
            let block_idx = (by * blocks_x + bx) as usize;
            let block_offset = block_idx * 16;

            if block_offset + 16 > data.len() {
                continue;
            }

            let block = &data[block_offset..block_offset + 16];
            decode_dxt3_block(block, bx * 4, by * 4, width, height, output);
        }
    }

    Ok(())
}

fn decode_dxt3_block(block: &[u8], bx: u32, by: u32, width: u32, height: u32, output: &mut [u8]) {
    // First 8 bytes are explicit alpha values
    let alpha_block = &block[0..8];
    // Last 8 bytes are DXT1 color block
    let color_block = &block[8..16];

    // Decode color part like DXT1 but without transparency
    let c0 = u16::from_le_bytes([color_block[0], color_block[1]]);
    let c1 = u16::from_le_bytes([color_block[2], color_block[3]]);

    let r0 = ((c0 >> 11) & 0x1F) as u8;
    let g0 = ((c0 >> 5) & 0x3F) as u8;
    let b0 = (c0 & 0x1F) as u8;

    let r1 = ((c1 >> 11) & 0x1F) as u8;
    let g1 = ((c1 >> 5) & 0x3F) as u8;
    let b1 = (c1 & 0x1F) as u8;

    let colors: [[u8; 3]; 4] = [
        [r0 << 3 | r0 >> 2, g0 << 2 | g0 >> 4, b0 << 3 | b0 >> 2],
        [r1 << 3 | r1 >> 2, g1 << 2 | g1 >> 4, b1 << 3 | b1 >> 2],
        [
            (((2 * r0 as u16 + r1 as u16) / 3) as u8) << 3,
            (((2 * g0 as u16 + g1 as u16) / 3) as u8) << 2,
            (((2 * b0 as u16 + b1 as u16) / 3) as u8) << 3,
        ],
        [
            (((r0 as u16 + 2 * r1 as u16) / 3) as u8) << 3,
            (((g0 as u16 + 2 * g1 as u16) / 3) as u8) << 2,
            (((b0 as u16 + 2 * b1 as u16) / 3) as u8) << 3,
        ],
    ];

    let indices = u32::from_le_bytes([
        color_block[4],
        color_block[5],
        color_block[6],
        color_block[7],
    ]);

    for py in 0..4u32 {
        for px in 0..4u32 {
            let x = bx + px;
            let y = by + py;
            if x >= width || y >= height {
                continue;
            }

            // Get alpha (4 bits per pixel)
            let alpha_byte_idx = (py * 2 + px / 2) as usize;
            let alpha_nibble = if px % 2 == 0 {
                alpha_block[alpha_byte_idx] & 0x0F
            } else {
                alpha_block[alpha_byte_idx] >> 4
            };
            let alpha = alpha_nibble << 4 | alpha_nibble;

            let color_idx = ((indices >> ((py * 4 + px) * 2)) & 0x3) as usize;
            let pixel_offset = ((y * width + x) * 4) as usize;

            if pixel_offset + 3 < output.len() {
                output[pixel_offset] = colors[color_idx][0];
                output[pixel_offset + 1] = colors[color_idx][1];
                output[pixel_offset + 2] = colors[color_idx][2];
                output[pixel_offset + 3] = alpha;
            }
        }
    }
}

fn decode_dxt5(data: &[u8], width: u32, height: u32, output: &mut [u8]) -> Result<()> {
    let blocks_x = (width + 3) / 4;
    let blocks_y = (height + 3) / 4;

    for by in 0..blocks_y {
        for bx in 0..blocks_x {
            let block_idx = (by * blocks_x + bx) as usize;
            let block_offset = block_idx * 16;

            if block_offset + 16 > data.len() {
                continue;
            }

            let block = &data[block_offset..block_offset + 16];
            decode_dxt5_block(block, bx * 4, by * 4, width, height, output);
        }
    }

    Ok(())
}

fn decode_dxt5_block(block: &[u8], bx: u32, by: u32, width: u32, height: u32, output: &mut [u8]) {
    // First 8 bytes are alpha block (2 endpoint alphas + 6 bytes of 3-bit indices)
    let alpha0 = block[0];
    let alpha1 = block[1];

    // Build alpha lookup table
    let alphas: [u8; 8] = if alpha0 > alpha1 {
        [
            alpha0,
            alpha1,
            ((6 * alpha0 as u16 + 1 * alpha1 as u16) / 7) as u8,
            ((5 * alpha0 as u16 + 2 * alpha1 as u16) / 7) as u8,
            ((4 * alpha0 as u16 + 3 * alpha1 as u16) / 7) as u8,
            ((3 * alpha0 as u16 + 4 * alpha1 as u16) / 7) as u8,
            ((2 * alpha0 as u16 + 5 * alpha1 as u16) / 7) as u8,
            ((1 * alpha0 as u16 + 6 * alpha1 as u16) / 7) as u8,
        ]
    } else {
        [
            alpha0,
            alpha1,
            ((4 * alpha0 as u16 + 1 * alpha1 as u16) / 5) as u8,
            ((3 * alpha0 as u16 + 2 * alpha1 as u16) / 5) as u8,
            ((2 * alpha0 as u16 + 3 * alpha1 as u16) / 5) as u8,
            ((1 * alpha0 as u16 + 4 * alpha1 as u16) / 5) as u8,
            0,
            255,
        ]
    };

    // Alpha indices are packed in 6 bytes (48 bits, 3 bits per pixel)
    let alpha_bits = u64::from_le_bytes([
        block[2], block[3], block[4], block[5], block[6], block[7], 0, 0,
    ]);

    // Color block (same as DXT1)
    let color_block = &block[8..16];
    let c0 = u16::from_le_bytes([color_block[0], color_block[1]]);
    let c1 = u16::from_le_bytes([color_block[2], color_block[3]]);

    let r0 = ((c0 >> 11) & 0x1F) as u8;
    let g0 = ((c0 >> 5) & 0x3F) as u8;
    let b0 = (c0 & 0x1F) as u8;

    let r1 = ((c1 >> 11) & 0x1F) as u8;
    let g1 = ((c1 >> 5) & 0x3F) as u8;
    let b1 = (c1 & 0x1F) as u8;

    let colors: [[u8; 3]; 4] = [
        [r0 << 3 | r0 >> 2, g0 << 2 | g0 >> 4, b0 << 3 | b0 >> 2],
        [r1 << 3 | r1 >> 2, g1 << 2 | g1 >> 4, b1 << 3 | b1 >> 2],
        [
            (((2 * r0 as u16 + r1 as u16) / 3) as u8) << 3,
            (((2 * g0 as u16 + g1 as u16) / 3) as u8) << 2,
            (((2 * b0 as u16 + b1 as u16) / 3) as u8) << 3,
        ],
        [
            (((r0 as u16 + 2 * r1 as u16) / 3) as u8) << 3,
            (((g0 as u16 + 2 * g1 as u16) / 3) as u8) << 2,
            (((b0 as u16 + 2 * b1 as u16) / 3) as u8) << 3,
        ],
    ];

    let color_indices = u32::from_le_bytes([
        color_block[4],
        color_block[5],
        color_block[6],
        color_block[7],
    ]);

    for py in 0..4u32 {
        for px in 0..4u32 {
            let x = bx + px;
            let y = by + py;
            if x >= width || y >= height {
                continue;
            }

            // Get alpha index (3 bits)
            let pixel_num = py * 4 + px;
            let alpha_idx = ((alpha_bits >> (pixel_num * 3)) & 0x7) as usize;
            let alpha = alphas[alpha_idx];

            let color_idx = ((color_indices >> (pixel_num * 2)) & 0x3) as usize;
            let pixel_offset = ((y * width + x) * 4) as usize;

            if pixel_offset + 3 < output.len() {
                output[pixel_offset] = colors[color_idx][0];
                output[pixel_offset + 1] = colors[color_idx][1];
                output[pixel_offset + 2] = colors[color_idx][2];
                output[pixel_offset + 3] = alpha;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_texture_format_bpp() {
        assert_eq!(TextureFormat::ARGB8888.bytes_per_pixel(), Some(4));
        assert_eq!(TextureFormat::RGB888.bytes_per_pixel(), Some(3));
        assert_eq!(TextureFormat::R8.bytes_per_pixel(), Some(1));
        assert_eq!(TextureFormat::DXT1.bytes_per_pixel(), None);
    }

    #[test]
    fn test_texture_format_compressed() {
        assert!(TextureFormat::DXT1.is_compressed());
        assert!(TextureFormat::DXT5.is_compressed());
        assert!(!TextureFormat::ARGB8888.is_compressed());
    }

    #[test]
    fn test_parse_real_tex_file() {
        // Try to parse a real .tex file from PKG
        let home = std::env::var("HOME").unwrap_or_default();
        let pkg_path = std::path::Path::new(&home)
            .join(".steam/steam/steamapps/workshop/content/431960/3578699777/scene.pkg");

        if !pkg_path.exists() {
            println!("Skipping: PKG not found");
            return;
        }

        // Use PKG reader to get a tex file
        use crate::we::scene::pkg::PkgReader;
        let pkg = PkgReader::open(&pkg_path).expect("Failed to open PKG");

        // Find a .tex file
        let tex_files: Vec<_> = pkg
            .list_files()
            .iter()
            .filter(|f| f.ends_with(".tex"))
            .cloned()
            .collect();

        if tex_files.is_empty() {
            println!("No .tex files found in PKG");
            return;
        }

        // Try to parse the first small tex file
        for tex_file in tex_files.iter().take(3) {
            println!("Parsing: {}", tex_file);
            match pkg.read_file(tex_file) {
                Ok(data) => match Texture::parse(&data) {
                    Ok(tex) => {
                        println!("  Format: {:?}", tex.format);
                        println!("  Size: {}x{}", tex.width, tex.height);
                        println!("  Mipmaps: {}", tex.mipmaps.len());
                        if let Some(mip) = tex.mipmaps.first() {
                            println!("  Mip0 data size: {} bytes", mip.data.len());
                        }
                    }
                    Err(e) => {
                        println!("  Parse error: {}", e);
                    }
                },
                Err(e) => {
                    println!("  Read error: {}", e);
                }
            }
        }
    }
}

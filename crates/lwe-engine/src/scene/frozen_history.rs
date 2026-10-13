//! Sealed CPU copies of unique histories while an old generation is frozen.
//! The presented output stays on the GPU; these bytes only restore its next tick.
use anyhow::{Context, Result};
use lwe_scene::{AtlasMemoryBudget, AtlasMemoryReservation};

use super::{RenderTarget, check_gl, rgba_bytes};

pub(super) struct FrozenHistory {
    pixels: Vec<u8>,
    width: i32,
    height: i32,
    // Pixels must drop before their capacity reservation.
    _reservation: AtlasMemoryReservation,
}

impl FrozenHistory {
    pub fn capture(target: &RenderTarget, budget: &AtlasMemoryBudget) -> Result<Self> {
        let width = target.texture.width;
        let height = target.texture.height;
        let bytes = usize::try_from(rgba_bytes(width as u64, height as u64, 1)?)?;
        let mut reservation = budget.reserve_bytes(bytes)?;
        let mut pixels = Vec::new();
        pixels
            .try_reserve_exact(bytes)
            .context("Reserve frozen Scene history")?;
        reservation.resize(pixels.capacity())?;
        pixels.resize(bytes, 0);
        let _pack = PackState::capture();
        unsafe {
            gl::BindBuffer(gl::PIXEL_PACK_BUFFER, 0);
            for parameter in [
                gl::PACK_ROW_LENGTH,
                gl::PACK_SKIP_PIXELS,
                gl::PACK_SKIP_ROWS,
            ] {
                gl::PixelStorei(parameter, 0);
            }
            gl::PixelStorei(gl::PACK_ALIGNMENT, 1);
            gl::BindFramebuffer(gl::READ_FRAMEBUFFER, target.fbo);
            gl::ReadBuffer(gl::COLOR_ATTACHMENT0);
            gl::ReadPixels(
                0,
                0,
                width,
                height,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                pixels.as_mut_ptr().cast(),
            );
        }
        check_gl("snapshotting frozen Scene history")?;
        Ok(Self {
            pixels,
            width,
            height,
            _reservation: reservation,
        })
    }
    pub fn restore(&self) -> Result<RenderTarget> {
        let target = RenderTarget::new(self.width, self.height)?;
        // The renderer's GlState guard already disables unpack buffers/strides.
        // History snapshots are GPU-oriented rows and require no flip/copy.
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, target.texture.id);
            gl::TexSubImage2D(
                gl::TEXTURE_2D,
                0,
                0,
                0,
                self.width,
                self.height,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                self.pixels.as_ptr().cast(),
            );
        }
        check_gl("restoring frozen Scene history")?;
        Ok(target)
    }
    pub fn allocation_id(&self) -> usize {
        self.pixels.as_ptr() as usize
    }
    pub fn capacity_bytes(&self) -> usize {
        self.pixels.capacity()
    }
    #[cfg(test)]
    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }
}

#[derive(Debug, PartialEq)]
struct PackState {
    buffer: i32,
    alignment: i32,
    layout: [i32; 3],
    read_fbo: i32,
    read_buffer: i32,
}
impl PackState {
    fn capture() -> Self {
        let mut result = Self {
            buffer: 0,
            alignment: 0,
            layout: [0; 3],
            read_fbo: 0,
            read_buffer: 0,
        };
        unsafe {
            gl::GetIntegerv(gl::PIXEL_PACK_BUFFER_BINDING, &mut result.buffer);
            gl::GetIntegerv(gl::PACK_ALIGNMENT, &mut result.alignment);
            gl::GetIntegerv(gl::READ_FRAMEBUFFER_BINDING, &mut result.read_fbo);
            gl::GetIntegerv(gl::READ_BUFFER, &mut result.read_buffer);
            for (i, parameter) in [
                gl::PACK_ROW_LENGTH,
                gl::PACK_SKIP_PIXELS,
                gl::PACK_SKIP_ROWS,
            ]
            .into_iter()
            .enumerate()
            {
                gl::GetIntegerv(parameter, &mut result.layout[i]);
            }
        }
        result
    }
}
impl Drop for PackState {
    fn drop(&mut self) {
        unsafe {
            gl::BindBuffer(gl::PIXEL_PACK_BUFFER, self.buffer as u32);
            gl::PixelStorei(gl::PACK_ALIGNMENT, self.alignment);
            for (i, parameter) in [
                gl::PACK_ROW_LENGTH,
                gl::PACK_SKIP_PIXELS,
                gl::PACK_SKIP_ROWS,
            ]
            .into_iter()
            .enumerate()
            {
                gl::PixelStorei(parameter, self.layout[i]);
            }
            gl::BindFramebuffer(gl::READ_FRAMEBUFFER, self.read_fbo as u32);
            gl::ReadBuffer(self.read_buffer as u32);
        }
    }
}

// Copyright (c) 2022, BlockProject 3D
//
// All rights reserved.
//
// Redistribution and use in source and binary forms, with or without modification,
// are permitted provided that the following conditions are met:
//
//     * Redistributions of source code must retain the above copyright notice,
//       this list of conditions and the following disclaimer.
//     * Redistributions in binary form must reproduce the above copyright notice,
//       this list of conditions and the following disclaimer in the documentation
//       and/or other materials provided with the distribution.
//     * Neither the name of BlockProject 3D nor the names of its contributors
//       may be used to endorse or promote products derived from this software
//       without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
// "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
// LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
// A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT OWNER OR
// CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL,
// EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO,
// PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR
// PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF
// LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING
// NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS
// SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

use byteorder::{ByteOrder, LittleEndian};
use image::{DynamicImage, GrayAlphaImage, GrayImage, RgbaImage};
use nalgebra::Point2;
use crate::math::{Vec2f, Vec4f};
use crate::template::Format;

#[derive(Copy, Clone)]
pub enum Texel {
    L8(u8),
    LA8(u8, u8),
    RGBA8(u8, u8, u8, u8),
    F32(f32),
    RGBAF32(f32, f32, f32, f32)
}

impl Texel {
    /// Converts this texel to RGBA data (None when texel format is not compatible).
    pub fn rgba(&self) -> Option<(u8, u8, u8, u8)> {
        match self {
            Texel::L8(l) => Some((*l, *l, *l, 255 as _)),
            Texel::LA8(l, a) => Some((*l, *l, *l, *a)),
            Texel::RGBA8(r, g, b, a) => Some((*r, *g, *b, *a)),
            Texel::F32(_) => None,
            Texel::RGBAF32(_, _, _, _) => None
        }
    }

    /// Converts this texel to a floating point vector. When this texel is RGBA or RGBA compatible,
    /// value is normalized assuming a maximum of 255.
    pub fn normalize(&self) -> Vec4f {
        self.rgba().map(|(r, g, b, a)| Vec4f::new(r as _, g as _, b as _, a as _) / 255.0).unwrap_or_else(|| match self {
            Texel::F32(v) => Vec4f::from_element(*v as _),
            Texel::RGBAF32(r, g, b, a) => Vec4f::new(*r as _, *g as _, *b as _, *a as _),
            _ => unsafe { std::hint::unreachable_unchecked() }
        })
    }
}

pub trait Texture {
    /// Gets a texel by position, returns None if the position is out of range.
    fn get(&self, pos: Point2<u32>) -> Option<Texel>;

    /// Gets the texture format.
    fn format(&self) -> Format;

    /// Gets the texture width.
    fn width(&self) -> u32;

    /// Gets the texture height.
    fn height(&self) -> u32;

    /// Samples a texel by nearest position (individual coordinates in the 0-1 range).
    fn sample(&self, pos: Vec2f) -> Option<Texel> {
        let pos = pos.component_mul(&Vec2f::new(self.width() as _, self.height() as _)).map(|v| v as u32);
        self.get(pos.into())
    }
}

pub enum ImageTexture
{
    R8(GrayImage),

    RA8(GrayAlphaImage),

    RGBA8(RgbaImage)
}

impl ImageTexture {
    pub fn new(src: image::DynamicImage) -> ImageTexture {
        match src {
            DynamicImage::ImageLuma8(v) => ImageTexture::R8(v),
            DynamicImage::ImageLumaA8(v) => ImageTexture::RA8(v),
            DynamicImage::ImageRgba8(v) => ImageTexture::RGBA8(v),
            v => ImageTexture::RGBA8(v.to_rgba8())
        }
    }
}

impl Texture for ImageTexture {
    fn get(&self, pos: Point2<u32>) -> Option<Texel> {
        match self {
            ImageTexture::R8(v) => v.get_pixel_checked(pos.x, pos.y)
                .map(|v| Texel::L8(v[0])),
            ImageTexture::RA8(v) => v.get_pixel_checked(pos.x, pos.y)
                .map(|v| Texel::LA8(v[0], v[1])),
            ImageTexture::RGBA8(v) => v.get_pixel_checked(pos.x, pos.y)
                .map(|v| Texel::RGBA8(v[0], v[1], v[2], v[3]))
        }
    }

    fn format(&self) -> Format {
        match self {
            ImageTexture::R8(_) => Format::L8,
            ImageTexture::RA8(_) => Format::LA8,
            ImageTexture::RGBA8(_) => Format::RGBA8
        }
    }

    fn width(&self) -> u32 {
        match self {
            ImageTexture::R8(v) => v.width(),
            ImageTexture::RA8(v) => v.width(),
            ImageTexture::RGBA8(v) => v.width()
        }
    }

    fn height(&self) -> u32 {
        match self {
            ImageTexture::R8(v) => v.height(),
            ImageTexture::RA8(v) => v.height(),
            ImageTexture::RGBA8(v) => v.height()
        }
    }
}

#[derive(Debug)]
pub struct OutputTexture {
    width: u32,
    height: u32,
    format: Format,
    data: Box<[u8]>
}

impl OutputTexture {
    pub fn new(mut width: u32, mut height: u32, format: Format) -> OutputTexture {
        // Enforce texture is a power of two to pre-align on a majority of graphics hardware
        // and avoid bugs on some OpenGL implementations.
        if !width.is_power_of_two() {
            width = width.next_power_of_two();
        }
        if !height.is_power_of_two() {
            height = height.next_power_of_two();
        }
        OutputTexture {
            width,
            height,
            data: vec![0; (width * height * format.texel_size()) as usize].into_boxed_slice(),
            format,
        }
    }

    fn base_offset(&self, x: u32, y: u32) -> Option<u32> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let size = self.format.texel_size();
        Some((y * self.width * size) + (x * size))
    }

    pub fn set(&mut self, pos: Point2<u32>, texel: Texel) -> bool {
        let offset = self.base_offset(pos.x, pos.y)
            .expect("Illegal output render target position");
        match (self.format, texel) {
            (Format::L8, Texel::L8(l)) => {
                self.data[offset as usize] = l;
                true
            },
            (Format::LA8, Texel::LA8(l, a)) => {
                self.data[offset as usize] = l;
                self.data[offset as usize] = a;
                true
            },
            (Format::RGBA8, Texel::RGBA8(r, g, b, a)) => {
                self.data[offset as usize] = r;
                self.data[(offset + 1) as usize] = g;
                self.data[(offset + 2) as usize] = b;
                self.data[(offset + 3) as usize] = a;
                true
            },
            (Format::RGBAF32, Texel::RGBAF32(r, g, b, a)) => {
                LittleEndian::write_f32(&mut self.data[offset as usize..], r);
                LittleEndian::write_f32(&mut self.data[(offset + 4) as usize..], g);
                LittleEndian::write_f32(&mut self.data[(offset + 8) as usize..], b);
                LittleEndian::write_f32(&mut self.data[(offset + 12) as usize..], a);
                true
            },
            (Format::F32, Texel::F32(v)) => {
                LittleEndian::write_f32(&mut self.data[offset as usize..], v);
                true
            },
            (_, _) => false
        }
    }

    fn assume_rgba_compat(&self) -> RgbaImage {
        let mut image = RgbaImage::new(self.width, self.height);
        image.enumerate_pixels_mut().for_each(|(x, y, v)| {
            let (r, g, b, a) = self.get(Point2::new(x, y)).unwrap().rgba().unwrap();
            v[0] = r;
            v[1] = g;
            v[2] = b;
            v[3] = a;
        });
        image
    }

    /// Performs a potentially lossy conversion to an 8 bits RGBA image.
    pub fn to_rgba_lossy(self) -> RgbaImage {
        match self.format {
            Format::L8 => self.assume_rgba_compat(),
            Format::LA8 => self.assume_rgba_compat(),
            Format::RGBA8 => RgbaImage::from_raw(self.width, self.height, self.data.to_vec()).unwrap(),
            Format::RGBAF32 => {
                let mut image = RgbaImage::new(self.width, self.height);
                image.enumerate_pixels_mut().for_each(|(x, y, v)| {
                    let vec = self.get(Point2::new(x, y)).unwrap().normalize().map(|v| v as u8);
                    v[0] = vec.x;
                    v[1] = vec.y;
                    v[2] = vec.z;
                    v[3] = vec.w;
                });
                image
            },
            Format::F32 => RgbaImage::from_raw(self.width, self.height, self.data.to_vec()).unwrap()
        }
    }
}

impl Texture for OutputTexture {
    fn get(&self, pos: Point2<u32>) -> Option<Texel> {
        let offset = self.base_offset(pos.x, pos.y)?;
        Some(match self.format {
            Format::L8 => {
                let l = self.data[offset as usize];
                Texel::L8(l)
            }
            Format::LA8 => {
                let l = self.data[offset as usize];
                let a = self.data[(offset + 1) as usize];
                Texel::LA8(l, a)
            }
            Format::RGBA8 => {
                let r = self.data[offset as usize];
                let g = self.data[(offset + 1) as usize];
                let b = self.data[(offset + 2) as usize];
                let a = self.data[(offset + 3) as usize];
                Texel::RGBA8(r, g, b, a)
            }
            Format::RGBAF32 => {
                let r = &self.data[offset as usize..];
                let g = &self.data[(offset + 4) as usize..];
                let b = &self.data[(offset + 8) as usize..];
                let a = &self.data[(offset + 12) as usize..];
                Texel::RGBAF32(LittleEndian::read_f32(r),
                               LittleEndian::read_f32(g),
                               LittleEndian::read_f32(b),
                               LittleEndian::read_f32(a))
            }
            Format::F32 => {
                let v = &self.data[offset as usize..];
                Texel::F32(LittleEndian::read_f32(v))
            }
        })
    }

    fn format(&self) -> Format {
        self.format
    }

    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }
}

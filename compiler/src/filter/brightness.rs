// Copyright (c) 2023, BlockProject 3D
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

use std::sync::Arc;
use nalgebra::Point2;
use crate::filter::{Filter, FilterError, FrameBuffer, FrameBufferError, Function, New};
use crate::math::{Vec2f, Vec4f};
use crate::params::ParameterMap;
use crate::texture::{Format, OutputTexture, Texel, Texture};
use crate::math::Clamp;

pub struct Func {
    brightness: f64,
    format: Format,
    buffer: Arc<OutputTexture>
}

impl Func {
    pub fn convert(&self, rgba: Vec4f) -> Texel {
        match self.format {
            Format::L8 => Texel::L8((rgba.x * 255.0) as u8),
            Format::LA8 => {
                let la = (Vec2f::new(rgba.x, rgba.w) * 255.0).map(|v| v as u8);
                Texel::LA8(la.x, la.y)
            },
            Format::RGBA8 => {
                let rgba = (rgba * 255.0).map(|v| v as u8);
                Texel::RGBA8(rgba.x, rgba.y, rgba.z, rgba.w)
            },
            Format::RGBAF32 => {
                let rgba = rgba.cast();
                Texel::RGBAF32(rgba.x, rgba.y, rgba.z, rgba.w)
            },
            Format::F32 => Texel::F32(rgba.x as f32)
        }
    }
}

impl Function for Func {
    fn apply(&self, pos: Point2<u32>) -> Texel {
        let mut rgba = unsafe { self.buffer.get(pos).unwrap_unchecked().normalize() };
        let alpha = rgba.w;
        rgba *= self.brightness;
        rgba = rgba.clamp(&Vec4f::zeros(), &Vec4f::new(1.0, 1.0, 1.0, 1.0));
        rgba.w = alpha;
        self.convert(rgba)
    }
}

pub struct Brightness {
    brightness: f64,
    desc: String
}

impl Filter for Brightness {
    type Function = Func;

    fn get_texture_size(&self) -> Option<(u32, u32)> {
        None
    }

    fn get_texture_format(&self) -> Option<Format> {
        None
    }

    fn describe(&self) -> &str {
        &self.desc
    }

    fn new_function(&self, frame_buffer: FrameBuffer) -> Result<Self::Function, FrameBufferError> {
        let previous = frame_buffer.previous.ok_or(FrameBufferError::MissingPrevious)?;
        if frame_buffer.width != previous.width() || frame_buffer.height != previous.height() {
            return Err(FrameBufferError::UnsupportedPreviousSize);
        }
        Ok(Func {
            brightness: self.brightness,
            buffer: previous,
            format: frame_buffer.format
        })
    }
}

impl New for Brightness {
    fn new(params: &ParameterMap) -> Result<Self, FilterError> {
        let brightness = params.get("brightness").map(|v| v.as_float()
            .ok_or(FilterError::InvalidParameter("brightness"))).transpose()?.unwrap_or(1.0);
        Ok(Self {
            brightness,
            desc: format!("Brightness({})", brightness)
        })
    }
}

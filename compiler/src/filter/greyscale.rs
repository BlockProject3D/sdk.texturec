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
use crate::math::{Vec2, Vec2f};
use crate::params::ParameterMap;
use crate::texture::{Format, OutputTexture, Texel, Texture};

pub struct Func {
    buffer: Arc<OutputTexture>,
    is_equal_size: bool,
    size: Vec2f,
    format: Format
}

impl Function for Func {
    fn apply(&self, pos: Point2<u32>) -> Texel {
        let texel = match self.is_equal_size {
            true => self.buffer.get(pos),
            false => self.buffer.sample(pos.coords.cast().component_div(&self.size))
        };
        let (r, g, b, a) = unsafe { texel.unwrap_unchecked().rgba().unwrap_unchecked() };
        let luma = ((0.257 * r as f64 + 0.504 * g as f64 + 0.098 * b as f64) + 16.0).clamp(0.0, 255.0);
        match self.format {
            Format::L8 => Texel::L8(luma as _),
            Format::LA8 => Texel::LA8(luma as _, a),
            _ => std::unreachable!()
        }
    }
}

pub struct Greyscale {
    alpha: bool
}

impl Filter for Greyscale {
    type Function = Func;

    fn get_texture_size(&self) -> Option<(u32, u32)> {
        None
    }

    fn get_texture_format(&self) -> Option<Format> {
        if self.alpha {
            Some(Format::LA8)
        } else {
            Some(Format::L8)
        }
    }

    fn describe(&self) -> &str {
        "Greyscale"
    }

    fn new_function(&self, frame_buffer: FrameBuffer) -> Result<Self::Function, FrameBufferError> {
        let previous = frame_buffer.previous.ok_or(FrameBufferError::MissingPrevious)?;
        if frame_buffer.format != Format::L8 && frame_buffer.format != Format::LA8 {
            return Err(FrameBufferError::UnsupportedFormat);
        }
        if previous.format() != Format::RGBA8 {
            return Err(FrameBufferError::UnsupportedPreviousFormat);
        }
        Ok(Func {
            is_equal_size: frame_buffer.width == previous.width() && frame_buffer.height == previous.height(),
            buffer: previous,
            size: Vec2::from([frame_buffer.width, frame_buffer.height]).cast(),
            format: frame_buffer.format
        })
    }
}

impl New for Greyscale {
    fn new(params: &ParameterMap) -> Result<Self, FilterError> {
        let alpha = params.get("alpha").map(|v| v.as_bool()
            .ok_or(FilterError::InvalidParameter("alpha"))).transpose()?.unwrap_or(false);
        Ok(Greyscale { alpha })
    }
}

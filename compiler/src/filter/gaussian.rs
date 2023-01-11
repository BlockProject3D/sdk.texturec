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
use crate::math::{Clamp, Gaussian2d, Vec2, Vec3, Vec3f};
use crate::params::ParameterMap;
use crate::texture::{Format, OutputTexture, Texel, Texture};

pub struct Func {
    ksize: isize,
    size: Point2<u32>,
    sigma: f64,
    buffer: Arc<OutputTexture>
}

impl Function for Func {
    fn apply(&self, pos: Point2<u32>) -> Texel {
        let mut gsigma = Vec3f::zeros();
        let mut w = 0.0;
        for i in -self.ksize..self.ksize {
            for j in -self.ksize..self.ksize {
                let q = (pos.cast::<isize>() + Vec2::from([j, i]).cast()).clamp(&Point2::new(0, 0), &self.size.cast());
                let norm = (pos.cast() - q).cast::<f64>().norm_squared();
                let kernel = norm.gaussian2d(self.sigma);
                //SAFETY: This is never None because the size of the frame buffer is checked in
                // new_function. The format is also checked to always be compatible with rgba.
                let (r, g, b, _) = unsafe { self.buffer.get(pos).unwrap_unchecked().rgba().unwrap_unchecked() };
                gsigma += Vec3::new(r, g, b).cast() * kernel;
                w += kernel;
            }
        }
        let rgb = (gsigma / w).map(|v| v as u8);
        Texel::RGBA8(rgb.x, rgb.y, rgb.z, 255)
    }
}

pub struct Gaussian {
    sigma: f64,
    ksize: u32,
    desc: String
}

impl Filter for Gaussian {
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
        if previous.format() == Format::RGBAF32 || previous.format() == Format::F32 {
            return Err(FrameBufferError::UnsupportedPreviousFormat);
        }
        if frame_buffer.format == Format::RGBAF32 || frame_buffer.format == Format::F32 {
            return Err(FrameBufferError::UnsupportedFormat);
        }
        Ok(Func {
            buffer: previous,
            size: Point2::new(frame_buffer.width, frame_buffer.height),
            ksize: self.ksize as _,
            sigma: self.sigma
        })
    }
}

impl New for Gaussian {
    fn new(params: &ParameterMap) -> Result<Self, FilterError> {
        let sigma = params.get("sigma").map(|v| v.as_float()
            .ok_or(FilterError::InvalidParameter("sigma"))).transpose()?.unwrap_or(1.5);
        let ksize = params.get("ksize").map(|v| v.as_int()
            .ok_or(FilterError::InvalidParameter("ksize"))).transpose()?.unwrap_or(3);
        Ok(Self {
            sigma,
            ksize: ksize as _,
            desc: format!("Gaussian(𝜎={}, n={})", sigma, ksize)
        })
    }
}

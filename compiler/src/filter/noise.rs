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

use nalgebra::Point2;
use noise::{NoiseFn, Perlin};
use rand::distributions::{Distribution, Standard, Uniform};
use rand::rngs::OsRng;
use crate::filter::{Filter, FilterError, FrameBuffer, FrameBufferError, Function, New};
use crate::math::{Vec2, Vec2f, Vec4};
use crate::params::ParameterMap;
use crate::texture::{Format, Texel};

#[derive(Copy, Clone)]
enum Mode {
    Random,
    Perlin(u32)
}

pub struct Func {
    format: Format,
    mode: Mode,
    size: Vec2f
}

impl Function for Func {
    fn apply(&self, pos: Point2<u32>) -> Texel {
        match self.mode {
            Mode::Random => {
                let mut rng = OsRng::default();
                match self.format {
                    Format::L8 => Texel::L8(Uniform::from(0..=255).sample(&mut rng)),
                    Format::LA8 => {
                        let v = Vec2::from_distribution(&Uniform::from(0..=255), &mut rng);
                        Texel::LA8(v.x, v.y)
                    },
                    Format::RGBA8 => {
                        let v = Vec4::from_distribution(&Uniform::from(0..=255), &mut rng);
                        Texel::RGBA8(v.x, v.y, v.z, v.w)
                    }
                    Format::RGBAF32 => {
                        let v = Vec4::from_distribution(&Standard, &mut rng);
                        Texel::RGBAF32(v.x, v.y, v.z, v.w)
                    }
                    Format::F32 => Texel::F32(Standard.sample(&mut rng))
                }
            },
            Mode::Perlin(seed) => {
                let perlin = Perlin::new(seed);
                let pos = pos.cast::<f64>().coords.component_div(&self.size);
                let z = perlin.get([pos.x * 2.0, pos.y * 2.0]).abs();
                match self.format {
                    Format::L8 => Texel::L8((z * 255.0) as u8),
                    Format::LA8 => Texel::LA8((z * 255.0) as u8, 255),
                    Format::RGBA8 => Texel::RGBA8((z * 255.0) as u8, (z * 255.0) as u8, (z * 255.0) as u8, 255),
                    Format::RGBAF32 => Texel::RGBAF32(z as _, z as _, z as _, 1.0),
                    Format::F32 => Texel::F32(z as _)
                }
            }
        }
    }
}

pub struct Noise {
    desc: String,
    mode: Mode
}

impl Filter for Noise {
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
        Ok(Func {
            format: frame_buffer.format,
            mode: self.mode,
            size: Vec2::new(frame_buffer.width, frame_buffer.height).cast()
        })
    }
}

impl New for Noise {
    fn new(params: &ParameterMap) -> Result<Self, FilterError> {
        let mode = params.get("mode").map(|v| v.as_str()
            .ok_or(FilterError::InvalidParameter("mode"))).transpose()?
            .unwrap_or("random");
        let desc = format!("Noise({})", mode);
        let mode = match mode {
            "random" => Ok(Mode::Random),
            "perlin" => {
                let seed = params.get("seed").map(|v| v.as_int()
                    .ok_or(FilterError::InvalidParameter("seed"))).transpose()?.unwrap_or(0);
                Ok(Mode::Perlin(seed as _))
            },
            _ => Err(FilterError::InvalidParameter("mode"))
        }?;
        Ok(Noise { desc, mode })
    }
}

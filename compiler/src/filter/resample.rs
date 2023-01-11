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
use crate::texture::{Format, ImageTexture, Texel, Texture};

pub struct Func {
    is_eq_size: bool,
    base_texture: Arc<ImageTexture>,
    format: Format,
    size: Vec2f,
}

fn check_format_compatible(inf: Format, outf: Format) -> bool {
    if inf == outf {
        return true;
    }
    match outf {
        Format::L8 => inf == Format::L8 || inf == Format::LA8 || inf == Format::RGBA8,
        Format::LA8 => inf == Format::L8 || inf == Format::LA8 || inf == Format::RGBA8,
        Format::RGBA8 => inf == Format::L8 || inf == Format::LA8 || inf == Format::RGBA8,
        Format::RGBAF32 => inf == Format::RGBAF32,
        Format::F32 => inf ==  Format::F32
    }
}

impl Func {
    pub fn convert(&self, texel: Texel) -> Texel {
        if texel.format() == self.format {
            return texel;
        }
        //SAFETY: This is cannot fail as texel.rgba() returns None only if format is RGBAF32 or F32
        // and we have checked format compatibility in check_format_compatible.
        unsafe {
            match self.format {
                Format::L8 => Texel::L8(texel.rgba().unwrap_unchecked().0),
                Format::LA8 => Texel::LA8(texel.rgba().unwrap_unchecked().0,
                                          texel.rgba().unwrap_unchecked().1),
                Format::RGBA8 => Texel::RGBA8(texel.rgba().unwrap_unchecked().0,
                                              texel.rgba().unwrap_unchecked().1,
                                              texel.rgba().unwrap_unchecked().2,
                                              texel.rgba().unwrap_unchecked().3),
                Format::RGBAF32 => texel,
                Format::F32 => texel
            }
        }
    }
}

impl Function for Func {
    fn apply(&self, pos: Point2<u32>) -> Texel {
        match self.is_eq_size {
            //SAFETY: This is safe because if is_eq_size is true then output size is always
            // equal to input base texture size.
            true => self.convert(unsafe { self.base_texture.get(pos).unwrap_unchecked() }),
            false => {
                //Unfortunately nalgebra has removed to_vector long ago, so implement a workaround.
                let pos = pos.cast::<f64>().coords.component_div(&self.size);
                let texel = self.base_texture.sample(pos).unwrap();
                self.convert(texel)
            }
        }
    }
}

pub struct Resample {
    base_texture: Arc<ImageTexture>
}

impl Filter for Resample {
    type Function = Func;

    fn get_texture_size(&self) -> Option<(u32, u32)> {
        Some((self.base_texture.width(), self.base_texture.height()))
    }

    fn get_texture_format(&self) -> Option<Format> {
        Some(self.base_texture.format())
    }

    fn describe(&self) -> &str {
        "Resample(Nearest)"
    }

    fn new_function(&self, frame_buffer: FrameBuffer) -> Result<Self::Function, FrameBufferError> {
        if !check_format_compatible(self.base_texture.format(), frame_buffer.format) {
            return Err(FrameBufferError::UnsupportedFormat)
        }
        Ok(Func {
            format: frame_buffer.format,
            is_eq_size: (self.base_texture.width(), self.base_texture.height()) == (frame_buffer.width, frame_buffer.height),
            base_texture: self.base_texture.clone(),
            size: Vec2::from([frame_buffer.width, frame_buffer.height]).cast(),
        })
    }
}

impl New for Resample {
    fn new(params: &ParameterMap) -> Result<Self, FilterError> {
        let base_texture = params.get("base")
            .ok_or(FilterError::MissingParameter("base"))?.as_texture()
            .ok_or(FilterError::InvalidParameter("base"))?.clone();
        Ok(Self {
            base_texture
        })
    }
}

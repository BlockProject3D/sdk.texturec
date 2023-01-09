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
use crate::params::ParameterMap;
use crate::texture::{DynamicTexture, Format, Texel};

pub enum FrameBufferError {
    MissingPrevious,
    UnsupportedSize,
    UnsupportedFormat,
    UnsupportedPreviousSize,
    UnsupportedPreviousFormat,
    Other(String)
}

pub enum FilterError {
    MissingParameter(&'static str),
    InvalidParameter(&'static str),
    Other(String)
}

pub struct FrameBuffer {
    previous: Option<Arc<DynamicTexture>>,
    data: Arc<DynamicTexture>,
    width: u32,
    height: u32,
    format: Format
}

pub trait Filter {
    type Function: Function;

    /// Attempts to get the ideal texture size for this filter from the given parameters map.
    /// If this filter has no ideal texture size then return None.
    fn get_texture_size(&self) -> Option<(u32, u32)>;
    fn new_function(&self, frame_buffer: FrameBuffer) -> Result<Self::Function, FrameBufferError>;
}

pub trait New: Filter + Sized {
    fn new(params: &ParameterMap) -> Result<Self, FilterError>;
}

pub trait Function {
    fn apply(&self, pos: Point2<u32>) -> Texel;
}

macro_rules! impl_filter {
    (($d: ty, $df: ty) { $($name: ident),* }) => {
        impl Filter for $d {
            type Function = $df;

            fn get_texture_size(&self) -> Option<(u32, u32)> {
                match self {
                    $(
                        Self::$name(v) => v.get_texture_size(),
                    )*
                    _ => std::unreachable!()
                }
            }

            fn new_function(&self, frame_buffer: FrameBuffer) -> Result<Self::Function, FrameBufferError> {
                match self {
                    $(
                        Self::$name(v) => v.new_function(frame_buffer),
                    )*
                    _ => std::unreachable!()
                }
            }
        }
    };
}

macro_rules! impl_function {
    ($d: ty { $($name: ident),* }) => {
        impl Function for $d {
            fn apply(&self, pos: Point2<u32>) -> Texel {
                match self {
                    $(
                        Self::$name(v) => v.apply(pos),
                    )*
                    _ => std::unreachable!()
                }
            }
        }
    };
}

include!(env!("SRC_FILTER_REGISTRY"));

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

use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::Arc;
use thiserror::Error;
use crate::texture::Texture;

/// Enum for supported texture formats.
#[derive(Deserialize, Debug, Copy, Clone)]
pub enum Format {
    /// 8 bits greyscale (8bpp).
    L8,

    /// 8 bits greyscale with alpha (16bpp).
    LA8,

    /// 8 bits RGBA (32bpp).
    RGBA8,

    /// 32 bits float RGBA (128bpp).
    RGBAF32,

    /// 32 bits float (32bpp).
    F32

    // No support for RGB textures as these are not efficient and some rendering apis do not even
    // support loading those natively (ex DX11, etc).
}

impl Format {
    pub fn texel_size(&self) -> u32 { //Returns the texel size in bytes
        match self {
            Format::L8 => 1,
            Format::LA8 => 2,
            Format::RGBA8 => 4,
            Format::RGBAF32 => 16,
            Format::F32 => 4
        }
    }
}

#[derive(Deserialize, Debug, Copy, Clone)]
pub enum Type {
    Texture,
    Float,
    Bool,
    Int,
    Vector2,
    Vector3,
    Vector4
}

pub type Parameters = HashMap<String, Type>;

#[derive(Deserialize)]
pub struct Template {
    /// Default output texture width.
    pub default_width: u32,

    /// Default output texture height.
    pub default_height: u32,

    /// Base texture parameter to auto-detect the output texture size.
    pub base_texture: Option<String>,

    /// Output texture format.
    pub format: Format,

    /// Mipmap count.
    pub mipmaps: u8,

    /// Template parameters.
    pub parameters: Parameters,

    /// List of lua scripts to run, in order, before saving the output texture BPX.
    pub pipeline: Vec<String>
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("io error: {0}")]
    Io(std::io::Error),
    #[error("parse error: {0}")]
    Toml(toml::de::Error)
}

impl Template {
    pub fn load(path: &Path) -> Result<Template, Error> {
        std::fs::read_to_string(path)
            .map(|v| toml::from_str(&v))
            .map_err(Error::Io)?
            .map_err(Error::Toml)
    }

    pub fn try_width_from_base_texture(&self, params: &crate::params::Parameters) -> Option<u32> {
        params.get(self.base_texture.as_ref()?).map(|v| match v {
            crate::params::Parameter::Texture(tex) => Some(tex.width()),
            _ => None
        })?
    }

    pub fn try_height_from_base_texture(&self, params: &crate::params::Parameters) -> Option<u32> {
        params.get(self.base_texture.as_ref()?).map(|v| match v {
            crate::params::Parameter::Texture(tex) => Some(tex.height()),
            _ => None
        })?
    }

    /// Consumes and loads pipeline scripts.
    pub fn load_scripts(self, base_folder: &Path) -> std::io::Result<Vec<Arc<[u8]>>> {
        let mut res = Vec::new();
        for script_name in self.pipeline {
            let script_path = base_folder.join(script_name + ".lua");
            let mut v = Vec::new();
            File::open(script_path)?.read_to_end(&mut v)?;
            res.push(v.into());
        }
        Ok(res)
    }
}

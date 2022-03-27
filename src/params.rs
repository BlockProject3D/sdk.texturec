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

use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::Path;
use std::sync::Arc;
use bstr::ByteSlice;
use image::io::Reader;
use os_str_bytes::OsStrBytes;
use crate::math::{Vec2f, Vec3f, Vec4f};
use crate::template::{Template, Type};
use log::error;
use crate::texture::ImageTexture;
use thiserror::Error;

/// Image load error.
#[derive(Debug, Error)]
pub enum ImageError {
    #[error("io error: {0}")]
    Io(std::io::Error),
    #[error("decoding error: {0}")]
    Image(image::error::ImageError)
}

/// Parameter initialization error.
#[derive(Debug, Error)]
pub enum Error {
    /// Undeclared parameter.
    #[error("undeclared")]
    Undeclared,

    /// Parameter name is invalid UTF8.
    #[error("illegal bytes")]
    InvalidUtf8,

    /// Parameter format is invalid.
    #[error("bad format")]
    InvalidFormat,

    /// An image parameter failed to load.
    #[error("image error: {0}")]
    Image(ImageError)
}

pub enum Parameter {
    Texture(Arc<ImageTexture>),
    Float(f64),
    Bool(bool),
    Int(i64),
    Vector2(Vec2f),
    Vector3(Vec3f),
    Vector4(Vec4f)
}

pub struct Parameters {
    content: Option<HashMap<String, Parameter>>
}

impl Parameters {
    pub fn parse<'a>(template: &Template, params: Option<impl Iterator<Item = &'a OsStr>>) -> Result<Parameters, Error> {
        let mut content: Option<HashMap<String, Parameter>> = None;
        if params.is_none() {
            return Ok(Parameters { content });
        }
        let params = unsafe { params.unwrap_unchecked() };
        for par in params {
            let bytes = par.to_raw_bytes();
            let pos = bytes.find_byte(b'=').ok_or(Error::InvalidFormat)?;
            let name = std::str::from_utf8(&bytes[..pos]).map_err(|_| Error::InvalidUtf8)?;
            let value = &bytes[pos + 1..];
            match template.parameters.get(name) {
                Some(ty) => {
                    let val = match ty {
                        Type::Texture => {
                            let image = Reader::open(Path::new(&OsStr::from_raw_bytes(value).unwrap()))
                                .map_err(|e| Error::Image(ImageError::Io(e)))?.decode()
                                .map_err(|e| Error::Image(ImageError::Image(e)))?;
                            Parameter::Texture(Arc::new(ImageTexture::new(image)))
                        },
                        Type::Float => Parameter::Float(std::str::from_utf8(value)
                            .map_err(|_| Error::InvalidUtf8)?.parse()
                            .map_err(|_| Error::InvalidFormat)?),
                        Type::Bool => Parameter::Bool(if value == b"true" || value == b"on"
                            || value == b"1" { true } else { false }),
                        Type::Int => Parameter::Int(std::str::from_utf8(value)
                            .map_err(|_| Error::InvalidUtf8)?.parse()
                            .map_err(|_| Error::InvalidFormat)?),
                        Type::Vector2 => {
                            let subval = &value[1..value.len() - 1];
                            let mut val = std::str::from_utf8(subval).map_err(|_| Error::InvalidUtf8)?.split(',');
                            Parameter::Vector2(Vec2f::new(
                                val.next().ok_or(Error::InvalidFormat)?.parse()
                                    .map_err(|_| Error::InvalidFormat)?,
                                val.next().ok_or(Error::InvalidFormat)?.parse()
                                    .map_err(|_| Error::InvalidFormat)?
                            ))
                        },
                        Type::Vector3 => {
                            let subval = &value[1..value.len() - 1];
                            let mut val = std::str::from_utf8(subval).map_err(|_| Error::InvalidUtf8)?.split(',');
                            Parameter::Vector3(Vec3f::new(
                                val.next().ok_or(Error::InvalidFormat)?.parse()
                                    .map_err(|_| Error::InvalidFormat)?,
                                val.next().ok_or(Error::InvalidFormat)?.parse()
                                    .map_err(|_| Error::InvalidFormat)?,
                                val.next().ok_or(Error::InvalidFormat)?.parse()
                                    .map_err(|_| Error::InvalidFormat)?
                            ))
                        }
                        Type::Vector4 => {
                            let subval = &value[1..value.len() - 1];
                            let mut val = std::str::from_utf8(subval).map_err(|_| Error::InvalidUtf8)?.split(',');
                            Parameter::Vector4(Vec4f::new(
                                val.next().ok_or(Error::InvalidFormat)?.parse()
                                    .map_err(|_| Error::InvalidFormat)?,
                                val.next().ok_or(Error::InvalidFormat)?.parse()
                                    .map_err(|_| Error::InvalidFormat)?,
                                val.next().ok_or(Error::InvalidFormat)?.parse()
                                    .map_err(|_| Error::InvalidFormat)?,
                                val.next().ok_or(Error::InvalidFormat)?.parse()
                                    .map_err(|_| Error::InvalidFormat)?
                            ))
                        }
                    };
                    content.get_or_insert_with(Default::default).insert(name.into(), val);
                },
                None => {
                    error!("Undeclared parameter '{}'", name);
                    return Err(Error::Undeclared)
                }
            }
        }
        Ok(Parameters { content })
    }

    pub fn get(&self, name: &str) -> Option<&Parameter> {
        self.content.as_ref()?.get(name)
    }
}

pub type SharedParameters = Arc<Parameters>;

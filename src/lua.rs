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

use std::sync::Arc;
use bp3d_lua::LuaEngine;
use bp3d_lua::number::{Checked, Int, NumToLua};
use bp3d_lua::vector::{LuaVec2, LuaVec3, LuaVec4};
use nalgebra::{Point2, Vector4};
use rlua::{Context, Error, FromLua, FromLuaMulti, Table, ToLua, ToLuaMulti, UserData, UserDataMethods, Value};
use rlua::prelude::{LuaMultiValue, LuaString};
use crate::math::Vec4f;
use crate::params::{Parameter, SharedParameters};
use crate::template::Format;
use crate::texture::{Texel, Texture};

pub const GLOBAL_PARAMETERS: &str = "Parameters";
pub const GLOBAL_PREVIOUS: &str = "Previous";
pub const GLOBAL_BUFFER: &str = "Buffer";
pub const BUFFER_FORMAT: &str = "format";
pub const BUFFER_WIDTH: &str = "width";
pub const BUFFER_HEIGHT: &str = "height";

impl<'lua> ToLua<'lua> for Format {
    fn to_lua(self, lua: Context<'lua>) -> rlua::Result<Value<'lua>> {
        match self {
            Format::L8 => Int(0).to_lua(lua),
            Format::LA8 => Int(1).to_lua(lua),
            Format::RGBA8 => Int(2).to_lua(lua),
            Format::RGBAF32 => Int(3).to_lua(lua),
            Format::F32 => Int(4).to_lua(lua)
        }
    }
}

impl<'lua> FromLua<'lua> for Format {
    fn from_lua(lua_value: Value<'lua>, lua: Context<'lua>) -> rlua::Result<Self> {
        let v: Checked<i32> = Checked::from_lua(lua_value, lua)?;
        match v.0 {
            0 => Ok(Format::L8),
            1 => Ok(Format::LA8),
            2 => Ok(Format::RGBA8),
            3 => Ok(Format::RGBAF32),
            4 => Ok(Format::F32),
            _ => Err(Error::FromLuaConversionError {
                from: "i32",
                to: "Format",
                message: Some("invalid format enum".to_string())
            })
        }
    }
}

#[derive(Copy, Clone)]
pub struct LuaTexel(Texel);

impl UserData for LuaTexel {
    fn add_methods<'lua, T: UserDataMethods<'lua, Self>>(methods: &mut T) {
        methods.add_method("rgba", |ctx, this, ()| {
            match this.0.rgba() {
                Some((r, g, b, a)) => (Checked(r), Checked(g), Checked(b), Checked(a)).to_lua_multi(ctx),
                None => Ok(LuaMultiValue::new())
            }
        });
        methods.add_method("normalize", |_, this, ()| Ok(LuaVec4::from(this.0.normalize())));
    }
}

pub struct LuaTexture<T>(Arc<T>);

impl<T> LuaTexture<T> {
    pub fn new(inner: Arc<T>) -> LuaTexture<T> {
        Self(inner)
    }
}

impl<T> UserData for LuaTexture<T> where T: Texture {
    fn add_methods<'lua, T1: UserDataMethods<'lua, Self>>(methods: &mut T1) {
        methods.add_method("width", |_, this, ()| Ok(Checked(this.0.width())));
        methods.add_method("height", |_, this, ()| Ok(Checked(this.0.height())));
        methods.add_method("format", |_, this, ()| Ok(this.0.format()));
        methods.add_method("get", |_, this, (x, y): (Checked<u32>, Checked<u32>)| Ok(this.0.get(Point2::new(x.0, y.0)).map(|v| LuaTexel(v))));
        methods.add_method("sample", |_, this, pos: LuaVec2<f64>| Ok(this.0.sample(pos.into()).map(|v| LuaTexel(v))));
    }
}

fn ensure_value_count(actual: usize, expected: &[usize]) -> rlua::Result<usize> {
    let mut count = None;
    for v in expected {
        if actual == *v {
            count = Some(*v);
            break;
        }
    }
    count.ok_or_else(|| Error::FromLuaConversionError {
        from: "?",
        to: "Texel",
        message: Some(format!("expected {:#?} value(s) got {} value(s)", expected, actual))
    })
}

#[derive(Copy, Clone)]
pub struct LuaOutTexel(Texel);

impl LuaOutTexel {
    pub fn into_inner(self) -> Texel {
        self.0
    }
}

impl<'lua> FromLuaMulti<'lua> for LuaOutTexel {
    fn from_lua_multi(values: LuaMultiValue<'lua>, lua: Context<'lua>) -> rlua::Result<Self> {
        let table: Table = lua.globals().get(GLOBAL_BUFFER)?;
        let format: Format = table.raw_get(BUFFER_FORMAT)?;
        Ok(LuaOutTexel(match format {
            Format::L8 => {
                ensure_value_count(values.len(), &[1])?;
                let luma: Checked<u8> = Checked::from_lua(values.into_iter().last().unwrap(), lua)?;
                Texel::L8(luma.0)
            },
            Format::LA8 => {
                ensure_value_count(values.len(), &[2])?;
                let mut iter = values.into_iter();
                let luma: Checked<u8> = Checked::from_lua(iter.next().unwrap(), lua)?;
                let alpha: Checked<u8> = Checked::from_lua(iter.next().unwrap(), lua)?;
                Texel::LA8(luma.0, alpha.0)
            },
            Format::RGBA8 => {
                let size = ensure_value_count(values.len(), &[1, 3, 4])?;
                let mut iter = values.into_iter();
                match size {
                    1 => {
                        // Maybe be a Vec3 normalized RGB or a Vec4 normalized RGBA or it's an error.
                        let v = iter.next().unwrap();
                        let vec: Vec4f = LuaVec4::from_lua(v.clone(), lua)
                            .map(|v| v.into_inner())
                            .or_else(|_| LuaVec3::from_lua(v, lua)
                                .map(|v| v.into_inner().push(1.0)))?;
                        let denormalized = (vec * 255.0).map(|v| v as u8);
                        Texel::RGBA8(denormalized.x, denormalized.y, denormalized.z, denormalized.w)
                    },
                    3 => {
                        // Must be RGB u8 or error.
                        let r: Checked<u8> = Checked::from_lua(iter.next().unwrap(), lua)?;
                        let g: Checked<u8> = Checked::from_lua(iter.next().unwrap(), lua)?;
                        let b: Checked<u8> = Checked::from_lua(iter.next().unwrap(), lua)?;
                        Texel::RGBA8(r.0, g.0, b.0, 255)
                    },
                    4 => {
                        // Must be RGBA u8 or error.
                        let r: Checked<u8> = Checked::from_lua(iter.next().unwrap(), lua)?;
                        let g: Checked<u8> = Checked::from_lua(iter.next().unwrap(), lua)?;
                        let b: Checked<u8> = Checked::from_lua(iter.next().unwrap(), lua)?;
                        let a: Checked<u8> = Checked::from_lua(iter.next().unwrap(), lua)?;
                        Texel::RGBA8(r.0, g.0, b.0, a.0)
                    },
                    _ => unreachable!()
                }
            },
            Format::RGBAF32 => {
                let size = ensure_value_count(values.len(), &[1, 4])?;
                let mut iter = values.into_iter();
                match size {
                    1 => {
                        // Must be a Vec4.
                        let v = iter.next().unwrap();
                        let vec: Vector4<f32> = LuaVec4::from_lua(v, lua)?.into_inner();
                        Texel::RGBAF32(vec.x, vec.y, vec.z, vec.w)
                    },
                    4 => {
                        // Must be RGBA f32.
                        let r: Checked<f32> = Checked::from_lua(iter.next().unwrap(), lua)?;
                        let g: Checked<f32> = Checked::from_lua(iter.next().unwrap(), lua)?;
                        let b: Checked<f32> = Checked::from_lua(iter.next().unwrap(), lua)?;
                        let a: Checked<f32> = Checked::from_lua(iter.next().unwrap(), lua)?;
                        Texel::RGBAF32(r.0, g.0, b.0, a.0)
                    },
                    _ => unreachable!()
                }
            },
            Format::F32 => {
                ensure_value_count(values.len(), &[1])?;
                let val: Checked<f32> = Checked::from_lua(values.into_iter().last().unwrap(), lua)?;
                Texel::F32(val.0)
            }
        }))
    }
}

pub struct LuaParameters(SharedParameters);

impl LuaParameters {
    pub fn new(inner: SharedParameters) -> LuaParameters {
        Self(inner)
    }
}

impl UserData for LuaParameters {
    fn add_methods<'lua, T: UserDataMethods<'lua, Self>>(methods: &mut T) {
        methods.add_method("get", |ctx, this, name: LuaString| {
            let name = name.to_str()?;
            this.0.get(name).map(|v| match v {
                Parameter::Texture(a) => LuaTexture(a.clone()).to_lua(ctx),
                Parameter::Float(v) => Ok(v.num_to_lua()),
                Parameter::Bool(v) => v.to_lua(ctx),
                Parameter::Int(v) => Ok(v.num_to_lua()),
                Parameter::Vector2(v) => LuaVec2::from(*v).to_lua(ctx),
                Parameter::Vector3(v) => LuaVec3::from(*v).to_lua(ctx),
                Parameter::Vector4(v) => LuaVec4::from(*v).to_lua(ctx)
            }).transpose()
        })
    }
}

pub trait Lib {
    fn load_format(&self) -> rlua::Result<()>;
}

impl Lib for LuaEngine {
    fn load_format(&self) -> rlua::Result<()> {
        self.create_library("format", false, |ctx| {
            ctx.constant("L8", Format::L8)?;
            ctx.constant("LA8", Format::LA8)?;
            ctx.constant("RGBA8", Format::RGBA8)?;
            ctx.constant("RGBAF32", Format::RGBAF32)?;
            ctx.constant("F32", Format::F32)?;
            Ok(())
        })
    }
}

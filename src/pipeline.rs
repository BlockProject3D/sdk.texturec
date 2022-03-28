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
use std::sync::atomic::{AtomicU32, Ordering};
use bp3d_lua::LuaEngine;
use bp3d_lua::math::LibMath;
use bp3d_lua::number::Checked;
use bp3d_lua::vector::LibVector;
use bp3d_threads::{ThreadPool, UnscopedThreadManager};
use log::{info, warn};
use nalgebra::Point2;
use rlua::Function;
use crate::lua::{GLOBAL_BUFFER, BUFFER_FORMAT, BUFFER_WIDTH, BUFFER_HEIGHT, GLOBAL_PREVIOUS, GLOBAL_PARAMETERS, Lib, LuaOutTexel, LuaParameters, LuaTexture};
use crate::params::{Parameters, SharedParameters};
use crate::SwapChain;
use crate::texture::{OutputTexture, Texel};

const DISPLAY_INTERVAL: u32 = 8192;

pub struct Pipeline {
    scripts: Vec<Arc<[u8]>>,
    cur_pass: usize,
    parameters: SharedParameters,
    swap_chain: SwapChain,
    n_threads: usize
}

static PROCESSED_TEXELS: AtomicU32 = AtomicU32::new(0);

impl Pipeline {
    pub fn new(scripts: Vec<Arc<[u8]>>, parameters: Parameters, swap_chain: SwapChain, n_threads: usize) -> Pipeline {
        Pipeline {
            scripts,
            cur_pass: 0,
            parameters: Arc::new(parameters),
            swap_chain,
            n_threads
        }
    }

    pub fn next_pass(&mut self) -> rlua::Result<()> {
        assert!(self.cur_pass < self.scripts.len()); //Make sure we're not gonna jump into a
        // non-existent pass
        let previous = if self.cur_pass == 0 { None } else { Some(self.swap_chain.next()) }.map(Arc::new);
        let mut render_target = self.swap_chain.next();
        let mut pool: ThreadPool<UnscopedThreadManager, rlua::Result<(Point2<u32>, Texel)>> = ThreadPool::new(self.n_threads);
        let manager = UnscopedThreadManager::new();
        info!("Initialized thread pool with {} max thread(s)", self.n_threads);
        //At this point we don't yet have threads so use relaxed ordering.
        PROCESSED_TEXELS.store(0, Ordering::Relaxed);
        let total = self.swap_chain.height() * self.swap_chain.width();
        for y in 0..self.swap_chain.height() {
            for x in 0..self.swap_chain.width() {
                let script_code = self.scripts[self.cur_pass].clone();
                let previous = previous.clone();
                let parameters = self.parameters.clone();
                let format = self.swap_chain.format();
                let width = self.swap_chain.width();
                let height = self.swap_chain.height();
                pool.send(&manager, move |_| {
                    let engine = LuaEngine::new()?;
                    engine.load_format()?;
                    engine.load_math()?;
                    engine.load_vec2()?;
                    engine.load_vec3()?;
                    engine.load_vec4()?;
                    if let Some(prev) = previous {
                        engine.context(|ctx| ctx.globals().set(GLOBAL_PREVIOUS, LuaTexture::new(prev)))?;
                    }
                    engine.context(|ctx| {
                        let globals = ctx.globals();
                        let table = ctx.create_table()?;
                        table.raw_set(BUFFER_FORMAT, format)?;
                        table.raw_set(BUFFER_WIDTH, Checked(width))?;
                        table.raw_set(BUFFER_HEIGHT, Checked(height))?;
                        globals.set(GLOBAL_BUFFER, table)?;
                        globals.set(GLOBAL_PARAMETERS, LuaParameters::new(parameters))?;
                        ctx.load(&script_code).exec()
                    })?;
                    let res = engine.context(|ctx| {
                        let main: Function = ctx.globals().get("main")?;
                        main.call((x, y))
                    }).map(|v: LuaOutTexel| v.into_inner()).map(|v| (Point2::new(x, y), v));
                    match res {
                        Ok(v) => {
                            let current = PROCESSED_TEXELS.fetch_add(1, Ordering::Relaxed);
                            if current % DISPLAY_INTERVAL == 0 {
                                info!("{:.2}% done...", (current as f64 / total as f64) * 100.0);
                            }
                            Ok(v)
                        },
                        Err(e) => {
                            warn!("script error: {}", e);
                            Err(e)
                        }
                    }
                });
            }
        }
        for task in pool.reduce().map(|v| v.unwrap()) {
            let (pos, texel) = task?;
            if !render_target.set(pos, texel) {
                warn!("Ignored texel at position {} due to format mismatch (expected format '{:?}')", pos, self.swap_chain.format());
            }
        }
        self.cur_pass += 1;
        self.swap_chain.put_back(render_target);
        if let Some(prev) = previous {
            self.swap_chain.put_back(Arc::try_unwrap(prev)
                .expect("ThreadPool termination failure"));
        }
        Ok(())
    }

    /// Finishes this pipeline and return the final output render target.
    pub fn finish(mut self) -> OutputTexture {
        assert!(self.cur_pass > 0); // If we're still at render pass 0 that means the pipeline
        // never ran, and, as such, is not safe to be finished.
        self.swap_chain.next();
        self.swap_chain.next()
    }
}

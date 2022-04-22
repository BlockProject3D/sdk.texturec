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

use std::io::Write;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use bp3d_lua::LuaEngine;
use bp3d_lua::math::LibMath;
use bp3d_lua::number::Checked;
use bp3d_lua::vector::LibVector;
use bp3d_threads::{ThreadPool, UnscopedThreadManager};
use bp3d_tracing::DisableStdoutLogger;
use crossbeam::queue::ArrayQueue;
use tracing::{info, span, warn, Level, instrument};
use nalgebra::Point2;
use rlua::Function;
use crate::lua::{GLOBAL_BUFFER, BUFFER_FORMAT, BUFFER_WIDTH, BUFFER_HEIGHT, GLOBAL_PREVIOUS, GLOBAL_PARAMETERS, Lib, LuaOutTexel, LuaParameters, LuaTexture};
use crate::params::{Parameters, SharedParameters};
use crate::SwapChain;
use crate::template::Format;
use crate::texture::{OutputTexture, Texel};

const DISPLAY_INTERVAL: u32 = 2;

fn print_progress(progress: f64) {
    let useless = std::io::stdout();
    let mut lock = useless.lock();
    write!(lock, "\r{:.2}% done...", progress).unwrap();
    lock.flush().unwrap();
}

struct Task {
    script_code: Arc<[u8]>,
    previous: Option<Arc<OutputTexture>>,
    parameters: SharedParameters,
    format: Format,
    width: u32,
    height: u32,
    vms: Arc<ArrayQueue<LuaEngine>>
}

impl Task {
    fn init_lua_engine(self) -> rlua::Result<(Arc<ArrayQueue<LuaEngine>>, LuaEngine)> {
        match self.vms.pop() {
            Some(v) => Ok((self.vms, v)),
            None => {
                let _span = span!(Level::TRACE, "LuaEngine_new").entered();
                let engine = LuaEngine::new()?;
                engine.load_format()?;
                engine.load_math()?;
                engine.load_vec2()?;
                engine.load_vec3()?;
                engine.load_vec4()?;
                if let Some(prev) = self.previous {
                    engine.context(|ctx| ctx.globals().set(GLOBAL_PREVIOUS, LuaTexture::new(prev)))?;
                }
                engine.context(|ctx| {
                    let globals = ctx.globals();
                    let table = ctx.create_table()?;
                    table.raw_set(BUFFER_FORMAT, self.format)?;
                    table.raw_set(BUFFER_WIDTH, Checked(self.width))?;
                    table.raw_set(BUFFER_HEIGHT, Checked(self.height))?;
                    globals.set(GLOBAL_BUFFER, table)?;
                    globals.set(GLOBAL_PARAMETERS, LuaParameters::new(self.parameters))?;
                    ctx.load(&self.script_code).exec()
                })?;
                Ok((self.vms, engine))
            }
        }
    }

    #[instrument(level = "trace", skip(self, total, intty))]
    fn run(self, x: u32, y: u32, total: f64, intty: bool) -> rlua::Result<(Point2<u32>, Texel)> {
        let (vms, engine) = self.init_lua_engine()?;
        let res = engine.context(|ctx| {
            let _span = span!(Level::TRACE, "Lua_main").entered();
            let main: Function = ctx.globals().get("main")?;
            main.call((x, y))
        }).map(|v: LuaOutTexel| v.into_inner()).map(|v| (Point2::new(x, y), v));
        vms.push(engine).ok().unwrap();
        match res {
            Ok(v) => {
                let current = PROCESSED_TEXELS.fetch_add(1, Ordering::Relaxed);
                if intty && current % DISPLAY_INTERVAL == 0 {
                    print_progress((current as f64 / total as f64) * 100.0);
                }
                Ok(v)
            },
            Err(e) => {
                warn!("script error: {}", e);
                Err(e)
            }
        }
    }
}

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
        let _span = span!(Level::DEBUG, "Next render pass", render_pass = self.cur_pass).entered();
        let mut render_target = self.swap_chain.next();
        let previous = if self.cur_pass == 0 { None } else { Some(self.swap_chain.next()) }.map(Arc::new);
        let mut pool: ThreadPool<UnscopedThreadManager, rlua::Result<(Point2<u32>, Texel)>> = ThreadPool::new(self.n_threads);
        let manager = UnscopedThreadManager::new();
        info!(max_threads = self.n_threads, "Initialized thread pool");
        //At this point we don't yet have threads so use relaxed ordering.
        PROCESSED_TEXELS.store(0, Ordering::Relaxed);
        {
            let total = self.swap_chain.height() * self.swap_chain.width();
            let vms = Arc::new(ArrayQueue::new(self.n_threads));
            let intty = atty::is(atty::Stream::Stdout);
            let _guard = match intty {
                true => {
                    let guard = DisableStdoutLogger::new();
                    print!("0% done...");
                    Some(guard)
                },
                false => None
            };
            for y in 0..self.swap_chain.height() {
                for x in 0..self.swap_chain.width() {
                    let task = Task {
                        script_code: self.scripts[self.cur_pass].clone(),
                        previous: previous.clone(),
                        parameters: self.parameters.clone(),
                        format: self.swap_chain.format(),
                        width: self.swap_chain.width(),
                        height: self.swap_chain.height(),
                        vms: vms.clone()
                    };
                    pool.send(&manager, move |_| task.run(x, y, total as _, intty));
                }
            }
            for task in pool.reduce().map(|v| v.unwrap()) {
                let (pos, texel) = task?;
                if !render_target.set(pos, texel) {
                    warn!(?pos, expected_format = ?self.swap_chain.format(), "Ignored texel due to format mismatch");
                }
            }
            if intty {
                println!()
            }
        }
        self.cur_pass += 1;
        if let Some(prev) = previous {
            self.swap_chain.put_back(Arc::try_unwrap(prev)
                .expect("ThreadPool termination failure"));
        }
        self.swap_chain.put_back(render_target);
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

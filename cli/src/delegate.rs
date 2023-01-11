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
use std::sync::atomic::{AtomicUsize, Ordering};
use bp3d_tracing::DisableStdoutLogger;
use bp3d_texturec::{Delegate, PassDelegate};

const DISPLAY_INTERVAL: usize = 2;

fn print_progress(progress: f64) {
    let useless = std::io::stdout();
    let mut lock = useless.lock();
    write!(lock, "\r{:.2}% done...", progress).unwrap();
    lock.flush().unwrap();
}

pub struct TtyPass {
    _guard: DisableStdoutLogger,
    total_texels: usize,
    cur_texels: AtomicUsize,
}

impl TtyPass {
    pub fn new(total_texels: usize) -> TtyPass {
        let guard = DisableStdoutLogger::new();
        print!("0% done...");
        TtyPass {
            _guard: guard,
            total_texels,
            cur_texels: AtomicUsize::new(0)
        }
    }
}

impl Drop for TtyPass {
    fn drop(&mut self) {
        println!()
    }
}

impl PassDelegate for TtyPass {
    fn on_start_texel(&self, _: u32, _: u32) {}

    fn on_end_texel(&self) {
        let current = self.cur_texels.fetch_add(1, Ordering::Relaxed);
        if current % DISPLAY_INTERVAL == 0 {
            print_progress((current as f64 / self.total_texels as f64) * 100.0);
        }
    }
}

pub struct TtyDelegate;

impl Delegate for TtyDelegate {
    type Pass = TtyPass;

    fn on_start_render_pass(&mut self, _: usize, total_texels: usize) -> Self::Pass {
        TtyPass::new(total_texels)
    }
}

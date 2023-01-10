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

use crate::texture::{Format, OutputTexture};

const SWAP_CHAIN_LEN: usize = 2;

pub struct SwapChain {
    chain: [Option<OutputTexture>; SWAP_CHAIN_LEN],
    index: usize,
    width: u32,
    height: u32,
    format: Format,
}

impl SwapChain {
    pub fn new(mut width: u32, mut height: u32, format: Format) -> SwapChain {
        // Enforce texture is a power of two to pre-align on a majority of graphics hardware
        // and avoid bugs on some OpenGL implementations.
        if !width.is_power_of_two() {
            width = width.next_power_of_two();
        }
        if !height.is_power_of_two() {
            height = height.next_power_of_two();
        }
        SwapChain {
            chain: [None, None],
            index: 0,
            width,
            height,
            format,
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn format(&self) -> Format {
        self.format
    }

    /// Extracts the next texture.
    pub fn next(&mut self) -> OutputTexture {
        //If the swap chain reached the end move back to begining.
        if self.index >= SWAP_CHAIN_LEN {
            self.index = 0;
        }
        let texture = self.chain[self.index]
            .take()
            .unwrap_or_else(|| OutputTexture::new(self.width, self.height, self.format));
        self.index += 1;
        texture
    }

    /// Puts an used texture back into the chain.
    pub fn put_back(&mut self, texture: OutputTexture) {
        if self.index >= SWAP_CHAIN_LEN {
            self.index = 0;
        }
        self.chain[self.index] = Some(texture);
        self.index += 1;
    }
}

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

use image::ImageError;
use thiserror::Error;
use tracing::{debug, info};

//mod lua;
mod math;
pub mod params;
mod pipeline;
mod swapchain;
//mod template;
pub mod texture;
mod filter;

const DEFAULT_WIDTH: u32 = 256;
const DEFAULT_HEIGHT: u32 = 256;

pub use pipeline::ProgressDelegate as Delegate;

#[derive(Debug, Error)]
pub enum AddFilterError<'a> {
    #[error("parameter error: {0}")]
    Parameters(params::Error),
    #[error("unknown filter name: {0}")]
    Unknown(&'a str),
    #[error("filter error: {0}")]
    Filter(filter::FilterError)
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("frame buffer error: {0}")]
    FrameBuffer(filter::FrameBufferError),
    #[error("image error: {0}")]
    Image(ImageError)
}

pub struct Config<'a> {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub format: Option<texture::Format>,
    pub debug: bool,
    pub n_threads: usize,
    pub output: &'a std::path::Path
}

pub struct Compiler<'a, D> {
    config: Config<'a>,
    filters: Vec<filter::DynamicFilter>,
    delegate: Option<D>
}

impl<'a> Compiler<'a, pipeline::NullDelegate> {
    pub fn new(config: Config<'a>) -> Compiler<'a, pipeline::NullDelegate> {
        Compiler {
            config,
            filters: Vec::new(),
            delegate: None
        }
    }
}

impl<'a, D: Delegate> Compiler<'a, D> {
    pub fn with_delegate(config: Config<'a>, delegate: D) -> Compiler<'a, D> {
        Compiler {
            config,
            filters: Vec::new(),
            delegate: Some(delegate)
        }
    }

    pub fn add_filter<'b>(&mut self, name: &'b str, params: Option<impl Iterator<Item = (&'b str, &'b std::ffi::OsStr)>>) -> Result<(), AddFilterError<'b>> {
        let params = params::ParameterMap::parse(params).map_err(AddFilterError::Parameters)?;
        let filter = filter::DynamicFilter::from_name(&params, name)
            .ok_or(AddFilterError::Unknown(name))?.map_err(AddFilterError::Filter)?;
        self.filters.push(filter);
        Ok(())
    }

    pub fn run(self) -> Result<(), Error> {
        use filter::Filter;
        let mut width = self.config.width;
        let mut height = self.config.height;
        let mut format = self.config.format;
        info!(width, height, ?format, "Creating new swap chain...");
        if width.is_none() || height.is_none() || format.is_none() {
            for f in &self.filters {
                if let Some((w, h)) = f.get_texture_size() {
                    if width.is_none() {
                        width = Some(w);
                    }
                    if height.is_none() {
                        height = Some(h);
                    }
                }
                if format.is_none() {
                    if let Some(f) = f.get_texture_format() {
                        format = Some(f)
                    }
                }
            }
        }
        let chain = swapchain::SwapChain::new(
            width.unwrap_or(DEFAULT_WIDTH),
            height.unwrap_or(DEFAULT_HEIGHT),
            format.unwrap_or(texture::Format::RGBA8)
        );
        debug!(width = chain.width(), height = chain.height(), format = ?chain.format(), "Created new swap chain");
        let pass_count = self.filters.len();
        let mut pipeline = pipeline::Pipeline::new(self.filters, chain, self.config.n_threads, self.delegate);
        for _ in 0..pass_count {
            pipeline.next_pass().map_err(Error::FrameBuffer)?;
        }
        let render_target = pipeline.finish();
        if self.config.debug {
            info!("Writing debug output image...");
            render_target.to_rgba_lossy().save("debug.png").map_err(Error::Image)?;
        }
        //TODO: Mipmaps
        //TODO: Actual BPX save
        Ok(())
    }
}

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

use clap::{Arg, Command};
use std::path::Path;
//use log::{info, LevelFilter};
use crate::swapchain::SwapChain;
use tracing::{debug, info};

mod lua;
mod math;
mod params;
mod pipeline;
mod swapchain;
mod template;
mod texture;

const PROG_NAME: &str = env!("CARGO_PKG_NAME");
const PROG_VERSION: &str = env!("CARGO_PKG_VERSION");

macro_rules! etry {
    (($msg: literal $status: literal) => $code: expr) => {
        match $code {
            Ok(v) => v,
            Err(e) => {
                eprintln!("{}: {}", $msg, e);
                return $status;
            }
        }
    };
}

fn run() -> i32 {
    let matches = Command::new(PROG_NAME)
        .author("BlockProject 3D")
        .about("BlockProject 3D SDK - Shader Compiler")
        .version(PROG_VERSION)
        .args([
            Arg::new("debug").short('d').long("debug")
                .help("Enable debug PNG output"),
            Arg::new("template").short('t').long("--template").allow_invalid_utf8(true).takes_value(true).required(true)
                .help("Specify the texture template"),
            Arg::new("output").short('o').long("output").takes_value(true)
                .allow_invalid_utf8(true).help("Output texture file name"),
            Arg::new("threads").short('n').long("threads").takes_value(true)
                .help("Specify the maximum number of threads to use when processing shaders"),
            Arg::new("width").long("width").takes_value(true)
                .help("Override output texture width"),
            Arg::new("height").long("height").takes_value(true)
                .help("Override output texture height"),
            Arg::new("parameter").short('p').long("parameter").takes_value(true).multiple_occurrences(true).allow_invalid_utf8(true)
                .help("Specify a template parameter using the syntax <parameter name>=<parameter value>")
        ]).get_matches();
    let template_path = matches.value_of_os("template").map(Path::new).unwrap();
    info!("Loading template {:?}...", template_path);
    let template = etry!(("failed to load template" 1) =>
        template::Template::load(template_path));
    let params = etry!(("failed to parse parameters" 1) =>
        params::Parameters::parse(&template, matches.values_of_os("parameter")));
    let width: u32 = matches
        .value_of_t("width")
        .or_else(|_| template.try_width_from_base_texture(&params).ok_or(()))
        .unwrap_or(template.default_width);
    let height: u32 = matches
        .value_of_t("height")
        .or_else(|_| template.try_height_from_base_texture(&params).ok_or(()))
        .unwrap_or(template.default_height);
    info!(width, height, format = ?template.format, "Creating new swap chain...");
    let chain = SwapChain::new(width, height, template.format);
    debug!(width = chain.width(), height = chain.height(), format = ?chain.format(), "Created new swap chain");
    info!("Loading scripts...");
    let scripts = etry!(("failed to load pipeline scripts" 1) =>
        template.load_scripts(template_path.parent().unwrap_or(Path::new("."))));
    let pass_count = scripts.len();
    let mut pipeline = pipeline::Pipeline::new(
        scripts,
        params,
        chain,
        matches.value_of_t("threads").unwrap_or(1),
    );
    for _ in 0..pass_count {
        etry!(("failed to run pass" 1) => pipeline.next_pass());
    }
    let render_target = pipeline.finish();
    if matches.is_present("debug") {
        info!("Writing debug output image...");
        etry!(("failed to save debug image" 1) => render_target.to_rgba_lossy().save("debug.png"));
    }
    //TODO: Mipmaps
    //TODO: Actual BPX save
    0
}

fn main() {
    let code = {
        bp3d_tracing::setup!("bp3d-sdk");
        run()
    };
    std::process::exit(code);
}

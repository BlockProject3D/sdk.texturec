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

use std::ffi::{OsStr, OsString};
use clap::{Arg, ArgAction, Command, value_parser};
use std::path::Path;
use std::path::PathBuf;
use bp3d_texturec::{Compiler, Config};
//use log::{info, LevelFilter};
//use crate::swapchain::SwapChain;
//use tracing::{debug, info};
//use crate::params::ParameterMap;
use bp3d_texturec::texture::Format;

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

fn main() {
    let matches = Command::new(PROG_NAME)
        .author("BlockProject 3D")
        .about("BlockProject 3D SDK - Shader Compiler")
        .version(PROG_VERSION)
        .args([
            Arg::new("debug").short('d').long("debug")
                .help("Enable debug PNG output"),
            Arg::new("output").short('o').long("output").num_args(1)
                .value_parser(value_parser!(PathBuf)).help("Output texture file name"),
            Arg::new("threads").short('n').long("threads").num_args(1)
                .help("Specify the maximum number of threads to use when processing shaders"),
            Arg::new("format").short('f').long("format")
                .value_parser(["l8", "la8", "rgba8", "rgba32", "f32"]).num_args(1)
                .help("Override output texture format"),
            Arg::new("width").long("width").value_parser(value_parser!(u32))
                .num_args(1).help("Override output texture width"),
            Arg::new("height").long("height").value_parser(value_parser!(u32))
                .num_args(1).help("Override output texture height"),
            Arg::new("filter").long("filter").short('t').num_args(1)
                .action(ArgAction::Append).help("Adds a filter to apply").required(true),
            Arg::new("parameter").short('p').long("parameter").action(ArgAction::Append)
                .num_args(2).value_parser(value_parser!(OsString))
                .help("Specify a template parameter using the syntax <parameter name> <parameter value>")
        ]).get_matches();
    let output: &Path = matches.get_one::<PathBuf>("output").map(|v| &**v).unwrap_or(Path::new("a.out.bpx"));
    let filters = matches.get_many::<String>("filter").unwrap().map(|v| &**v);
    let fuckingrust = matches.get_many::<OsString>("parameter")
        .map(|v| v.map(|v| &**v).collect::<Vec<&OsStr>>());
    let params = fuckingrust.as_deref().map(|v| v.chunks_exact(2).map(|v| {
        match v[0].to_str() {
            Some(k) => (k, &*v[1]),
            None => {
                eprintln!("One ore more parameters have non-UTF8 characters in the name");
                std::process::exit(1);
            }
        }
    }));
    let format = matches.get_one::<String>("format").map(|v| match &**v {
        "l8" => Format::L8,
        "la8" => Format::LA8,
        "rgba8" => Format::RGBA8,
        "rgba32" => Format::RGBAF32,
        "f32" => Format::F32,
        _ => unreachable!()
    });
    let width: Option<u32> = matches.get_one("width").map(|v| *v);
    let height: Option<u32> = matches.get_one("height").map(|v| *v);
    let n_threads: usize = matches.get_one("threads").map(|v| *v).unwrap_or(1);
    bp3d_tracing::setup!("bp3d-sdk");
    let compiler = Compiler::new(Config {
        n_threads,
        width,
        height,
        format,
        debug: matches.contains_id("debug"),
        output
    });

}

/*fn run() -> i32 {
    let matches = Command::new(PROG_NAME)
        .author("BlockProject 3D")
        .about("BlockProject 3D SDK - Shader Compiler")
        .version(PROG_VERSION)
        .args([
            Arg::new("debug").short('d').long("debug")
                .help("Enable debug PNG output"),
            Arg::new("template").short('t').long("template")
                .value_parser(value_parser!(PathBuf)).num_args(1).required(true)
                .help("Specify the texture template"),
            Arg::new("output").short('o').long("output").num_args(1)
                .value_parser(value_parser!(PathBuf)).help("Output texture file name"),
            Arg::new("threads").short('n').long("threads").num_args(1)
                .help("Specify the maximum number of threads to use when processing shaders"),
            Arg::new("width").long("width").num_args(1)
                .help("Override output texture width"),
            Arg::new("height").long("height").num_args(1)
                .help("Override output texture height"),
            Arg::new("parameter").short('p').long("parameter").action(ArgAction::Append)
                .num_args(2).value_parser(value_parser!(OsString))
                .help("Specify a template parameter using the syntax <parameter name>=<parameter value>")
        ]).get_matches();
    let template_path = matches.get_one::<PathBuf>("template").map(|v| &**v).unwrap();
    info!("Loading template {:?}...", template_path);
    let template = etry!(("failed to load template" 1) =>
        template::Template::load(template_path));
    //Yet another variable which has no meaning
    let fuckingrust = matches.get_many::<OsString>("parameter")
        .map(|v| v.map(|v| &**v).collect::<Vec<&OsStr>>());
    let new_params = fuckingrust.as_deref().map(|v| v.chunks_exact(2).map(|v| (v[0].to_str(), &*v[1])));
    let params = etry!(("failed to parse parameters" 1) =>
        params::Parameters::parse(&template, matches.get_many::<OsString>("parameter").map(|v| v.map(|v| &**v))));
    let width: u32 = matches
        .get_one("width").map(|v| *v)
        .or_else(|| template.try_width_from_base_texture(&params))
        .unwrap_or(template.default_width);
    let height: u32 = matches
        .get_one("height").map(|v| *v)
        .or_else(|| template.try_height_from_base_texture(&params))
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
        matches.get_one("threads").map(|v| *v).unwrap_or(1),
    );
    for _ in 0..pass_count {
        etry!(("failed to run pass" 1) => pipeline.next_pass());
    }
    let render_target = pipeline.finish();
    if matches.contains_id("debug") {
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
}*/

// Copyright (c) 2023, BlockProject 3D
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

use std::fs::File;
use std::path::Path;
use std::io;
use std::io::BufWriter;
use std::io::Write;

fn snake_case_to_pascal_case(mut name: &str) -> String {
    let mut res = String::new();
    while let Some(index) = name.find('_') {
        if name.len() > 1 {
            res += &(name[0..1].to_uppercase() + &name[1..index]);
        }
        if index + 1 < name.len() {
            name = &name[index + 1..];
        }
    }
    if name.len() > 1 {
        res += &(name[0..1].to_uppercase() + &name[1..]);
    }
    res
}

fn list_filters() -> io::Result<Vec<String>> {
    let path = Path::new("src/filter");
    let mut names = Vec::new();
    for v in std::fs::read_dir(path)? {
        let entry = v?;
        let fuckyourust = entry.file_name();
        let filter_file_name = fuckyourust.to_string_lossy();
        if filter_file_name == "mod.rs" {
            continue;
        }
        if filter_file_name.ends_with(".rs") {
            let filter_name = &filter_file_name[..filter_file_name.len() - 3];
            names.push(filter_name.into());
        } else if entry.file_type()?.is_dir() {
            names.push(filter_file_name.into());
        }
    }
    Ok(names)
}

fn write_file(filters: Vec<String>, out_file: &Path) -> io::Result<()> {
    let path = std::fs::canonicalize(Path::new("src/filter")).expect("Failed to get absolute path to source directory");
    let mut file = BufWriter::new(File::create(out_file)?);
    let module_imports: Vec<String> = filters.iter().map(|v| format!("#[path=\"{}.rs\"] mod {};", path.join(v).to_string_lossy(), v)).collect();
    let variants = filters.iter()
        .map(|v| snake_case_to_pascal_case(v))
        .collect::<Vec<String>>().join(",");
    let variants_filter: Vec<String> = filters.iter()
        .map(|v| (v, snake_case_to_pascal_case(v)))
        .map(|(module, obj)| format!("{}({}::{}),", obj, module, obj)).collect();
    let variants_function: Vec<String> = filters.iter()
        .map(|v| (v, snake_case_to_pascal_case(v)))
        .map(|(module, obj)| format!("{}(<{}::{} as crate::filter::Filter>::Function),", obj, module, obj)).collect();
    let variants_from_name: Vec<String> = filters.iter()
        .map(|v| (v, snake_case_to_pascal_case(v)))
        .map(|(module, obj)| format!("\"{}\" => Some(<{}::{} as crate::filter::New>::new(params).map(DynamicFilter::{})),", module, module, obj, obj)).collect();
    writeln!(file, "{}", module_imports.join("\n"))?;
    writeln!(file, "pub enum DynamicFilter {{")?;
    writeln!(file, "    {}", variants_filter.join("\n"))?;
    writeln!(file, "}}")?;
    writeln!(file, "")?;
    writeln!(file, "pub enum DynamicFunction {{")?;
    writeln!(file, "    {}", variants_function.join("\n"))?;
    writeln!(file, "}}")?;
    writeln!(file, "")?;
    writeln!(file, "impl_function!(DynamicFunction {{ {} }});", variants)?;
    writeln!(file, "impl_filter!((DynamicFilter, DynamicFunction) {{ {} }});", variants)?;
    writeln!(file, "")?;
    writeln!(file, "impl DynamicFilter {{")?;
    writeln!(file, "    pub fn from_name(params: &ParameterMap, name: &str) -> Option<Result<DynamicFilter, FilterError>> {{")?;
    writeln!(file, "        match name {{")?;
    writeln!(file, "            {}", variants_from_name.join(",\n"))?;
    writeln!(file, "            _ => None")?;
    writeln!(file, "        }}")?;
    writeln!(file, "    }}")?;
    writeln!(file, "}}")?;
    Ok(())
}

fn main() {
    let out_file = std::env::var_os("OUT_DIR")
        .map(|v| Path::new(&v).join("filters.rs"))
        .expect("Could not obtain Cargo output directory");
    let filters = list_filters().expect("Failed to obtain the list of available filters");
    write_file(filters, &out_file).expect("Failed to generate filter registry");
    println!("cargo:rustc-env=SRC_FILTER_REGISTRY={}", out_file.to_string_lossy());
}

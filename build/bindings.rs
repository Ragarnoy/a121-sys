use crate::error::{BuildError, Result};
use bindgen::Builder;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::{env, fs};

pub fn generate_bindings(rss_path: &Path) -> Result<()> {
    let headers = rss_path.join("include");
    if !headers.exists() {
        return Err(BuildError::HeadersNotFound(headers));
    }

    let mut bindings = Builder::default()
        .use_core()
        .clang_arg("-I/usr/lib/arm-none-eabi/include/")
        .clang_arg(format!("-I{}", headers.display()))
        .generate_cstr(true);

    bindings = add_headers_to_bindings(bindings, &headers)?;
    bindings = add_log_wrapper(bindings)?;

    let out_path = PathBuf::from(env::var("OUT_DIR").map_err(BuildError::EnvVar)?);
    let bindings = bindings
        .generate()
        .map_err(|_| BuildError::BindgenError("Failed to generate bindings".into()))?;

    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .map_err(BuildError::Io)?;

    Ok(())
}

fn add_headers_to_bindings(mut bindings: Builder, headers: &Path) -> Result<Builder> {
    for entry in fs::read_dir(headers)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension() == Some(OsStr::new("h")) {
            let filename = path.file_name().unwrap().to_str().unwrap();
            let is_distance = filename.contains("distance") && cfg!(feature = "distance");
            let is_presence = filename.contains("presence") && cfg!(feature = "presence");
            let is_other = !filename.contains("distance") && !filename.contains("presence");

            if is_distance || is_presence || is_other {
                bindings = bindings.header(path.to_str().unwrap());
            }
        }
    }
    Ok(bindings)
}

fn add_log_wrapper(mut bindings: Builder) -> Result<Builder> {
    cc::Build::new()
        .file("c_src/logging.c")
        .include("c_src")
        .warnings_into_errors(true)
        .extra_warnings(true)
        .compile("log");

    println!("cargo:rerun-if-changed=c_src/logging.c");
    println!("cargo:rustc-link-lib=static=log");
    bindings = bindings.header("c_src/logging.h");

    Ok(bindings)
}

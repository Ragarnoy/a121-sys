use crate::error::{BuildError, Result};
use bindgen::Builder;
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

pub fn generate_bindings(rss_path: &Path) -> Result<()> {
    let headers = rss_path.join("include");
    if !headers.exists() {
        return Err(BuildError::HeadersNotFound(headers));
    }

    // Get target-specific configuration
    let target = env::var("TARGET").unwrap_or_default();

    // Base bindgen configuration
    let mut builder = Builder::default().use_core().generate_cstr(true);

    // Add target-specific configurations
    if target.contains("thumb") || target.contains("arm") {
        // For ARM targets
        builder = builder
            .clang_arg("--target=arm-none-eabi")
            .clang_arg("-mcpu=cortex-m4")
            .clang_arg("-mthumb")
            .clang_arg("-mfloat-abi=hard")
            .clang_arg("-mfpu=fpv4-sp-d16");

        // Add ARM-specific include paths
        if let Ok(gcc_path) = env::var("ARM_GCC_PATH") {
            builder = builder
                .clang_arg(format!("-I{}/arm-none-eabi/include", gcc_path))
                .clang_arg(format!(
                    "-I{}/lib/gcc/arm-none-eabi/9.3.1/include",
                    gcc_path
                ));
        }
    }

    // Add common include paths
    if let Ok(cpath) = env::var("CPATH") {
        for path in cpath.split(':') {
            builder = builder.clang_arg(format!("-I{}", path));
        }
    }

    // Add our headers path
    builder = builder.clang_arg(format!("-I{}", headers.display()));

    // Add headers and generate bindings
    let mut bindings = add_headers_to_bindings(builder, &headers)?;
    bindings = add_log_wrapper(bindings)?;

    let bindings = bindings
        .generate()
        .map_err(|_| BuildError::BindgenError("Failed to generate bindings".into()))?;

    // Write bindings to file
    let out_path = PathBuf::from(env::var("OUT_DIR").map_err(BuildError::EnvVar)?);
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
    // Determine target-specific compiler settings
    let target = env::var("TARGET").unwrap_or_default();
    let mut build = cc::Build::new();

    if target.contains("thumb") || target.contains("arm") {
        build
            .compiler("arm-none-eabi-gcc")
            .flag("-mcpu=cortex-m4")
            .flag("-mthumb")
            .flag("-mfloat-abi=hard")
            .flag("-mfpu=fpv4-sp-d16");
    }

    build
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

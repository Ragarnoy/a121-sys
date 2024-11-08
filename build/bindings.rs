use crate::error::{BuildError, Result};
use bindgen::Builder;
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn generate_bindings(rss_path: &Path) -> Result<()> {
    let headers = rss_path.join("include");
    if !headers.exists() {
        return Err(BuildError::HeadersNotFound(headers));
    }

    // Get target-specific configuration
    let target = env::var("TARGET").unwrap_or_default();

    // Get GCC sysroot
    let sysroot = get_gcc_sysroot()?;

    // Base bindgen configuration
    let mut builder = Builder::default()
        .use_core()
        .generate_cstr(true)
        .detect_include_paths(true);

    // Add target-specific configurations
    if target.contains("thumb") || target.contains("arm") {
        builder = builder
            .clang_arg("--target=arm-none-eabi")
            .clang_arg("-mcpu=cortex-m4")
            .clang_arg("-mthumb")
            .clang_arg("-mfloat-abi=hard")
            .clang_arg("-mfpu=fpv4-sp-d16");

        // Add sysroot includes
        builder = builder
            .clang_arg(format!("--sysroot={}", sysroot))
            .clang_arg(format!("-I{}/include", sysroot))
            .clang_arg(format!("-I{}/arm-none-eabi/include", sysroot));

        // Add GCC includes
        if let Ok(output) = Command::new("arm-none-eabi-gcc")
            .args(["-print-libgcc-file-name"])
            .output()
        {
            if let Ok(libgcc_path) = String::from_utf8(output.stdout) {
                let libgcc_dir = Path::new(libgcc_path.trim()).parent().unwrap();
                builder = builder.clang_arg(format!("-I{}/include", libgcc_dir.display()));
                builder = builder.clang_arg(format!("-I{}/include-fixed", libgcc_dir.display()));
            }
        }
    }

    // Add our headers path
    builder = builder.clang_arg(format!("-I{}", headers.display()));

    // Add headers and generate bindings
    let mut bindings = add_headers_to_bindings(builder, &headers)?;
    bindings = add_log_wrapper(bindings)?;

    let bindings = bindings
        .generate()
        .map_err(|e| BuildError::BindgenError(format!("Failed to generate bindings: {:?}", e)))?;

    // Write bindings to file
    let out_path = PathBuf::from(env::var("OUT_DIR").map_err(BuildError::EnvVar)?);
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .map_err(BuildError::Io)?;

    Ok(())
}

fn get_gcc_sysroot() -> Result<String> {
    let output = Command::new("arm-none-eabi-gcc")
        .args(["-print-sysroot"])
        .output()
        .map_err(|e| BuildError::BindgenError(format!("Failed to get GCC sysroot: {}", e)))?;

    String::from_utf8(output.stdout)
        .map(|s| s.trim().to_string())
        .map_err(|e| BuildError::BindgenError(format!("Invalid sysroot path: {}", e)))
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

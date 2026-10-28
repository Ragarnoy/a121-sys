use crate::error::{BuildError, Result};
use crate::target;
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

    let target = target::target();
    let config = target::target_config(&target);

    // CargoCallbacks reruns the build when a header or a BINDGEN_EXTRA_CLANG_ARGS* variable changes
    let mut builder = Builder::default()
        .use_core()
        .generate_cstr(true)
        .detect_include_paths(true)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        // clang aligns max_align_t to 16 bytes on some targets (riscv32), which no Rust type can
        // match, so its layout test fails. The SDK does not use it.
        .blocklist_type("max_align_t");

    if let Some(clang_target) = config.clang_target {
        if config.needs_esp_clang {
            check_esp_clang(&target)?;
        }
        builder = builder.clang_arg(format!("--target={clang_target}"));
        for arg in config.clang_args {
            builder = builder.clang_arg(*arg);
        }
        // Use the C library headers of the cross compiler, not the host's
        let compiler = target::cc_build().get_compiler();
        let include_paths = find_include_paths(compiler.path())?;
        eprintln!("Using include paths of {}:", compiler.path().display());
        for path in include_paths {
            eprintln!("  {}", path.display());
            builder = builder.clang_arg(format!("-isystem{}", path.display()));
        }
    }

    // Add our headers path last
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

/// Xtensa needs Espressif's libclang; the stock one does not know the architecture.
fn check_esp_clang(target: &str) -> Result<()> {
    println!("cargo:rerun-if-env-changed=LIBCLANG_PATH");
    let has_esp_clang = env::var("LIBCLANG_PATH")
        .map(|path| path.contains("esp"))
        .unwrap_or(false);
    if has_esp_clang {
        Ok(())
    } else {
        Err(BuildError::BindgenError(format!(
            "{target} needs Espressif's libclang. Install it with espup and run \
             `source ~/export-esp.sh`, or set LIBCLANG_PATH to its lib directory."
        )))
    }
}

/// Returns the system include directories of `compiler`, from its search list.
fn find_include_paths(compiler: &Path) -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();

    if let Ok(output) = Command::new(compiler)
        .args(["-E", "-Wp,-v", "-xc", "/dev/null"])
        .output()
    {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let mut in_search_list = false;
        for line in stderr.lines() {
            if line.contains("#include") && line.contains("search starts here") {
                in_search_list = true;
                continue;
            }
            if line.contains("End of search list") {
                break;
            }
            if in_search_list && line.starts_with(' ') {
                let path = PathBuf::from(line.trim());
                if path.exists() {
                    paths.push(path);
                }
            }
        }
    }

    // Common Arm toolchain locations, if the compiler could not be asked
    if paths.is_empty() && compiler.to_string_lossy().contains("arm-none-eabi") {
        let common_locations = [
            "/usr/arm-none-eabi/include",
            "/usr/lib/arm-none-eabi/include",
            "/usr/local/arm-none-eabi/include",
            "/opt/homebrew/arm-none-eabi/include",
            "/opt/arm-none-eabi/include",
            "/opt/gcc-arm-none-eabi/arm-none-eabi/include",
        ];
        paths.extend(
            common_locations
                .iter()
                .map(PathBuf::from)
                .filter(|path| path.is_dir()),
        );
    }

    if paths.is_empty() {
        return Err(BuildError::BindgenError(format!(
            "Could not find the include paths of {}. Install the C cross compiler for the target \
             and put it in PATH, or set CC_<target> to it.",
            compiler.display()
        )));
    }

    // newlib-nano keeps some headers in a separate directory
    if !paths.iter().any(|path| path.join("inttypes.h").exists()) {
        let nano = paths
            .iter()
            .filter_map(|path| {
                path.parent()
                    .map(|parent| parent.join("newlib-nano/include"))
            })
            .find(|nano| nano.join("inttypes.h").exists());
        paths.extend(nano);
    }

    Ok(paths)
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

fn add_log_wrapper(bindings: Builder) -> Result<Builder> {
    target::cc_build()
        .file("c_src/logging.c")
        .include("c_src")
        .warnings_into_errors(true)
        .extra_warnings(true)
        .try_compile("log")
        .map_err(|e| BuildError::CompilationError(e.to_string()))?;

    println!("cargo:rerun-if-changed=c_src/logging.c");
    Ok(bindings.header("c_src/logging.h"))
}

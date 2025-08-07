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

    // Base bindgen configuration
    let mut builder = Builder::default()
        .use_core()
        .generate_cstr(true)
        .detect_include_paths(true);

    // Add target-specific configurations
    if target.contains("thumb") || target.contains("arm") {
        // Find ARM toolchain include paths
        let include_paths = find_arm_include_paths()?;

        eprintln!("Using ARM include paths:");
        for path in &include_paths {
            eprintln!("  {}", path.display());
        }

        builder = builder
            .clang_arg("--target=thumbv7em-none-eabihf")
            .clang_arg("-mthumb")
            .clang_arg("-mcpu=cortex-m4")
            .clang_arg("-mfloat-abi=hard")
            .clang_arg("-mfpu=fpv4-sp-d16")
            // Define common macros for embedded systems
            .clang_arg("-D__GNUC__")
            .clang_arg("-D__STDC__=1")
            .clang_arg("-D__ARM_ARCH_7EM__=1");

        // Add all include paths
        for path in include_paths {
            builder = builder.clang_arg(format!("-isystem{}", path.display()));
        }
    } else if target.contains("riscv32imac-esp-espidf") || target.contains("riscv32imc-esp-espidf")
    {
        let sysroot = get_riscv_sysroot()?;

        builder = builder
            .clang_arg("--target=riscv32")
            .clang_arg(format!("--sysroot={}", sysroot))
            .clang_arg(format!("-I{}/include", sysroot))
            .clang_arg(format!("-I{}/riscv32-esp-elf/include", sysroot));

        // Add GCC includes for RISC-V
        if let Ok(output) = Command::new("riscv32-esp-elf-gcc")
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

fn find_arm_include_paths() -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();

    // Method 1: Use arm-none-eabi-gcc to find include paths
    if let Ok(output) = Command::new("arm-none-eabi-gcc")
        .args(["-E", "-Wp,-v", "-xc", "/dev/null"])
        .stderr(std::process::Stdio::piped())
        .output()
    {
        if let Ok(stderr) = String::from_utf8(output.stderr) {
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
                    let path = line.trim();
                    let path_buf = PathBuf::from(path);
                    if path_buf.exists() {
                        paths.push(path_buf);
                    }
                }
            }
        }
    }

    // Method 2: Common locations if method 1 fails
    if paths.is_empty() {
        let common_locations = [
            "/usr/arm-none-eabi/include",
            "/usr/lib/arm-none-eabi/include",
            "/usr/local/arm-none-eabi/include",
            "/opt/arm-none-eabi/include",
            "/opt/gcc-arm-none-eabi/arm-none-eabi/include",
        ];

        for location in &common_locations {
            let path = PathBuf::from(location);
            if path.exists() && path.is_dir() {
                paths.push(path);
            }
        }
    }

    // Method 3: Find relative to arm-none-eabi-gcc location
    if paths.is_empty() {
        if let Ok(output) = Command::new("which").arg("arm-none-eabi-gcc").output() {
            if let Ok(gcc_path) = String::from_utf8(output.stdout) {
                let gcc_path = PathBuf::from(gcc_path.trim());
                if let Some(bin_dir) = gcc_path.parent() {
                    if let Some(prefix_dir) = bin_dir.parent() {
                        let include_path = prefix_dir.join("arm-none-eabi/include");
                        if include_path.exists() {
                            paths.push(include_path);
                        }
                    }
                }
            }
        }
    }

    if paths.is_empty() {
        // Last resort: try using the BINDGEN_EXTRA_CLANG_ARGS hint
        eprintln!("Warning: Could not automatically find ARM toolchain headers.");
        eprintln!("You may need to set BINDGEN_EXTRA_CLANG_ARGS environment variable:");
        eprintln!("export BINDGEN_EXTRA_CLANG_ARGS=\"-I/usr/arm-none-eabi/include\"");

        return Err(BuildError::BindgenError(
            "Could not find ARM toolchain include paths. Please ensure arm-none-eabi-gcc is installed and in PATH.".to_string()
        ));
    }

    // Verify at least one path contains inttypes.h
    let has_inttypes = paths.iter().any(|p| p.join("inttypes.h").exists());
    if !has_inttypes {
        eprintln!("Warning: None of the found paths contain inttypes.h");

        // Check if we need to look for newlib-nano includes
        for path in &paths.clone() {
            let nano_path = path.parent().map(|p| p.join("newlib-nano/include"));
            if let Some(nano_path) = nano_path {
                if nano_path.exists() && nano_path.join("inttypes.h").exists() {
                    paths.push(nano_path);
                    break;
                }
            }
        }
    }

    Ok(paths)
}

fn get_riscv_sysroot() -> Result<String> {
    let output = Command::new("riscv32-esp-elf-gcc")
        .args(["-print-sysroot"])
        .output()
        .map_err(|e| BuildError::BindgenError(format!("Failed to get GCC sysroot: {}", e)))?;

    // Handle empty sysroot for RISC-V
    if output.stdout.is_empty() || output.stdout == b"" {
        let path_output = Command::new("which")
            .args(["riscv32-esp-elf-gcc"])
            .output()
            .map_err(|e| {
                BuildError::BindgenError(format!("Failed to get path via which: {}", e))
            })?;

        let path_string = String::from_utf8(path_output.stdout)
            .map_err(|e| BuildError::BindgenError(format!("Invalid path: {}", e)))?;

        let path = Path::new(path_string.trim());
        if let Some(parent) = path.parent() {
            let sysroot_path = parent.join("../riscv32-esp-elf");
            return Ok(sysroot_path.to_string_lossy().to_string());
        }
    }

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
    } else if target.contains("riscv32imac-esp-espidf") || target.contains("riscv32imc-esp-espidf")
    {
        build.compiler("riscv32-esp-elf-gcc");
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

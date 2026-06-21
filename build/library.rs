use crate::error::{BuildError, Result};
use std::env;
use std::path::{Path, PathBuf};

/// Headers + optional `lib/` directory (crate `rss/` tree or external SDK).
pub fn get_rss_path() -> Result<PathBuf> {
    if let Ok(include) = env::var("A121_RSS_INCLUDE") {
        let include = PathBuf::from(include);
        if include.is_dir() {
            return include
                .parent()
                .map(Path::to_path_buf)
                .ok_or(BuildError::RssPathNotFound);
        }
        return Err(BuildError::HeadersNotFound(include));
    }

    PathBuf::from("rss")
        .canonicalize()
        .map_err(|_| BuildError::RssPathNotFound)
}

/// Directory containing `libacconeer_a121.a` (and optional detector archives).
pub fn discover_library() -> Result<PathBuf> {
    for key in ["A121_RSS_LIB", "ACC_RSS_LIBS"] {
        if let Ok(path) = env::var(key) {
            if path.is_empty() {
                continue;
            }
            let path = PathBuf::from(&path);
            if path.is_dir() && path != Path::new(".") {
                println!("cargo:rerun-if-env-changed={key}");
                return Ok(path);
            }
            return Err(BuildError::LibraryNotFound(path));
        }
    }

    let rss = get_rss_path()?;
    let lib = rss.join("lib");
    if lib.is_dir() {
        return Ok(lib);
    }

    for loc in [
        "libs",
        "staticlibs",
        "../libs",
        "/usr/local/lib/acconeer",
        "/usr/lib/acconeer",
    ] {
        let path = PathBuf::from(loc);
        if path.is_dir() {
            return Ok(path);
        }
    }

    Err(BuildError::LibraryNotFound(lib))
}

pub fn setup_linking(lib_path: &Path) -> Result<()> {
    println!("cargo:rustc-link-search=native={}", lib_path.display());
    println!("cargo:rustc-link-lib=static=acconeer_a121");

    if cfg!(feature = "distance") {
        println!("cargo:rustc-link-lib=static=acc_detector_distance_a121");
    }

    if cfg!(feature = "presence") {
        println!("cargo:rustc-link-lib=static=acc_detector_presence_a121");
    }

    if cfg!(feature = "stub_library") {
        setup_stub_linking()?;
    }

    Ok(())
}

fn setup_stub_linking() -> Result<()> {
    if cfg!(target_arch = "arm") {
        let target = env::var("TARGET").unwrap_or_default();
        let cpu = if target.contains("v8m") || target.contains("m33") {
            "cortex-m33"
        } else {
            "cortex-m4"
        };
        let fpu = if target.contains("v8m") || target.contains("m33") {
            "fpv5-sp-d16"
        } else {
            "fpv4-sp-d16"
        };

        println!("cargo:rustc-linker=arm-none-eabi-gcc");
        println!("cargo:rustc-link-arg=-mcpu={cpu}");
        println!("cargo:rustc-link-arg=-mthumb");
        println!("cargo:rustc-link-arg=-mfloat-abi=hard");
        println!("cargo:rustc-link-arg=-mfpu={fpu}");
    }
    Ok(())
}

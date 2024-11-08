use crate::error::{BuildError, Result};
use std::env;
use std::path::{Path, PathBuf};

pub fn get_rss_path() -> Result<PathBuf> {
    PathBuf::from("rss")
        .canonicalize()
        .map_err(|_| BuildError::RssPathNotFound)
}

pub fn discover_library() -> Result<PathBuf> {
    // Try environment variable first
    if let Ok(path) = env::var("ACC_RSS_LIBS") {
        let path = PathBuf::from(path);
        if path.exists() {
            return Ok(path);
        }
    }

    // Try common locations
    let locations = [
        "libs",
        "staticlibs",
        "../libs",
        "/usr/local/lib/acconeer",
        "/usr/lib/acconeer",
    ];

    for loc in &locations {
        let path = PathBuf::from(loc);
        if path.exists() {
            return Ok(path);
        }
    }

    Err(BuildError::LibraryNotFound(PathBuf::from(".")))
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
    println!("cargo:rustc-linker=arm-none-eabi-gcc");
    println!("cargo:rustc-link-arg=-mcpu=cortex-m4");
    println!("cargo:rustc-link-arg=-mthumb");
    println!("cargo:rustc-link-arg=-mfloat-abi=hard");
    println!("cargo:rustc-link-arg=-mfpu=fpv4-sp-d16");
    Ok(())
}

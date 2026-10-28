use crate::error::{BuildError, Result};
use std::env;
use std::path::{Path, PathBuf};

/// Returns the `rss` directory: the parent of `A121_RSS_INCLUDE` if set, else the crate's `rss`.
///
/// The headers must be from the SDK version the crate binds, or the bindings will not match the
/// libraries.
pub fn get_rss_path() -> Result<PathBuf> {
    println!("cargo:rerun-if-env-changed=A121_RSS_INCLUDE");
    if let Some(include) = env::var_os("A121_RSS_INCLUDE").filter(|path| !path.is_empty()) {
        // Relative paths resolve against this crate's directory, which is rarely meant
        let include = PathBuf::from(include);
        let Ok(include) = include.canonicalize() else {
            return Err(BuildError::HeadersNotFound(include));
        };
        if !include.is_dir() {
            return Err(BuildError::HeadersNotFound(include));
        }
        // The rest of the build expects the headers in `<rss>/include`
        if include.file_name().is_some_and(|name| name == "include") {
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

/// Returns the directory holding `libacconeer_a121.a` and the detector libraries.
pub fn discover_library() -> Result<PathBuf> {
    for key in ["A121_RSS_LIB", "ACC_RSS_LIBS"] {
        println!("cargo:rerun-if-env-changed={key}");
        if let Some(path) = env::var_os(key).filter(|path| !path.is_empty()) {
            // A set but wrong path is an error rather than a silent fallback. Relative paths
            // resolve against this crate's directory, so the error asks for an absolute one.
            let path = PathBuf::from(path);
            return match path.canonicalize() {
                Ok(path) if path.is_dir() => Ok(path),
                _ => Err(BuildError::LibraryNotFound(path)),
            };
        }
    }

    let rss_lib = get_rss_path()?.join("lib");
    if rss_lib.is_dir() {
        return Ok(rss_lib);
    }

    let locations = [
        "libs",
        "staticlibs",
        "../libs",
        "/usr/local/lib/acconeer",
        "/usr/lib/acconeer",
    ];
    locations
        .iter()
        .map(PathBuf::from)
        .find(|path| path.exists())
        .ok_or_else(|| BuildError::LibraryNotFound(PathBuf::from(".")))
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

    Ok(())
}

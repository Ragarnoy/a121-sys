use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let acc_rss_libs =
        get_acc_rss_libs_path().expect("Error determining Acconeer static libs path");
    let rss_path = get_rss_path().expect("Error determining rss directory path");

    setup_linking(&acc_rss_libs);
    rerun_if_changed(&rss_path);
    check_headers_existence(&rss_path);

    generate_bindings(&rss_path).expect("Unable to generate bindings");
}

fn get_acc_rss_libs_path() -> Result<PathBuf, String> {
    let acc_rss_libs = env::var("ACC_RSS_LIBS").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(acc_rss_libs)
        .canonicalize()
        .map_err(|_| "Error pointing to Acconeer static libs path.".to_string())
}

fn get_rss_path() -> Result<PathBuf, String> {
    PathBuf::from("rss")
        .canonicalize()
        .map_err(|_| "rss directory not found".to_string())
}

fn setup_linking(acc_rss_libs: &Path) {
    println!("cargo:rustc-link-search={}", acc_rss_libs.display());
    println!("cargo:rustc-link-lib=static=acconeer_a121");

    if cfg!(feature = "distance") {
        println!("cargo:rustc-link-lib=static=acc_detector_distance_a121");
    }
    if cfg!(feature = "presence") {
        println!("cargo:rustc-link-lib=static=acc_detector_presence_a121");
    }

    eprintln!("ACC_RSS_LIBS: {}", acc_rss_libs.to_str().unwrap());
}

fn rerun_if_changed(rss_path: &Path) {
    println!(
        "cargo:rerun-if-changed={}",
        rss_path.join("include").display()
    );
}

fn check_headers_existence(rss_path: &Path) {
    let headers = rss_path.join("include");
    if !headers.exists() {
        panic!("headers not found");
    }
}

fn generate_bindings(rss_path: &Path) -> Result<(), String> {
    let headers = rss_path.join("include");
    let mut bindings = bindgen::Builder::default()
        .use_core()
        .clang_arg(format!("-I{}", headers.display()))
        .layout_tests(false)
        .generate_cstr(true);

    for entry in
        fs::read_dir(&headers).map_err(|_| "Unable to read headers directory".to_string())?
    {
        let entry = entry.map_err(|_| "Error iterating headers directory".to_string())?;
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

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .generate()
        .map_err(|_| "Unable to generate bindings".to_string())?
        .write_to_file(out_path.join("bindings.rs"))
        .map_err(|_| "Unable to write bindings to file".to_string())
        .expect("Unable to write bindings to file");
    Ok(())
}

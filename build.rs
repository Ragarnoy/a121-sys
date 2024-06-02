use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    let rss_path = get_rss_path()
        .expect("Error determining rss directory path")
        .canonicalize()
        .unwrap();
    let lib_path = if cfg!(feature = "stub_library") {
        let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
        run_python_script(&rss_path);
        generate_stub_libraries(&rss_path, &out_dir);
        out_dir
    } else {
        get_acc_rss_libs_path().expect("Error determining Acconeer static libs path")
    };

    setup_linking(&lib_path); // Now always called regardless of stubs or actual libs
    rerun_if_changed(&rss_path);
    check_headers_existence(&rss_path);
    generate_bindings(&rss_path)
        .map_err(|e| panic!("{}", e))
        .unwrap();
}

fn run_python_script(rss_path: &Path) {
    eprintln!(
        "Running Python script for generating bindings {:?}",
        rss_path.join("generate_bindings.py")
    );
    let script = include_str!("./rss/generate_bindings.py");
    Command::new("python")
        .current_dir(rss_path)
        .arg("-c")
        .arg(script)
        .status()
        .expect("Failed to run Python script for generating bindings");
}

fn generate_stub_libraries(rss_path: &Path, out_dir: &Path) {
    compile_and_archive(
        out_dir,
        rss_path,
        "acconeer_a121_stubs.c",
        "acconeer_a121_stubs.o",
        "libacconeer_a121.a",
    );

    if cfg!(feature = "distance") {
        compile_and_archive(
            out_dir,
            rss_path,
            "acc_detector_distance_a121_stubs.c",
            "acc_detector_distance_a121_stubs.o",
            "libacc_detector_distance_a121.a",
        );
    }

    if cfg!(feature = "presence") {
        compile_and_archive(
            out_dir,
            rss_path,
            "acc_detector_presence_a121_stubs.c",
            "acc_detector_presence_a121_stubs.o",
            "libacc_detector_presence_a121.a",
        );
    }
}

fn compile_and_archive(
    out_dir: &Path,
    rss_path: &Path,
    source_file: &str,
    obj_file_name: &str,
    lib_name: &str,
) {
    let source_path = rss_path.join(source_file);
    let obj_path = out_dir.join(obj_file_name);
    let lib_path = out_dir.join(lib_name);

    Command::new("arm-none-eabi-gcc")
        .args([
            "-c",
            source_path.to_str().unwrap(),
            "-o",
            obj_path.to_str().unwrap(),
            "-I",
            rss_path.join("include").to_str().unwrap(),
            "-mcpu=cortex-m4",
            "-mthumb",
            "-mfloat-abi=hard",
            "-mfpu=fpv4-sp-d16",
            "-DTARGET_ARCH_cm4",
            "-DFLOAT_ABI_HARD",
            "-std=c99",
            "-MMD",
            "-MP",
            "-O2",
            "-g",
            "-fno-math-errno",
            "-ffunction-sections",
            "-fdata-sections",
            "-flto=auto",
            "-ffat-lto-objects",
        ])
        .status()
        .expect("Failed to compile C source file");

    Command::new("ar")
        .args([
            "rcs",
            lib_path.to_str().unwrap(),
            obj_path.to_str().unwrap(),
        ])
        .status()
        .expect("Failed to create static library");
}

fn setup_linking(lib_path: &Path) {
    println!("cargo:rustc-link-search=native={}", lib_path.display());
    println!("cargo:rustc-link-lib=static=acconeer_a121");

    if cfg!(feature = "distance") {
        println!("cargo:rustc-link-lib=static=acc_detector_distance_a121");
    }

    if cfg!(feature = "presence") {
        println!("cargo:rustc-link-lib=static=acc_detector_presence_a121");
    }
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
        .map_err(|_| "RSS directory not found".to_string())
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
        panic!("Headers not found");
    }
}

fn generate_bindings(rss_path: &Path) -> Result<(), String> {
    let headers = rss_path.join("include");
    let mut bindings = bindgen::Builder::default()
        .use_core()
        .clang_arg("-I/usr/lib/arm-none-eabi/include/")
        .clang_arg(format!("-I{}", headers.display()))
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
    bindings = c_log_wrapper(bindings);

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .generate()
        .map_err(|_| "Unable to generate bindings".to_string())?
        .write_to_file(out_path.join("bindings.rs"))
        .map_err(|_| "Unable to write bindings to file".to_string())
        .expect("Unable to write bindings to file");
    Ok(())
}

fn c_log_wrapper(mut bindings: bindgen::Builder) -> bindgen::Builder {
    cc::Build::new()
        .file("c_src/logging.c")
        .include("c_src")
        .warnings_into_errors(true)
        .extra_warnings(true)
        .compile("log");
    println!("cargo:rerun-if-changed=c_src/logging.c");
    println!("cargo:rustc-link-lib=static=log");
    bindings = bindings.header("c_src/logging.h");
    bindings
}

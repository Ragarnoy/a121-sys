use crate::error::{BuildError, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn generate_stubs(rss_path: &Path, out_dir: &PathBuf) -> Result<()> {
    run_python_script(rss_path)?;
    generate_stub_libraries(rss_path, out_dir)?;
    // check if arm-none-eabi-nm is available, if so, validate stub libraries
    if Command::new("arm-none-eabi-nm").output().is_ok() {
        validate_stub_library(&out_dir.join("libacconeer_a121.a"))?;
        if cfg!(feature = "distance") {
            validate_stub_library(&out_dir.join("libacc_detector_distance_a121.a"))?;
        }
        if cfg!(feature = "presence") {
            validate_stub_library(&out_dir.join("libacc_detector_presence_a121.a"))?;
        }
    }
    Ok(())
}

fn run_python_script(rss_path: &Path) -> Result<()> {
    let script_path = rss_path.join("generate_bindings.py");
    if !script_path.exists() {
        return Err(BuildError::StubGenerationFailed(
            "Python script not found".into(),
        ));
    }

    let output = Command::new("python3")
        .current_dir(rss_path)
        .arg("generate_bindings.py")
        .output()
        .map_err(|e| BuildError::PythonError(e.to_string()))?;

    if !output.status.success() {
        return Err(BuildError::PythonError(
            String::from_utf8_lossy(&output.stderr).into_owned(),
        ));
    }

    Ok(())
}

fn generate_stub_libraries(rss_path: &Path, out_dir: &Path) -> Result<()> {
    compile_and_archive(
        out_dir,
        rss_path,
        "acconeer_a121_stubs.c",
        "acconeer_a121_stubs.o",
        "libacconeer_a121.a",
    )?;

    if cfg!(feature = "distance") {
        compile_and_archive(
            out_dir,
            rss_path,
            "acc_detector_distance_a121_stubs.c",
            "acc_detector_distance_a121_stubs.o",
            "libacc_detector_distance_a121.a",
        )?;
    }

    if cfg!(feature = "presence") {
        compile_and_archive(
            out_dir,
            rss_path,
            "acc_detector_presence_a121_stubs.c",
            "acc_detector_presence_a121_stubs.o",
            "libacc_detector_presence_a121.a",
        )?;
    }

    Ok(())
}

fn compile_and_archive(
    out_dir: &Path,
    rss_path: &Path,
    source_file: &str,
    obj_file_name: &str,
    lib_name: &str,
) -> Result<()> {
    let source_path = rss_path.join(source_file);
    let obj_path = out_dir.join(obj_file_name);
    let lib_path = out_dir.join(lib_name);

    let status = Command::new("arm-none-eabi-gcc")
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
        .map_err(|e| BuildError::CompilationError(e.to_string()))?;

    if !status.success() {
        return Err(BuildError::CompilationError(format!(
            "Failed to compile {}",
            source_file
        )));
    }

    let status = Command::new("arm-none-eabi-ar")
        .args([
            "rcs",
            lib_path.to_str().unwrap(),
            obj_path.to_str().unwrap(),
        ])
        .status()
        .map_err(|e| BuildError::CompilationError(e.to_string()))?;

    if !status.success() {
        return Err(BuildError::CompilationError(format!(
            "Failed to create archive {}",
            lib_name
        )));
    }

    Ok(())
}

fn validate_stub_library(lib_path: &Path) -> Result<()> {
    let output = Command::new("arm-none-eabi-nm").arg(lib_path).output()?;

    if !output.status.success() {
        Err(BuildError::StubGenerationFailed(format!(
            "Failed to validate stub library {}\n nm output:\n {}\n",
            lib_path.to_str().unwrap(),
            String::from_utf8_lossy(&output.stderr)
        )))
    } else {
        Ok(())
    }
}

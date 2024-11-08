use crate::error::{BuildError, Result};
use crate::stub_generator::StubGenerator;
use std::path::Path;
use std::process::Command;

pub fn generate_stubs(rss_path: &Path, out_dir: &Path) -> Result<()> {
    let include_dir = rss_path.join("include");
    if !include_dir.exists() {
        return Err(BuildError::StubGenerationFailed(
            "Include directory not found".into(),
        ));
    }

    // Generate stubs using our Rust generator
    let generator = StubGenerator::default();
    generator.generate_stubs(&include_dir, out_dir)?;

    // Compile the generated stubs
    generate_stub_libraries(out_dir, &include_dir)?;

    // Validate the generated libraries if the tools are available
    if Command::new("arm-none-eabi-nm").output().is_ok() {
        validate_stub_libraries(out_dir)?;
    }

    Ok(())
}

pub fn generate_stub_libraries(out_dir: &Path, include_dir: &Path) -> Result<()> {
    compile_and_archive(
        out_dir,
        include_dir,
        "acconeer_a121_stubs.c",
        "acconeer_a121_stubs.o",
        "libacconeer_a121.a",
    )?;

    if cfg!(feature = "distance") {
        compile_and_archive(
            out_dir,
            include_dir,
            "acc_detector_distance_a121_stubs.c",
            "acc_detector_distance_a121_stubs.o",
            "libacc_detector_distance_a121.a",
        )?;
    }

    if cfg!(feature = "presence") {
        compile_and_archive(
            out_dir,
            include_dir,
            "acc_detector_presence_a121_stubs.c",
            "acc_detector_presence_a121_stubs.o",
            "libacc_detector_presence_a121.a",
        )?;
    }

    Ok(())
}

fn compile_and_archive(
    out_dir: &Path,
    include_dir: &Path,
    source_file: &str,
    obj_file_name: &str,
    lib_name: &str,
) -> Result<()> {
    let source_path = out_dir.join(source_file);
    let obj_path = out_dir.join(obj_file_name);
    let lib_path = out_dir.join(lib_name);

    // Compile the source file
    let status = Command::new("arm-none-eabi-gcc")
        .args([
            "-c",
            source_path.to_str().unwrap(),
            "-o",
            obj_path.to_str().unwrap(),
            "-I",
            include_dir.to_str().unwrap(),
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

    // Create archive
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

fn validate_stub_libraries(out_dir: &Path) -> Result<()> {
    validate_stub_library(out_dir, "libacconeer_a121.a")?;

    if cfg!(feature = "distance") {
        validate_stub_library(out_dir, "libacc_detector_distance_a121.a")?;
    }

    if cfg!(feature = "presence") {
        validate_stub_library(out_dir, "libacc_detector_presence_a121.a")?;
    }

    Ok(())
}

fn validate_stub_library(out_dir: &Path, lib_name: &str) -> Result<()> {
    let lib_path = out_dir.join(lib_name);

    let output = Command::new("arm-none-eabi-nm")
        .arg(&lib_path)
        .output()
        .map_err(|e| {
            BuildError::StubGenerationFailed(format!("Failed to run nm on {}: {}", lib_name, e))
        })?;

    if !output.status.success() {
        return Err(BuildError::StubGenerationFailed(format!(
            "Failed to validate stub library {}\n nm output:\n {}",
            lib_path.display(),
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    // Additional validation could be added here, such as checking for specific symbols

    Ok(())
}

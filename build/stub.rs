use crate::error::{BuildError, Result};
use crate::stub_generator::StubGenerator;
use std::collections::HashMap;
use std::env;
use std::path::Path;
use std::process::Command;

/// Tools used to compile, archive and inspect the stub libraries.
struct StubToolchain {
    compiler: &'static str,
    archiver: &'static str,
    nm: &'static str,
    compile_args: &'static [&'static str],
}

/// Selects the stub toolchain for the target being built.
///
/// `cfg!(target_arch)` describes the host in a build script, so the target must be read
/// from the `TARGET` environment variable instead.
fn stub_toolchain() -> StubToolchain {
    let target = env::var("TARGET").unwrap_or_default();

    if target.contains("thumb") || target.contains("arm") {
        StubToolchain {
            compiler: "arm-none-eabi-gcc",
            archiver: "arm-none-eabi-ar",
            nm: "arm-none-eabi-nm",
            compile_args: &[
                "-mcpu=cortex-m4",
                "-mthumb",
                "-mfloat-abi=hard",
                "-mfpu=fpv4-sp-d16",
                "-DTARGET_ARCH_cm4",
                "-DFLOAT_ABI_HARD",
            ],
        }
    } else if target.contains("riscv32imac-esp-espidf") || target.contains("riscv32imc-esp-espidf")
    {
        StubToolchain {
            compiler: "riscv32-esp-elf-gcc",
            archiver: "riscv32-esp-elf-ar",
            nm: "riscv32-esp-elf-nm",
            compile_args: &[],
        }
    } else {
        StubToolchain {
            compiler: "cc",
            archiver: "ar",
            nm: "nm",
            compile_args: &[],
        }
    }
}

/// Stub source file and the static library built from it.
fn stub_libraries() -> Vec<(&'static str, &'static str)> {
    let mut libs = vec![("acconeer_a121_stubs.c", "libacconeer_a121.a")];

    if cfg!(feature = "distance") {
        libs.push((
            "acc_detector_distance_a121_stubs.c",
            "libacc_detector_distance_a121.a",
        ));
    }

    if cfg!(feature = "presence") {
        libs.push((
            "acc_detector_presence_a121_stubs.c",
            "libacc_detector_presence_a121.a",
        ));
    }

    libs
}

pub fn generate_stubs(rss_path: &Path, out_dir: &Path) -> Result<()> {
    let include_dir = rss_path.join("include");
    if !include_dir.exists() {
        return Err(BuildError::StubGenerationFailed(
            "Include directory not found".into(),
        ));
    }

    // Generate stubs using our Rust generator
    let generator = StubGenerator::default();
    let stubbed = generator.generate_stubs(&include_dir, out_dir)?;

    // Compile the generated stubs
    generate_stub_libraries(out_dir, &include_dir)?;

    // Validate the generated libraries if the tools are available
    if Command::new(stub_toolchain().nm)
        .arg("--version")
        .output()
        .is_ok()
    {
        validate_stub_libraries(out_dir, &stubbed)?;
    }

    Ok(())
}

pub fn generate_stub_libraries(out_dir: &Path, include_dir: &Path) -> Result<()> {
    for (source_file, lib_name) in stub_libraries() {
        let obj_file_name = source_file.replace(".c", ".o");
        compile_and_archive(out_dir, include_dir, source_file, &obj_file_name, lib_name)?;
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

    let toolchain = stub_toolchain();

    // Common compile arguments
    let mut args = vec![
        "-c",
        source_path.to_str().unwrap(),
        "-o",
        obj_path.to_str().unwrap(),
        "-I",
        include_dir.to_str().unwrap(),
        "-std=c99",
        "-MMD",
        "-MP",
        "-O2",
        "-g",
        "-fno-math-errno",
        "-ffunction-sections",
        "-fdata-sections",
    ];

    // Add architecture-specific args
    args.extend(toolchain.compile_args);

    // Compile the source file
    let status = Command::new(toolchain.compiler)
        .args(&args)
        .status()
        .map_err(|e| BuildError::CompilationError(e.to_string()))?;

    if !status.success() {
        return Err(BuildError::CompilationError(format!(
            "Failed to compile {}",
            source_file
        )));
    }

    // Create archive
    let status = Command::new(toolchain.archiver)
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

fn validate_stub_libraries(out_dir: &Path, stubbed: &HashMap<String, Vec<String>>) -> Result<()> {
    for (source_file, lib_name) in stub_libraries() {
        let expected = stubbed
            .get(source_file)
            .map(Vec::as_slice)
            .unwrap_or_default();
        validate_stub_library(out_dir, lib_name, expected)?;
    }

    Ok(())
}

fn validate_stub_library(out_dir: &Path, lib_name: &str, expected: &[String]) -> Result<()> {
    let lib_path = out_dir.join(lib_name);

    let output = Command::new(stub_toolchain().nm)
        .args(["-g", "--defined-only"])
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

    // Every function declared in the headers must be defined in the archive.
    // Mach-O (macOS) prefixes C symbols with an underscore.
    let stdout = String::from_utf8_lossy(&output.stdout);
    let defined: Vec<&str> = stdout
        .lines()
        .filter_map(|line| line.split_whitespace().last())
        .flat_map(|symbol| [symbol, symbol.strip_prefix('_').unwrap_or(symbol)])
        .collect();
    let missing: Vec<&str> = expected
        .iter()
        .map(String::as_str)
        .filter(|name| !defined.contains(name))
        .collect();

    if expected.is_empty() {
        return Err(BuildError::StubGenerationFailed(format!(
            "No functions were extracted from the headers for stub library {}",
            lib_name
        )));
    }

    if !missing.is_empty() {
        return Err(BuildError::StubGenerationFailed(format!(
            "Stub library {} is missing functions: {:?}",
            lib_name, missing
        )));
    }

    Ok(())
}

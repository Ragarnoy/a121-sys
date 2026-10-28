use crate::error::{BuildError, Result};
use crate::stub_generator::StubGenerator;
use crate::target;
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

/// Returns the `nm` matching the compiler `cc` picks for the target, such as
/// `arm-none-eabi-nm` next to `arm-none-eabi-gcc`, or the host `nm`.
fn nm_tool() -> String {
    let compiler = target::cc_build().get_compiler();
    let name = compiler
        .path()
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    target::binutil(&name, "nm").unwrap_or_else(|| "nm".into())
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
    let nm = nm_tool();
    if Command::new(&nm).arg("--version").output().is_ok() {
        validate_stub_libraries(&nm, out_dir, &stubbed)?;
    }

    Ok(())
}

pub fn generate_stub_libraries(out_dir: &Path, include_dir: &Path) -> Result<()> {
    for (source_file, lib_name) in stub_libraries() {
        let name = lib_name
            .strip_prefix("lib")
            .and_then(|name| name.strip_suffix(".a"))
            .expect("stub libraries are named lib<name>.a");
        // setup_linking links the libraries, so cc only builds them
        let mut build = target::cc_build();
        if target::target() == "thumbv7em-none-eabihf" {
            build
                .define("TARGET_ARCH_cm4", None)
                .define("FLOAT_ABI_HARD", None);
        }
        build
            .file(out_dir.join(source_file))
            .include(include_dir)
            .out_dir(out_dir)
            .cargo_metadata(false)
            // The generated stubs have unused variables
            .warnings(false)
            .std("c99")
            .opt_level(2)
            .flag("-fno-math-errno")
            .try_compile(name)
            .map_err(|e| BuildError::CompilationError(format!("{source_file}: {e}")))?;
    }

    Ok(())
}

fn validate_stub_libraries(
    nm: &str,
    out_dir: &Path,
    stubbed: &HashMap<String, Vec<String>>,
) -> Result<()> {
    for (source_file, lib_name) in stub_libraries() {
        let expected = stubbed
            .get(source_file)
            .map(Vec::as_slice)
            .unwrap_or_default();
        validate_stub_library(nm, out_dir, lib_name, expected)?;
    }

    Ok(())
}

fn validate_stub_library(
    nm: &str,
    out_dir: &Path,
    lib_name: &str,
    expected: &[String],
) -> Result<()> {
    let lib_path = out_dir.join(lib_name);

    let output = Command::new(nm)
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

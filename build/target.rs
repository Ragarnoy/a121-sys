//! Compiler flags for the target being built, shared by bindgen and the C builds.
//!
//! `cfg!(target_arch)` describes the host in a build script, so the target is read from the
//! `TARGET` environment variable instead.

use std::env;
use std::process::Command;

/// Flags for one target triple.
pub struct TargetConfig {
    /// Target passed to clang for bindgen, if the triple needs one
    pub clang_target: Option<&'static str>,
    /// Extra clang arguments for bindgen
    pub clang_args: &'static [&'static str],
    /// Extra arguments for the C compiler, on top of those `cc` derives from the triple
    pub cc_flags: &'static [&'static str],
    /// Whether the target needs Espressif's clang, which knows Xtensa
    pub needs_esp_clang: bool,
    /// C compilers to try, in order, when `CC_<target>` is not set. Empty leaves the choice to `cc`.
    pub compilers: &'static [&'static str],
}

pub fn target() -> String {
    env::var("TARGET").unwrap_or_default()
}

/// Returns the flags for `target`. Targets without an entry, such as the host, need none.
pub fn target_config(target: &str) -> TargetConfig {
    if target.starts_with("thumbv8m.main") {
        // Cortex-M33. `cc` passes `-march=armv8-m.main+fp` for the triple, and `-mcpu` would
        // conflict with it.
        TargetConfig {
            clang_target: Some("thumbv8m.main-none-eabihf"),
            clang_args: &[
                "-mthumb",
                "-mcpu=cortex-m33",
                "-mfloat-abi=hard",
                "-mfpu=fpv5-sp-d16",
                "-D__ARM_ARCH_8M_MAIN__=1",
                "-D__GNUC__",
                "-D__STDC__=1",
            ],
            cc_flags: &["-mfloat-abi=hard", "-mfpu=fpv5-sp-d16"],
            needs_esp_clang: false,
            compilers: &["arm-none-eabi-gcc"],
        }
    } else if target.starts_with("thumb") || target.starts_with("arm") {
        // Cortex-M4F, which the Acconeer Cortex-M libraries are built for
        TargetConfig {
            clang_target: Some("thumbv7em-none-eabihf"),
            clang_args: &[
                "-mthumb",
                "-mcpu=cortex-m4",
                "-mfloat-abi=hard",
                "-mfpu=fpv4-sp-d16",
                "-D__ARM_ARCH_7EM__=1",
                "-D__GNUC__",
                "-D__STDC__=1",
            ],
            cc_flags: &[
                "-mcpu=cortex-m4",
                "-mthumb",
                "-mfloat-abi=hard",
                "-mfpu=fpv4-sp-d16",
            ],
            needs_esp_clang: false,
            compilers: &["arm-none-eabi-gcc"],
        }
    } else if target.starts_with("riscv32imac") {
        // ESP32-C6, ESP32-H2. The ESP toolchain defaults to rv32gc, which may emit FPU instructions.
        TargetConfig {
            clang_target: Some("riscv32-unknown-elf"),
            clang_args: &["-march=rv32imac", "-mabi=ilp32"],
            cc_flags: &["-march=rv32imac", "-mabi=ilp32"],
            needs_esp_clang: false,
            compilers: &["riscv32-esp-elf-gcc", "riscv32-unknown-elf-gcc"],
        }
    } else if target.starts_with("riscv32imc") {
        // ESP32-C3, ESP32-C2
        TargetConfig {
            clang_target: Some("riscv32-unknown-elf"),
            clang_args: &["-march=rv32imc", "-mabi=ilp32"],
            cc_flags: &["-march=rv32imc", "-mabi=ilp32"],
            needs_esp_clang: false,
            compilers: &["riscv32-esp-elf-gcc", "riscv32-unknown-elf-gcc"],
        }
    } else if target.starts_with("xtensa-esp32s3") {
        TargetConfig {
            clang_target: Some("xtensa-esp-elf"),
            clang_args: &["-mcpu=esp32s3"],
            cc_flags: &[],
            needs_esp_clang: true,
            compilers: &["xtensa-esp32s3-elf-gcc"],
        }
    } else if target.starts_with("xtensa-esp32s2") {
        TargetConfig {
            clang_target: Some("xtensa-esp-elf"),
            clang_args: &["-mcpu=esp32s2"],
            cc_flags: &[],
            needs_esp_clang: true,
            compilers: &["xtensa-esp32s2-elf-gcc"],
        }
    } else if target.starts_with("xtensa-esp32") {
        TargetConfig {
            clang_target: Some("xtensa-esp-elf"),
            clang_args: &["-mcpu=esp32"],
            cc_flags: &[],
            needs_esp_clang: true,
            compilers: &["xtensa-esp32-elf-gcc"],
        }
    } else {
        TargetConfig {
            clang_target: None,
            clang_args: &[],
            cc_flags: &[],
            needs_esp_clang: false,
            compilers: &[],
        }
    }
}

/// A `cc::Build` for the target with its flags and cross compiler. `cc` honours `CC_<target>`,
/// `CFLAGS_<target>` and `AR_<target>`.
pub fn cc_build() -> cc::Build {
    let target = target();
    let config = target_config(&target);
    let mut build = cc::Build::new();
    if !compiler_overridden(&target) {
        if let Some(compiler) = config.compilers.iter().find(|compiler| runs(compiler)) {
            build.compiler(compiler);
            // The host `ar` may not index archives of another architecture
            if let Some(prefix) = compiler.strip_suffix("gcc") {
                build.archiver(format!("{prefix}ar"));
            }
        }
    }
    for flag in config.cc_flags {
        build.flag(flag);
    }
    build
}

/// Whether the user chose the C compiler for `target`, which `cc` then uses.
fn compiler_overridden(target: &str) -> bool {
    let vars = [
        format!("CC_{target}"),
        format!("CC_{}", target.replace(['-', '.'], "_")),
        "TARGET_CC".to_string(),
    ];
    vars.iter().any(|var| {
        println!("cargo:rerun-if-env-changed={var}");
        env::var_os(var).is_some()
    })
}

fn runs(program: &str) -> bool {
    Command::new(program)
        .arg("--version")
        .output()
        .is_ok_and(|output| output.status.success())
}

//! Compiler flags for the target being built, shared by bindgen and the C builds.
//!
//! `cfg!(target_arch)` describes the host in a build script, so the target is read from the
//! `TARGET` environment variable instead.

use std::env;
use std::process::Command;
use std::sync::OnceLock;

/// Flags for one target triple.
pub struct TargetConfig {
    /// Target passed to clang for bindgen, if the triple is cross-compiled
    pub clang_target: Option<String>,
    /// Extra clang arguments for bindgen
    pub clang_args: &'static [&'static str],
    /// Extra arguments for the C compiler, on top of those `cc` derives from the triple
    pub cc_flags: &'static [&'static str],
    /// Whether the target needs Espressif's clang, which knows Xtensa
    pub needs_esp_clang: bool,
    /// C compilers to try, in order, when the user has not chosen one. Empty leaves the choice
    /// to `cc`.
    pub compilers: &'static [&'static str],
}

pub fn target() -> String {
    env::var("TARGET").unwrap_or_default()
}

fn cross(
    clang_target: &str,
    clang_args: &'static [&'static str],
    cc_flags: &'static [&'static str],
    compilers: &'static [&'static str],
    needs_esp_clang: bool,
) -> TargetConfig {
    TargetConfig {
        clang_target: Some(clang_target.to_string()),
        clang_args,
        cc_flags,
        needs_esp_clang,
        compilers,
    }
}

const ARM: &[&str] = &["arm-none-eabi-gcc"];
const RISCV: &[&str] = &["riscv32-esp-elf-gcc", "riscv32-unknown-elf-gcc"];

/// Returns the flags for `target`. Targets without an entry, such as the host, need none.
pub fn target_config(target: &str) -> TargetConfig {
    match target {
        // Cortex-M4F, which the Acconeer Cortex-M libraries are built for
        "thumbv7em-none-eabihf" => cross(
            target,
            &[
                "-mthumb",
                "-mcpu=cortex-m4",
                "-mfloat-abi=hard",
                "-mfpu=fpv4-sp-d16",
                "-D__ARM_ARCH_7EM__=1",
                "-D__GNUC__",
                "-D__STDC__=1",
            ],
            &[
                "-mcpu=cortex-m4",
                "-mthumb",
                "-mfloat-abi=hard",
                "-mfpu=fpv4-sp-d16",
            ],
            ARM,
            false,
        ),
        // Cortex-M33. `cc` passes `-march=armv8-m.main+fp` for the triple, and `-mcpu` would
        // conflict with it.
        "thumbv8m.main-none-eabihf" => cross(
            target,
            &[
                "-mthumb",
                "-mcpu=cortex-m33",
                "-mfloat-abi=hard",
                "-mfpu=fpv5-sp-d16",
                "-D__ARM_ARCH_8M_MAIN__=1",
                "-D__GNUC__",
                "-D__STDC__=1",
            ],
            &["-mfloat-abi=hard", "-mfpu=fpv5-sp-d16"],
            ARM,
            false,
        ),
        // Other Arm targets: `cc` and clang derive the flags from the triple
        t if t.starts_with("thumb") || t.starts_with("arm") => cross(t, &[], &[], ARM, false),
        // ESP32-C6, ESP32-H2. The ESP toolchain defaults to rv32gc, which may emit FPU
        // instructions.
        t if t.starts_with("riscv32imac-") => cross(
            "riscv32-unknown-elf",
            &["-march=rv32imac", "-mabi=ilp32"],
            &["-march=rv32imac", "-mabi=ilp32"],
            RISCV,
            false,
        ),
        // ESP32-C3, ESP32-C2
        t if t.starts_with("riscv32imc-") => cross(
            "riscv32-unknown-elf",
            &["-march=rv32imc", "-mabi=ilp32"],
            &["-march=rv32imc", "-mabi=ilp32"],
            RISCV,
            false,
        ),
        t if t.starts_with("xtensa-esp32s3-") => cross(
            "xtensa-esp-elf",
            &["-mcpu=esp32s3"],
            &[],
            &["xtensa-esp32s3-elf-gcc"],
            true,
        ),
        t if t.starts_with("xtensa-esp32s2-") => cross(
            "xtensa-esp-elf",
            &["-mcpu=esp32s2"],
            &[],
            &["xtensa-esp32s2-elf-gcc"],
            true,
        ),
        t if t.starts_with("xtensa-esp32-") => cross(
            "xtensa-esp-elf",
            &["-mcpu=esp32"],
            &[],
            &["xtensa-esp32-elf-gcc"],
            true,
        ),
        _ => TargetConfig {
            clang_target: None,
            clang_args: &[],
            cc_flags: &[],
            needs_esp_clang: false,
            compilers: &[],
        },
    }
}

/// The C compiler chosen for the target from [`TargetConfig::compilers`], or `None` to leave the
/// choice to `cc`: the user set `CC_<target>` or `TARGET_CC`, or none of the compilers runs.
fn chosen_compiler() -> Option<&'static str> {
    static COMPILER: OnceLock<Option<&'static str>> = OnceLock::new();
    *COMPILER.get_or_init(|| {
        let target = target();
        if user_set(&target_vars("CC", &target)) {
            return None;
        }
        target_config(&target)
            .compilers
            .iter()
            .copied()
            .find(|compiler| runs(compiler))
    })
}

/// A `cc::Build` for the target with its flags and cross compiler. `cc` honours `CC_<target>`,
/// `CFLAGS_<target>` and `AR_<target>`.
pub fn cc_build() -> cc::Build {
    let target = target();
    let mut build = cc::Build::new();
    if let Some(compiler) = chosen_compiler() {
        build.compiler(compiler);
        // The host `ar` may not index archives of another architecture
        if !user_set(&target_vars("AR", &target)) {
            if let Some(ar) = binutil(compiler, "ar") {
                build.archiver(ar);
            }
        }
    }
    for flag in target_config(&target).cc_flags {
        build.flag(flag);
    }
    build
}

/// Returns the binutils `tool` matching the gcc `compiler`, such as `arm-none-eabi-nm` for
/// `arm-none-eabi-gcc`. Espressif toolchains may only ship the unified `xtensa-esp-elf-` tools.
pub fn binutil(compiler: &str, tool: &str) -> Option<String> {
    let prefix = compiler.strip_suffix("gcc")?;
    let mut candidates = vec![format!("{prefix}{tool}")];
    if prefix.starts_with("xtensa-esp32") {
        candidates.push(format!("xtensa-esp-elf-{tool}"));
    }
    candidates.into_iter().find(|candidate| runs(candidate))
}

/// The variables `cc` reads to choose `tool` (`CC` or `AR`) for `target`.
fn target_vars(tool: &str, target: &str) -> [String; 3] {
    [
        format!("{tool}_{target}"),
        format!("{tool}_{}", target.replace(['-', '.'], "_")),
        format!("TARGET_{tool}"),
    ]
}

/// Whether any of `vars` is set, telling cargo to rerun when any of them changes.
fn user_set(vars: &[String]) -> bool {
    let mut set = false;
    for var in vars {
        println!("cargo:rerun-if-env-changed={var}");
        set |= env::var_os(var).is_some();
    }
    set
}

fn runs(program: &str) -> bool {
    Command::new(program)
        .arg("--version")
        .output()
        .is_ok_and(|output| output.status.success())
}

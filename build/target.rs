//! Compiler flags for the target being built, shared by bindgen and the C builds.
//!
//! `cfg!(target_arch)` describes the host in a build script, so the target is read from the
//! `TARGET` environment variable instead.

use std::env;
use std::path::Path;
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
        // Cortex-M4F, which the Acconeer Cortex-M libraries are built for. `cc` derives the
        // architecture and FPU flags from the triple. The defines are kept from earlier versions.
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
            &["-mcpu=cortex-m4"],
            ARM,
            false,
        ),
        // Cortex-M33. `cc` passes `-march=armv8-m.main` and the FPU flags for the triple, and
        // `-mcpu` would conflict with `-march`.
        "thumbv8m.main-none-eabihf" => cross(
            target,
            &[
                "-mthumb",
                "-mcpu=cortex-m33",
                "-mfloat-abi=hard",
                "-mfpu=fpv5-sp-d16",
            ],
            &[],
            ARM,
            false,
        ),
        // Other bare-metal Arm targets: `cc` and clang derive the flags from the triple. Hosted
        // Arm targets (Linux) are left to `cc`.
        t if (t.starts_with("thumb") || t.starts_with("arm")) && t.contains("-none-") => {
            cross(t, &[], &[], ARM, false)
        }
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

/// The C compiler, archiver and `nm` chosen for the target.
pub struct Toolchain {
    /// `None` leaves the choice to `cc`
    pub compiler: Option<&'static str>,
    /// `None` leaves the choice to `cc`
    pub archiver: Option<String>,
    /// `None` when no working `nm` was found
    pub nm: Option<String>,
}

/// Picks the target's tools once. A compiler from [`TargetConfig::compilers`] is used unless the
/// user chose one with `CC_<target>` or `TARGET_CC`, which `cc` then uses. Plain `CC` and `AR` are
/// not taken as a choice for the target, since they often name the host's tools.
pub fn toolchain() -> &'static Toolchain {
    static TOOLCHAIN: OnceLock<Toolchain> = OnceLock::new();
    TOOLCHAIN.get_or_init(|| {
        let target = target();
        let config = target_config(&target);
        let user_cc = user_set(&target_vars("CC", &target));
        let compiler = if user_cc {
            None
        } else {
            config
                .compilers
                .iter()
                .copied()
                .find(|compiler| runs(compiler))
        };
        if compiler.is_none() && config.needs_esp_clang && !user_cc {
            println!(
                "cargo:warning=None of {:?} was found. The C code is built with cc's default \
                 compiler, which may not target this chip; install the toolchain with espup.",
                config.compilers
            );
        }

        // The host `ar` may not index archives of another architecture
        let archiver = match compiler {
            Some(compiler) if !user_set(&target_vars("AR", &target)) => binutil(compiler, "ar"),
            _ => None,
        };
        // `nm` is found next to the compiler `cc` picks, wherever that is
        let cc_compiler = {
            let mut build = cc::Build::new();
            if let Some(compiler) = compiler {
                build.compiler(compiler);
            }
            build.try_get_compiler().ok()
        };
        let nm = cc_compiler.and_then(|tool| {
            let path = tool.path().to_string_lossy();
            // A clang compiler targets many architectures, which only `llvm-nm` can read
            let llvm = tool
                .is_like_clang()
                .then(|| binutil(&path, "llvm-nm"))
                .flatten();
            llvm.or_else(|| binutil(&path, "nm"))
        });
        Toolchain {
            compiler,
            archiver,
            nm,
        }
    })
}

/// A `cc::Build` for the target with its flags and tools. `cc` honours `CC_<target>`,
/// `CFLAGS_<target>` and `AR_<target>`.
pub fn cc_build() -> cc::Build {
    let toolchain = toolchain();
    let mut build = cc::Build::new();
    if let Some(compiler) = toolchain.compiler {
        build.compiler(compiler);
    }
    if let Some(archiver) = &toolchain.archiver {
        build.archiver(archiver);
    }
    for flag in target_config(&target()).cc_flags {
        build.flag(flag);
    }
    build
}

/// Returns the binutils `tool` matching the gcc or clang `compiler`, such as `arm-none-eabi-nm`
/// for `arm-none-eabi-gcc`, looked up next to the compiler and then in `PATH`. Espressif
/// toolchains may only ship the unified `xtensa-esp-elf-` tools.
pub fn binutil(compiler: &str, tool: &str) -> Option<String> {
    let path = Path::new(compiler);
    let file_name = path.file_name()?.to_string_lossy();
    let (stem, exe) = match file_name.strip_suffix(".exe") {
        Some(stem) => (stem, ".exe"),
        None => (&*file_name, ""),
    };
    let prefix = ["gcc", "clang", "cc"]
        .iter()
        .find_map(|suffix| stem.strip_suffix(suffix))?;
    let mut names = vec![format!("{prefix}{tool}{exe}")];
    if prefix.starts_with("xtensa-esp32") {
        names.push(format!("xtensa-esp-elf-{tool}{exe}"));
    }
    let mut candidates = Vec::new();
    for name in &names {
        if path.parent().is_some_and(|dir| !dir.as_os_str().is_empty()) {
            candidates.push(path.with_file_name(name).to_string_lossy().into_owned());
        }
        candidates.push(name.clone());
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
        set |= env::var_os(var).is_some_and(|value| !value.is_empty());
    }
    set
}

fn runs(program: &str) -> bool {
    Command::new(program)
        .arg("--version")
        .output()
        .is_ok_and(|output| output.status.success())
}

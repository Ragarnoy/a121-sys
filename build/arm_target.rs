//! ARM Cortex-M bindgen / GCC flags derived from `TARGET`.

/// GCC / clang arguments for the current embedded ARM target.
pub struct ArmTargetConfig {
    pub clang_target: &'static str,
    pub cpu: &'static str,
    pub fpu: &'static str,
    pub arch_define: Option<&'static str>,
}

/// Select flags for `thumbv7em` (Cortex-M4F) or `thumbv8m.main` (Cortex-M33), etc.
pub fn arm_target_config(target: &str) -> ArmTargetConfig {
    if target.starts_with("thumbv8m.main") || target.contains("m33") {
        ArmTargetConfig {
            clang_target: "thumbv8m.main-none-eabihf",
            cpu: "cortex-m33",
            fpu: "fpv5-sp-d16",
            arch_define: None,
        }
    } else if target.starts_with("thumbv7em-none-eabihf") {
        ArmTargetConfig {
            clang_target: "thumbv7em-none-eabihf",
            cpu: "cortex-m4",
            fpu: "fpv4-sp-d16",
            arch_define: Some("-D__ARM_ARCH_7EM__=1"),
        }
    } else {
        panic!("Unsupported ARM target: {target}");
    }
}
}

impl ArmTargetConfig {
    /// Apply CPU/FPU flags to a `cc::Build`.
    ///
    /// For Cortex-M33, `cc` already passes `-march=armv8-m.main+fp` from the target
    /// triple; adding `-mcpu=cortex-m33` conflicts, so only FPU/ABI flags are set.
    pub fn apply_cc_flags(self, build: &mut cc::Build) {
        if self.cpu == "cortex-m33" {
            build
                .flag("-mfloat-abi=hard")
                .flag(format!("-mfpu={}", self.fpu));
        } else {
            build
                .flag(format!("-mcpu={}", self.cpu))
                .flag("-mthumb")
                .flag("-mfloat-abi=hard")
                .flag(format!("-mfpu={}", self.fpu));
        }
    }

    /// Apply CPU/FPU flags to bindgen/clang.
    pub fn apply_clang_args(self, builder: bindgen::Builder) -> bindgen::Builder {
        let mut builder = builder
            .clang_arg(format!("--target={}", self.clang_target))
            .clang_arg("-mthumb")
            .clang_arg(format!("-mcpu={}", self.cpu))
            .clang_arg("-mfloat-abi=hard")
            .clang_arg(format!("-mfpu={}", self.fpu))
            .clang_arg("-D__GNUC__")
            .clang_arg("-D__STDC__=1");

        if let Some(define) = self.arch_define {
            builder = builder.clang_arg(define);
        }
        builder
    }
}

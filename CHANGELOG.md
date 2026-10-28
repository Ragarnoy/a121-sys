# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.8.1] - 2026-10-28

### Added
- Cortex-M33 (`thumbv8m.main-none-eabihf`) support, from PR #18 by leftger.
- Bare-metal ESP targets: `riscv32imac-unknown-none-elf` and `riscv32imc-unknown-none-elf` (ESP32-C6,
  -H2, -C3, -C2 with esp-hal) and `xtensa-esp32s3-none-elf`, `xtensa-esp32s2-none-elf` and
  `xtensa-esp32-none-elf`. bindgen takes the C library headers from the target's cross compiler,
  so `BINDGEN_EXTRA_CLANG_ARGS_<target>` is no longer needed. Xtensa needs Espressif's libclang
  (`source ~/export-esp.sh`) and the build says so when it is missing.
- `A121_RSS_LIB` and `A121_RSS_INCLUDE` to point at an Acconeer SDK outside the crate, and
  `rss/lib` as a library location, from PR #18. `ACC_RSS_LIBS` is kept as an alias of `A121_RSS_LIB`.

### Changed
- The C code (the logging bridge and the stub libraries) is built with the `cc` crate for every
  target, so `CC_<target>`, `CFLAGS_<target>` and `AR_<target>` apply. Without `CC_<target>`,
  or `TARGET_CC`, the build uses `arm-none-eabi-gcc`, `riscv32-esp-elf-gcc`,
  `riscv32-unknown-elf-gcc` or `xtensa-esp32*-elf-gcc` as the bare-metal target needs; plain `CC`
  and `AR` are not used for the target, since they usually name the host's tools.
- The build reruns when the headers or the `BINDGEN_EXTRA_CLANG_ARGS*`, `CC_<target>`,
  `A121_RSS_LIB`, `A121_RSS_INCLUDE` or `ACC_RSS_LIBS` variables change.
- An `ACC_RSS_LIBS` or `A121_RSS_LIB` pointing at a missing directory is an error instead of being
  skipped for the default locations.
  `A121_RSS_INCLUDE` must name an existing directory called `include`.
- The build warns when an Xtensa target's `xtensa-esp32*-elf-gcc` is not found.
- Build errors print their message instead of their `Debug` form.
- `max_align_t` is no longer generated. clang aligns it to 16 bytes on riscv32, which no Rust type
  matches, so its layout test failed.
- CI installs Ubuntu's `gcc-arm-none-eabi` instead of the 2020 Arm toolchain, and checks
  `thumbv8m.main-none-eabihf`.

### Fixed
- Stub libraries for targets other than Arm and ESP-IDF RISC-V were built with the host compiler,
  so they could not be linked.

## [0.8.0] - 2026-10-27

### Changed
- Updated Acconeer A121 SDK bindings from v1.12.0 to v1.13.0 (no public API changes)

### Added
- `acc_definitions.h` header, providing the `float32_t` type alias

### Fixed
- Stub library generation produced no function stubs since the bindgen 0.71 upgrade;
  stubs are now extracted directly from the C headers and validated against the built archives
- Stub libraries now include the `acc_detector_presence_processing` functions
- Stub libraries are now compiled with the ARM toolchain when targeting ARM,
  and with the ESP RISC-V toolchain when targeting `riscv32imac-esp-espidf` or `riscv32imc-esp-espidf`

### Removed
- Build-script linker directives for stub builds, which never reached dependent crates;
  configure the linker in the final binary's `.cargo/config.toml` instead

## [0.7.0] - 2025-12-30

### Changed
- Updated Acconeer A121 SDK bindings from v1.11.* to v1.12.0

### Added
- `acc_detector_presence_reset_filters()` - Reset the presence detector filters

### Removed
- `acc_detector_presence_config_inter_phase_boost_set()` - Removed from SDK
- `acc_detector_presence_config_inter_phase_boost_get()` - Removed from SDK

### Fixed
- CI workflows now correctly trigger on `master` branch
- Fixed incorrect repository path in CI configuration

## [0.6.0] - 2025-08-07

### Changed
- Updated bindgen to 0.72
- Updated cc to 1.2
- Added support for non-ARM architectures in build scripts

## [0.5.0] - 2024-10-01

- Initial public release with A121 SDK v1.10.* bindings

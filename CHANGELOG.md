# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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

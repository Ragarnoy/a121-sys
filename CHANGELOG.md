# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.7.0] - 2024-12-30

### Changed
- Updated Acconeer A121 SDK bindings from v1.11.* to v1.12.0

### Added
- `acc_detector_presence_config_inter_phase_boost_set()` - Enable inter-frame phase boost for slow motion detection
- `acc_detector_presence_config_inter_phase_boost_get()` - Get inter-frame phase boost status

### Removed
- `acc_detector_presence_reset_filters()` - Removed from SDK

### Fixed
- CI workflows now correctly trigger on `master` branch
- Fixed incorrect repository path in CI configuration

## [0.6.0] - 2024-11-15

### Changed
- Updated bindgen to 0.72
- Updated cc to 1.2
- Added support for non-ARM architectures in build scripts

## [0.5.0] - 2024-10-01

- Initial public release with A121 SDK v1.10.* bindings

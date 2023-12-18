# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).


## [Unreleased]

### Fixed

- Keep lexicographical order of chart configuration files

### Removed

- Remove syscalls filtering


## [1.1.0] - 2023-12-18

### Added

- Use plotters from crates.io instead of github
- Add setting for minimal Y range in trend charts
- Add integration tests
- Add changelog

### Changed

- Draw colorbar entries according to colorbar width
- Scale colorbar width with font in temporal heatmap charts
- Interpolate color of status border in Proxmox and infrastructure charts
- Sort series before plotting them in trend and geographical heatmap charts

### Fixed

- Fix title position for different fonts


## [1.0.0] - 2023-03-08

The first stable release come with a large refactoring.

### Added

- Add title in infrastructure chart configuration
- Limit allowed syscalls with crate extrasafe
- Add support for showing status in Proxmox summary
- Add command-line option for output directory
- Add support for non-isometric geographical heatmaps
- Add option to change right margin for geographical heatmaps
- Add option to change right margin for temporal heatmaps

### Changed

- Split chart generation from chart saving to disk
- Remove host data point to validate case with offline host
- Draw axis below trends in trend chart
- Compute x range from query time range instead of data in trends
- Draw load bars only for online hosts
- Retry operations on failure
- Fallback on environment variables for command-line arguments
- Set title top margin based on font size
- Tie colorbar size to font scale
- Harden systemd unit

### Fixed

- Map NaN to minimal color in colormap
- Fix header of infrastructure and Proxmox summaries
- Use saturating operations to avoid panics for large charts


## [0.16.0] - 2022-07-05

### Added

- Add pre-commit configuration


## [0.15.0] - 2022-02-25

### Added

- Add a feature for framebuffer module
- Accept other data types than `f64` from influxdb
- Add Proxmox structure chart

### Changed

- Use features for enabling chart types
- Refactor headers in infrastructure summary chart
- Improve pipelines for continuous integration


## [0.14.0] - 2021-07-25

### Added

- Add support for colored borders in geographical heatmaps


## [0.13.0] - 2021-07-07

### Added

- Add "now" command-line argument to specify current time
- Add support for horizontal grid in trend charts
- Add configuration option for max y ticks in trend charts
- Add configuration option for max x ticks in trend charts


## [0.12.0] - 2021-06-06

### Added

- Read infrastructure chart vertical step from configuration
- Read trend chart top padding from configuration

### Fixed

- Fix range computation in trend chart

### Changed

- Improve pipelines for continuous integration


## [0.11.0] - 2021-01-28

### Added

- Add progress bar

### Changed

- Make "dangerously_accept_invalid_certs" optional
- Use a single HTTP client for all requests
- Hide progress bar
- Use tracing instead of log
- Improve pipelines for continuous integration


## [0.10.0] - 2020-12-29

### Added

- Add precision to configuration for all types of chart
- Add oldest time limit to geographical heat maps
- Add configuration option to hide legend in trend charts
- Add separate configuration entry for unit in trend charts

### Fixed

- Fix ticks in colormap

### Changed

- Log chart title
- Improve pipelines for continuous integration


## [0.9.0] - 2020-12-29

### Added

- Add more colormaps
- Add context to error
- Add support for drawing last value in trend charts

### Fixed

- Improve parsing of InfluxDB JSON to support null values
- Fill null with previous value in temporal heat maps

### Changed

- Specify font color at the beginning in trend charts
- Use Drone for continuous integration


## [0.8.0] - 2020-12-19

### Added

- Add support for last update notice in infrastructure chart


## [0.7.0] - 2020-10-24

### Added

- Add support for reversing colormaps
- Add support for removing suffix from hosts in infrastructure chart
- Add option for accepting invalid certificates


## [0.6.0] - 2020-07-12

### Added

- Add option to draw markers
- Add support for drawing infrastructure status as a chart

### Changed

- Construct query from parameters in the configuration file


## [0.5.0] - 2020-07-09

### Added

- Add implementation for user-selected palettes
- Add support for specifying font in configuration file
- Add support for drawing images to charts
- Add log entries for all chart types

### Changed

- Rename geographical map to geographical heat map
- Use two digits for chart filename

### Removed

- Remove framebuffer functionality



## [0.4.0] - 2020-07-07

### Added

- Add support for temporal heat maps

### Changed

- Refactor geographical map to use elements
- Center latest readings on centroids in geographical map


## [0.3.0] - 2020-07-06

### Added

- Add support for no regions in configuration file
- Display geographical maps using an isometric view
- Enable Cargo configuration option for release build on Raspberry Pi


## [0.2.0] - 2020-07-05

### Added

- Add support for geographical maps
- Add support for optional tag names

### Fixed

- Handle case when database returns empty time-series


## [0.1.0] - 2020-06-07

### Added

- Initial implementation

[unreleased]: https://gitlab.com/claudiomattera/house-dashboard/-/compare/1.0.0...v1.1.0-devel
[1.1.0]: https://gitlab.com/claudiomattera/house-dashboard/-/compare/1.0.0...1.1.0
[1.0.0]: https://gitlab.com/claudiomattera/house-dashboard/-/compare/0.16.0...1.0.0
[0.16.0]: https://gitlab.com/claudiomattera/house-dashboard/-/compare/0.15.0...0.16.0
[0.15.0]: https://gitlab.com/claudiomattera/house-dashboard/-/compare/0.14.0...0.15.0
[0.14.0]: https://gitlab.com/claudiomattera/house-dashboard/-/compare/0.13.0...0.14.0
[0.13.0]: https://gitlab.com/claudiomattera/house-dashboard/-/compare/0.12.0...0.13.0
[0.12.0]: https://gitlab.com/claudiomattera/house-dashboard/-/compare/0.11.0...0.12.0
[0.11.0]: https://gitlab.com/claudiomattera/house-dashboard/-/compare/0.10.0...0.11.0
[0.10.0]: https://gitlab.com/claudiomattera/house-dashboard/-/compare/0.9.0...0.10.0
[0.9.0]: https://gitlab.com/claudiomattera/house-dashboard/-/compare/0.8.0...0.9.0
[0.8.0]: https://gitlab.com/claudiomattera/house-dashboard/-/compare/0.7.0...0.8.0
[0.7.0]: https://gitlab.com/claudiomattera/house-dashboard/-/compare/0.6.0...0.7.0
[0.6.0]: https://gitlab.com/claudiomattera/house-dashboard/-/compare/0.5.0...0.6.0
[0.5.0]: https://gitlab.com/claudiomattera/house-dashboard/-/compare/0.4.0...0.5.0
[0.4.0]: https://gitlab.com/claudiomattera/house-dashboard/-/compare/0.3.0...0.4.0
[0.3.0]: https://gitlab.com/claudiomattera/house-dashboard/-/compare/0.2.0...0.3.0
[0.2.0]: https://gitlab.com/claudiomattera/house-dashboard/-/compare/0.1.0...0.2.0
[0.1.0]: https://gitlab.com/claudiomattera/house-dashboard/-/tags/0.1.0

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

## [0.20.0]

### Added

- Display log metadata when clicking on a loaded log.

### Fix

- `Mbed status Log v2` mistakenly interpreted as `Mbed status Log v1` when loaded via a path (native `logviewer-rs`)

## [0.19.0]

### Added

- Support for Mbed Log version 2

## [0.18.4]

### Changed

- Remove playback feature

## [0.18.3]

### Fix

- Integer overflow when searching for the appropriate downsampled MipMap level.

## [0.18.2]

### Changed

- Much faster implementation for finding the appropriate MipMap level for the current zoom level.

### Fix

- Min/Max MipMap algorithm flipped min and max.

## [0.18.1]

### Fix

- Fix accidentally using plot **name** instead of plot **label**, causing name conflicts when plotting multiple plots with the same name
- Bad naming in a PID controller implementation caused misunderstanding, what we thought was the PID error was actually the PID output.

## [0.18.0]

### Added

- Min/Max MipMap downsampling which makes plotting much larger datasets feasible, and facilitates outlier detection.

### Internals

- Major clearing of tech debts related to plot settings and the ui for plot settings.

## [0.17.0]

### Added

- Allow toggling whether or not plots from each loaded log are shown.

### Internals

- Update dependencies

## [0.16.0]

### Added

- Hovering on a plot now also shows the plot name.
- Allow Filtering the shown plots by legend/label name.

### Changed

- (Native only) Recursively parsing drag-n-dropped directories also parses zip archives
- Reduce UI by allowing toggling the list of logs.
- Reduce UI cluttering by removing the "Time" label on the X-axis.
- Reduce verbosity of the name of logs
- Better visuals for viewing and changing settings of loaded logs

### Internals

- Refactors

## [0.15.0]

### Added

- (Not available on web version) Recursively parse drag-n-dropped directories for supported logs
- (Not available on web version) Recursively parse drag-n-dropped zip archives for supported logs
- Show tooltip when hovering above a clickable log name

### Internals

- Refactors

## [0.14.0]

### Add

- Ability for logs to add labels (rough initial mvp, needs more work)
- `show grid` button for showing/hiding grid lines on plots

## [0.13.0]

### Internals

- Better divide interfaces in the `log_if` crate and use a prelude to make it easy to import all relevant traits via glob imports.
- Change `GitMetadata` trait to return `Option<String>` to accommodate logs that don't contain git metadata.

### Changes

- Mbed motor control PID log's `RPM Error Count` and `First Valid RPM Count` are moved to the plot ranging from 1-100.

## [0.12.0]

### Changes

- Allow Setting the date/offset of plots with the `Enter`-key and closing the settings window by pressing `Escape`
- Upgrade `egui` to v0.29
- Y-axis lock is now compatible with all zoom and scroll actions

### Internals

- Migrate to workspace project structure
- Decouple Log implementation from the plotting interface

# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Added
- Dev dependency: `image-compare` 0.3.1, `tempdir` 0.3.7.
- Tests for compressed image output order.
- CI for building and testing on `ubuntu-latest`, `windows-latest`, and `macos-latest`.
- Re-exported `ParallelIntoIterator`.
- Documentations and tests.
### Changed
- Parallel compression now returns compressed JPEG bytes in the same order you put them in.
- JPEG bytes are now shared across threads via a `VecDeque` (prev. `crossbeam:deque`). Allowing threads to access it as a FIFO queue.
- Implement what common traits is possible for structs.
### Removed
- `crossbeam` feature `deque`.
- `Default` impl for `Parallel`.
## [1.0.0] - 2024-04-15
### Added
- Added public methods `Single::from_bytes` and `Parallel::from_vec`.
- Added crate error enumerations.
- Added documentations & tests.
- Added dependency `thiserror`
### Changed
- **This crate now only process JPEG bytes.** It no longer interacts with your filesystem. Meaning it doesn't read and/or write files, nor does it read and/or create directories.
- `Parallel` now produces an iterator via `into_iter()` to iterate over compression results sent by spawned threads.
- `compress` method in `Single` is now fallible.
### Removed
- Removed mandatory method `output_dir()`.
- Removed dependencies: `clap`, `anyhow`, `colored`.
### Fixed
- Updated README.

## [0.4.0] - 2024-04-04
- **Now stands as its own crate**, called 'jippigy' (previously a cli tool [smoljpg](https://github.com/rfdzan/smoljpg)).
### Added
- Added `Single` and `Parallel` for processing JPEG images.
- Added required methods `output_dir`.
- Added optional methods prefixed by `with_`.
### Changed
- Renamed project to "jippigy".
- Renamed compression-invoking methods to `compress`.
### Fixed
- Updated README.

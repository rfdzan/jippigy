# 15 April 2024: 1.0.0
- **This crate now only process JPEG bytes. All other filesystem processes is handed over to the user of the crate.**
- Renamed primary public API to `Single::from_bytes` and `Parallel::from_vec` which reflect their input types.
- `Parallel` now produces an iterator via `into_iter()` to iterate over compression results sent by spawned threads.
- Added crate error enumerations.
- Removed mandatory method `output_dir()`.
- Removed dependency: `clap`, `anyhow`, `colored`.
# 04 April 2024: 0.4.0
- now stands as its own crate, called 'jippigy' (previously a cli tool [smoljpg](https://github.com/rfdzan/smoljpg)).
- defining public API, added tests and documentations.
# 22 March 2024: 0.3.0
- cli: Output directory is now optional. Previously, specifying custom quality requires specifying an output directory as well.
# 17 March 2024: 0.2.0
- feature: allow single image compression.

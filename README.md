# Examples
Both `Single` and `Parallel` require you to use both of their respective `output_dir` methods. `with_` methods are optional.

## Single image compressions with `Single`
```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
  Single::builder(image_path)
    .output_dir(output_dir) // This method is required.
    .with_quality(95)
    .with_prefix("my_prefix_".to_string())
    .build()
    .do_single()?;
    Ok(())
}
```
## Multi-threaded bulk compressions with `Parallel`
In this example, [`Parallel`] will attempt to create a separate directory `output_dir/compressed/` if it doesn't exist and save compressed images here.
```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
  Parallel::builder(image_dir.clone())
    .output_dir(image_dir.join("compressed"))? // This method is required.
    .with_quality(95)
    .with_prefix("my_prefix_".to_string())
    .with_device(4) // Use 4 threads for this job.
    .build()
    .do_bulk()?;
    Ok(())
}
```

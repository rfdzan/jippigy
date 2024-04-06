use image::RgbImage;
use jippigy::{bulk::Parallel, single::Single};
use std::fs;
use std::path::PathBuf;
struct Dummy {
    temp_dir: tempdir::TempDir,
    image_path: PathBuf,
}
impl Dummy {
    fn new(temp_dir: tempdir::TempDir) -> Self {
        let image_path = temp_dir.path().join("example_jpeg.jpg");
        Self {
            temp_dir,
            image_path,
        }
    }
    fn temp_dir_val(&self) -> PathBuf {
        self.temp_dir.path().to_path_buf()
    }
    fn image_path_val(&self) -> PathBuf {
        self.image_path.clone()
    }
    fn create_example_image(&self) {
        fs::File::create(self.image_path.as_path()).unwrap();
        RgbImage::new(1000, 1000)
            .save(self.image_path.as_path())
            .unwrap();
    }
}
fn create_dummy_file() -> Dummy {
    let temp_dir = tempdir::TempDir::new("example").unwrap();
    Dummy::new(temp_dir)
}
#[test]
fn test_single() {
    let dummy = create_dummy_file();
    let _create_image = dummy.create_example_image();
    let prefix = "jippigy_".to_string();
    let single = Single::builder(dummy.image_path_val())
        .output_dir(dummy.temp_dir_val())
        .unwrap()
        .with_quality(80)
        .with_prefix(prefix.clone())
        .build()
        .compress();
    assert!(single.is_ok());
    assert!(check_prefix_and_existence(dummy, prefix, false, None));
}
#[test]
fn test_parallel() {
    let dummy = create_dummy_file();
    let _create_image = dummy.create_example_image();
    let result_dir = "compressed";
    let prefix = "jippigy_".to_string();

    let parallel_has_image_dir =
        Parallel::builder(dummy.temp_dir_val()).output_dir(dummy.temp_dir_val().join(result_dir)); // This method is required.
    assert!(parallel_has_image_dir.is_ok());

    if let Ok(parallel) = parallel_has_image_dir {
        let with_additional_parameters = parallel
            .with_quality(95)
            .with_prefix(prefix.clone())
            .with_device(4) // Use 4 threads for this job.
            .build()
            .compress();
        assert!(with_additional_parameters.is_ok());
        assert!(check_prefix_and_existence(
            dummy,
            prefix,
            true,
            Some(result_dir)
        ));
    }
}
fn check_prefix_and_existence(
    dummy: Dummy,
    expected: String,
    parallel: bool,
    dir: Option<&str>,
) -> bool {
    if !parallel {
        let file = dummy
            .temp_dir_val()
            .join(expected.to_string() + "example_jpeg.jpg");
        println!("{}", file.display());
        file.exists()
    } else {
        let file = dummy
            .temp_dir_val()
            .join(dir.unwrap_or_default())
            .join(expected.to_string() + "example_jpeg.jpg");
        println!("{}", file.display());
        file.exists()
    }
}

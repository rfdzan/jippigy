use image::RgbImage;
use jippigy::{bulk::Parallel, single::Single};
use std::fs;
use std::path::PathBuf;

const EXAMPLE_JPEG_NAME: &str = "example_jpeg.jpg";
const TEMP_DIR_NAME: &str = "example";
const RESULT_DIR_NAME: &str = "compressed";

struct Dummy {
    temp_dir: tempdir::TempDir,
    image_path: PathBuf,
}
impl Dummy {
    fn new(temp_dir: tempdir::TempDir) -> Self {
        let image_path = temp_dir.path().join(EXAMPLE_JPEG_NAME);
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
    fn create_example_image(&self) -> Vec<u8> {
        fs::File::create(self.image_path.as_path()).unwrap();
        RgbImage::new(1000, 1000).into_vec().into()
    }
}
fn create_dummy_file() -> Dummy {
    let temp_dir = tempdir::TempDir::new(TEMP_DIR_NAME).unwrap();
    Dummy::new(temp_dir)
}
#[test]
fn test_single() {
    let dummy = create_dummy_file();
    let _create_image = dummy.create_example_image();
    let single = Single::from_bytes(dummy.create_example_image().as_slice())
        .with_quality(80)
        .build()
        .compress();
    assert!(single.is_ok());
    // assert!(check_prefix_and_existence(dummy, prefix, false, None));
}
// #[test]
// fn test_parallel() {
//     let dummy = create_dummy_file();
//     let _create_image = dummy.create_example_image();
//     let prefix = PREFIX.to_string();

//     let parallel_has_image_dir = Parallel::builder(dummy.temp_dir_val())
//         .output_dir(dummy.temp_dir_val().join(RESULT_DIR_NAME)); // This method is required.
//     assert!(parallel_has_image_dir.is_ok());

//     if let Ok(parallel) = parallel_has_image_dir {
//         let with_additional_parameters = parallel
//             .with_quality(95)
//             .with_prefix(prefix.clone())
//             .with_device(4) // Use 4 threads for this job.
//             .build()
//             .compress();
//         assert!(with_additional_parameters.is_ok());
//         assert!(check_prefix_and_existence(
//             dummy,
//             prefix,
//             true,
//             Some(RESULT_DIR_NAME)
//         ));
//     }
// }
// fn check_prefix_and_existence(
//     dummy: Dummy,
//     expected: String,
//     parallel: bool,
//     dir: Option<&str>,
// ) -> bool {
//     if !parallel {
//         let file = dummy
//             .temp_dir_val()
//             .join(expected.to_string() + EXAMPLE_JPEG_NAME);
//         println!("{}", file.display());
//         file.exists()
//     } else {
//         let file = dummy
//             .temp_dir_val()
//             .join(dir.unwrap_or_default())
//             .join(expected.to_string() + EXAMPLE_JPEG_NAME);
//         println!("{}", file.display());
//         file.exists()
//     }
// }

use image::{ImageFormat::Jpeg, RgbImage};
use image_compare::Algorithm;
use jippigy::{Parallel, Single};
use std::io::Cursor;
use std::path::PathBuf;
use std::thread;

struct Dummy {}
impl Dummy {
    fn create_failing_image() -> Vec<u8> {
        RgbImage::new(1000, 1000).into_vec()
    }
    fn create_jpeg_image() -> Vec<u8> {
        let mut jpeg = Vec::new();
        let img = RgbImage::new(1000, 1000);
        let _write = img.write_to(&mut Cursor::new(&mut jpeg), Jpeg).unwrap();
        jpeg
    }
}
#[test]
fn test_basic_failing_single() {
    let failing = Dummy::create_failing_image();
    let single = Single::from_bytes(failing).build().compress();
    assert!(single.is_err());
}
#[test]
fn test_basic_success_single() {
    let success = Dummy::create_jpeg_image();
    let single = Single::from_bytes(success).build().compress();
    assert!(single.is_ok());
}
#[test]
fn test_basic_failing_parallel() {
    let mut failing = Vec::new();
    for _ in 0..10 {
        failing.push(Dummy::create_failing_image());
    }
    for res in Parallel::from_vec(failing).build().into_iter() {
        assert!(res.is_err());
    }
}
#[test]
fn test_basic_success_parallel() {
    let mut success = Vec::new();
    for _ in 0..10 {
        success.push(Dummy::create_jpeg_image());
    }
    for res in Parallel::from_vec(success).build().into_iter() {
        assert!(res.is_ok());
    }
}
#[test]
/// This test attempts to check ONLY the **ordering** of the input original JPEG files and output compressed files. This check takes a while (adds around 3 mins of overall test time on low spec hardware) and it uses around 3-4 GBs of RAM.
fn test_ordering() {
    let test_img_path = "/home/user/Pictures/compare_img_test";
    let path = PathBuf::from(test_img_path);
    let read = std::fs::read_dir(path).unwrap();
    let mut filenames = Vec::new();
    let original = read
        .into_iter()
        .flatten()
        .filter(|direntry| direntry.path().is_file())
        .map(|direntry| {
            filenames.push(
                direntry
                    .path()
                    .file_name()
                    .unwrap_or_default()
                    .to_str()
                    .unwrap_or_default()
                    .to_string(),
            );
            std::fs::read(direntry.path()).unwrap()
        })
        .collect::<Vec<Vec<u8>>>();
    println!("loaded {} images", original.len());
    let original_rbg8 = original
        .clone()
        .into_iter()
        .map(|b| {
            image::load_from_memory_with_format(b.as_slice(), image::ImageFormat::Jpeg)
                .expect("error:\n")
                .into_rgb8()
        })
        .collect::<Vec<_>>();
    println!("converted {} images to rbg8", original_rbg8.len());
    let compressed = Parallel::from_vec(original)
        .with_quality(50)
        .with_device(4)
        .build()
        .into_iter()
        .flatten()
        .map(|r| {
            image::load_from_memory_with_format(r.as_slice(), image::ImageFormat::Jpeg)
                .expect("error:\n")
                .into_rgb8()
        })
        .collect::<Vec<_>>();
    println!("compressed {} images", compressed.len());
    let mut handles = Vec::new();
    for (bytes, filename_outer) in original_rbg8.iter().zip(filenames.clone()) {
        for (compressed_bytes, filename_inner) in compressed.iter().zip(filenames.clone()) {
            // This check assumes that two vectors `original_rgb8` and `compressed` is ordered in the exact same way with the order their paths are returned in `filenames`.

            // By zipping `original_rgb8` and `compressed` with `filenames` each, we can choose to check ONLY the images with the exact same filenames to avoid checking each item of `original_rgb8` against `compressed` which will take a long time.
            if !(filename_inner.as_str() == filename_outer.as_str()) {
                continue;
            }
            let local_bytes = bytes.clone();
            let local_compressed_bytes = compressed_bytes.clone();
            let local_filename_outer = filename_outer.clone();
            let handle = thread::spawn(move || {
                let result =
                    image_compare::rgb_hybrid_compare(&local_bytes, &local_compressed_bytes)
                        .unwrap()
                        .score;
                let result_as_percentage = result * 100.0;
                // With our initial assumption that vectors `filenames`, `original_rgb8` and `compressed` are all ordered in the exact same way, it means on every check we must be checking two of the same image (one is original the other is the compressed version of it) so it must have a similarity score (for this test and dataset anyway) above 60%.

                // disclaimer: this threshold is chosen for this dataset ONLY and it only tests that the output is ordered in the EXACT same way which allows users to do things like storing filenames in one vector, the compressed jpeg bytes in another and zipping them both so the same image will ever only have one filename and it is not mixed up with other images.
                assert!(result_as_percentage > 60.0);
                println!(
                    "{local_filename_outer} and {filename_inner} score: {result_as_percentage:.2}"
                );
            });
            handles.push(handle);
        }
    }
    for handle in handles.into_iter() {
        handle.join().unwrap();
    }
}

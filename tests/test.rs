use image::{ImageFormat::Jpeg, RgbImage};
use jippigy::{Parallel, Single};
use std::io::Cursor;

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

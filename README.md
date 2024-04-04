# smoljpg
A multi-threaded JPG compression tool, powered by [turbojpeg](https://github.com/honzasp/rust-turbojpeg).
# Why
DSLR Fine JPGs are quite large in size(>10MB), uploading a large number of them to social media platforms can take a lot of time.
# How to use
Compile it:
```bash
cargo build --release
```
Put the binary file in `PATH` so you can use it from anywhere.

# Examples
Compress with default parameters:
```bash
cd your_image_dir/
smoljpg
```
The tool comes with the following defaults:
1. Quality: `50`
2. Output directory name: `compressed/`.

Compress with custom parameters:
```bash
cd your_image_dir/
smoljpg 80
smoljpg -o dest/ 80 #create a directory with a custom name
```
Compress single images:
```bash
smoljpg -s path/to/your/image.jpg
```
Single images will be stored where your current working directory is.  

Help:
```bash
smoljpg -h
```

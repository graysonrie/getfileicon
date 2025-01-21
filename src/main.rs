use image::Image;

mod image;
mod renderer;
mod shell;
pub mod api;

fn main() {
    let file_path = r"C:\Users\grays\OneDrive\Desktop\Cursor.lnk";
    let image = Image::try_new_from_file(file_path, 64, 64).unwrap();
    println!("Image: {}", image.as_base64());
}

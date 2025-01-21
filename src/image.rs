use base64::Engine;
use image::{ImageBuffer, ImageEncoder, Rgba};
use std::path::Path;
use windows::Win32::Graphics::Gdi::DeleteObject;

use crate::{renderer, shell};

pub struct Image {
    pixels: Vec<u8>,
    width: u32,
    height: u32,
}

impl Image {
    /// Expects pixels in RGBA format
    pub fn try_new_from_file(
        path: &str,
        width: u32,
        height: u32,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        match shell::get_custom_sized_icon(path, width, height) {
            Ok(bitmap) => match renderer::extract_bitmap_pixels(bitmap) {
                Ok(pixels) => {
                    let rgba_pixels = Self::bgra_to_rgba(&pixels.0);
                    unsafe {
                        _ = DeleteObject(bitmap);
                    }
                    Ok(Self {
                        pixels: rgba_pixels,
                        width,
                        height,
                    })
                }
                Err(err) => {
                    unsafe {
                        _ = DeleteObject(bitmap);
                    }
                    Err(err.into())
                }
            },
            Err(err) => Err(err.into()),
        }
    }

    pub fn as_base64_raw(&self) -> String {
        base64::engine::general_purpose::STANDARD.encode(&self.pixels)
    }

    /// Returns the image encoded as a base64 PNG string
    pub fn as_base64_png(&self) -> Result<String, Box<dyn std::error::Error>> {
        // Create an ImageBuffer from the raw RGBA pixels
        let buffer =
            ImageBuffer::<Rgba<u8>, _>::from_raw(self.width, self.height, self.pixels.to_vec())
                .ok_or("Failed to create ImageBuffer from raw pixels")?;

        // Encode the ImageBuffer into PNG format
        let mut png_data = Vec::new();
        image::codecs::png::PngEncoder::new(&mut png_data).write_image(
            &buffer,
            self.width,
            self.height,
            image::ColorType::Rgba8,
        )?;

        // Base64 encode the PNG data
        let base64_png = base64::engine::general_purpose::STANDARD.encode(png_data);

        Ok(format!("data:image/png;base64,{}", base64_png))
    }

    pub fn save_as_png(
        &self,
        width: u32,
        height: u32,
        output_path: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let buffer = ImageBuffer::<Rgba<u8>, _>::from_raw(width, height, self.pixels.to_vec())
            .ok_or("Failed to create ImageBuffer from raw pixels")?;

        // Save the ImageBuffer as a PNG file
        buffer.save(Path::new(output_path))?;

        Ok(())
    }

    fn bgra_to_rgba(pixels: &[u8]) -> Vec<u8> {
        let mut rgba_pixels = pixels.to_vec();
        for chunk in rgba_pixels.chunks_exact_mut(4) {
            chunk.swap(0, 2); // Swap Red (chunk[2]) and Blue (chunk[0])
        }
        rgba_pixels
    }
}

use base64::Engine;
use image::{ImageBuffer, ImageEncoder, Rgba};
use std::path::Path;
use windows::Win32::Graphics::Gdi::DeleteObject;

use crate::{renderer, shell};

#[derive(Debug, Clone)]
pub struct Base64Png {
    pub base64: String,
    pub is_default: bool,
}

#[derive(Debug, Clone)]
pub struct Image {
    pixels: Vec<u8>,
    pub width: u32,
    pub height: u32,
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
    pub fn as_base64_png(&self) -> Result<Base64Png, Box<dyn std::error::Error>> {
        // Validate dimensions
        let expected_size = (self.width * self.height * 4) as usize;
        if self.pixels.len() != expected_size {
            return Err(format!(
                "Invalid dimensions: expected {} bytes for {}x{} image, got {} bytes",
                expected_size,
                self.width,
                self.height,
                self.pixels.len()
            )
            .into());
        }

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
        let base64 = format!("data:image/png;base64,{}", base64_png);
        let is_default = self.is_default_base64_png(&base64);

        Ok(Base64Png { base64, is_default })
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

    fn is_default_base64_png(&self, base64_png: &str) -> bool {
        let default = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABQAAAAUCAYAAACNiR0NAAABZElEQVR4Ae3AA6AkWZbG8f937o3IzKdyS2Oubdu2bdu2bdu2bWmMnpZKr54yMyLu+Xa3anqmhztr1a/yAJ/8CZ/wDg5v8kKEvUrVX/qSL/mSSzwvxAN80zd83dE7vvO7Lnghfu1XfinvvOP2X7vznrPv/pVf+ZXneE4EDxCllO3tbba3t9ne3mZ7e5vt7W22t7fZ3t5me3ubruvj7d7xnd/gIQ+66Xs/5mM+5iTPieDf4Nprr413eud3e5OHP+zBP/jJn/zJp3g2gn8TcfzECd76rd/uDU+f2Pm+T/qkTzrGFVT+lR728Ifzcz/z0xgD6Nrrrn/jsxcuvh3wnQCVf6XHvtiL89gXe3Hut7t7Uf/w+L+vXEHlPxaV/1hU/mNR+Y9F5T8Wlf9YVP5jUfmPReU/FpX/WFT+Y1F5gNVqNdz69Kf3/CvsH+xZSeMKKg9w9z13vvZ3fte39fwrRIb7yU/hCv4Rx8VNRaZSeusAAAAASUVORK5CYII=";
        base64_png == default
    }

    fn bgra_to_rgba(pixels: &[u8]) -> Vec<u8> {
        let mut rgba_pixels = pixels.to_vec();
        for chunk in rgba_pixels.chunks_exact_mut(4) {
            chunk.swap(0, 2); // Swap Red (chunk[2]) and Blue (chunk[0])
        }
        rgba_pixels
    }
}

use windows::Win32::Graphics::Gdi::{
    CreateCompatibleDC, DeleteDC, GetDIBits, GetObjectW, BITMAP, BITMAPINFO, BITMAPINFOHEADER,
    DIB_RGB_COLORS, HBITMAP, HDC,
};

struct DeviceContextGuard(HDC);

impl Drop for DeviceContextGuard {
    fn drop(&mut self) {
        unsafe {
            _ = DeleteDC(self.0);
        }
    }
}

/// If successful, returns the pixel values of the bitmap image in BGRA format
pub fn extract_bitmap_pixels(
    hbitmap: HBITMAP,
) -> Result<(Vec<u8>, u32, u32), windows::core::Error> {
    unsafe {
        // Get a device context with RAII guard
        let hdc = CreateCompatibleDC(None);
        if hdc.is_invalid() {
            return Err(windows::core::Error::from_win32());
        }
        let _dc_guard = DeviceContextGuard(hdc);

        // Get bitmap dimensions
        let mut bitmap: BITMAP = BITMAP::default();
        GetObjectW(
            hbitmap,
            std::mem::size_of::<BITMAP>() as i32,
            Some((&mut bitmap as *mut BITMAP).cast()),
        );

        let width = bitmap.bmWidth;
        let height = bitmap.bmHeight;
        let stride = ((width * 32 + 31) / 32) * 4; // Row size in bytes (aligned to 4 bytes)

        // Prepare bitmap info header
        let mut bitmap_info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width,
                biHeight: -height, // Negative for top-down DIB
                biPlanes: 1,
                biBitCount: 32,   // 32-bit BGRA
                biCompression: 0, // BI_RGB
                ..Default::default()
            },
            ..Default::default()
        };

        // Prepare buffer for pixel data
        let mut pixels = vec![0u8; (stride * height) as usize];

        // Extract the pixel data
        let success = GetDIBits(
            hdc,
            hbitmap,
            0,
            height as u32,
            Some(pixels.as_mut_ptr() as *mut _),
            &mut bitmap_info,
            DIB_RGB_COLORS,
        );

        if success == 0 {
            return Err(windows::core::Error::from_win32());
        }
        assert!(width >= 0, "width should be greater than 0");
        assert!(height >= 0, "height should be greater than 0");

        Ok((pixels, width as u32, height as u32))
    }
}

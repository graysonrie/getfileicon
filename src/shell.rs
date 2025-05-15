use windows::core::PCWSTR;
use windows::Win32;
use windows::Win32::Graphics::Gdi::{GetObjectW, BITMAP, HBITMAP};
use windows::Win32::Storage::FileSystem::FILE_ATTRIBUTE_NORMAL;
use windows::Win32::System::Com::{CoInitializeEx, CoUninitialize, COINIT_MULTITHREADED};
use windows::Win32::UI::Shell::{
    IShellItemImageFactory, SHCreateItemFromParsingName, SHGetFileInfoW, SHFILEINFOW,
    SHGFI_SYSICONINDEX, SHGFI_USEFILEATTRIBUTES, SIIGBF_BIGGERSIZEOK, SIIGBF_RESIZETOFIT,
};

pub fn get_custom_sized_icon(
    file_path: &str,
    width: u32,
    height: u32,
) -> Result<HBITMAP, windows::core::Error> {
    unsafe {
        _ = CoInitializeEx(None, COINIT_MULTITHREADED);

        let result = {
            // Convert file path to a PCWSTR
            let file_path_wide: Vec<u16> = file_path.encode_utf16().chain(Some(0)).collect();

            // Create the shell item from the file path
            let image_factory: IShellItemImageFactory =
                SHCreateItemFromParsingName(PCWSTR(file_path_wide.as_ptr()), None)?;

            // Get the bitmap with the desired size
            image_factory.GetImage(
                Win32::Foundation::SIZE {
                    cx: width as i32,
                    cy: height as i32,
                },
                SIIGBF_BIGGERSIZEOK | SIIGBF_RESIZETOFIT,
            )
        };

        CoUninitialize();
        result
    }
}

/// Gets the recommended icon size for a file
pub fn get_recommended_icon_size(file_path: &str) -> Result<(u32, u32), windows::core::Error> {
    unsafe {
        _ = CoInitializeEx(None, COINIT_MULTITHREADED);

        let result = {
            let file_path_wide: Vec<u16> = file_path.encode_utf16().chain(Some(0)).collect();
            let mut file_info = SHFILEINFOW::default();

            // Get the system icon index
            let _ = SHGetFileInfoW(
                PCWSTR(file_path_wide.as_ptr()),
                FILE_ATTRIBUTE_NORMAL,
                Some(&mut file_info),
                std::mem::size_of::<SHFILEINFOW>() as u32,
                SHGFI_SYSICONINDEX | SHGFI_USEFILEATTRIBUTES,
            );

            // Try to get the largest available icon size
            let image_factory: IShellItemImageFactory =
                SHCreateItemFromParsingName(PCWSTR(file_path_wide.as_ptr()), None)?;

            // Try different sizes in order of preference
            let sizes = [
                (256, 256), // JUMBO square
                (256, 128), // JUMBO wide
                (128, 256), // JUMBO tall
                (48, 48),   // EXTRALARGE square
                (48, 32),   // EXTRALARGE wide
                (32, 48),   // EXTRALARGE tall
                (32, 32),   // LARGE square
                (32, 24),   // LARGE wide
                (24, 32),   // LARGE tall
                (16, 16),   // SMALL square
                (16, 12),   // SMALL wide
                (12, 16),   // SMALL tall
            ];

            for (width, height) in sizes {
                tracing::debug!("Trying size {}x{}", width, height);
                match image_factory.GetImage(
                    Win32::Foundation::SIZE {
                        cx: width as i32,
                        cy: height as i32,
                    },
                    SIIGBF_BIGGERSIZEOK | SIIGBF_RESIZETOFIT,
                ) {
                    Ok(bitmap) => {
                        // Get the actual dimensions of the bitmap
                        let mut bm = BITMAP::default();
                        if GetObjectW(
                            bitmap,
                            std::mem::size_of::<BITMAP>() as i32,
                            Some(&mut bm as *mut _ as *mut _),
                        ) > 0
                        {
                            tracing::debug!(
                                "Got bitmap with actual size {}x{}",
                                bm.bmWidth,
                                bm.bmHeight
                            );
                            CoUninitialize();
                            return Ok((bm.bmWidth as u32, bm.bmHeight as u32));
                        }
                        // If GetObjectW fails, fall back to requested size
                        tracing::debug!("Using requested size {}x{}", width, height);
                        CoUninitialize();
                        return Ok((width, height));
                    }
                    Err(_) => continue,
                }
            }

            // If all sizes fail, return the smallest size
            Ok((16, 16))
        };

        CoUninitialize();
        result
    }
}

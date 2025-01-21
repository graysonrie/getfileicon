use windows::core::PCWSTR;
use windows::Win32;
use windows::Win32::Graphics::Gdi::HBITMAP;
use windows::Win32::System::Com::{CoInitializeEx, CoUninitialize, COINIT_MULTITHREADED};
use windows::Win32::UI::Shell::{
    IShellItemImageFactory, SHCreateItemFromParsingName, SIIGBF_BIGGERSIZEOK, SIIGBF_RESIZETOFIT,
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
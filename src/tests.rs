#[cfg(test)]
mod tests {
    use crate::image::Image;
    use std::env;

    fn get_system32_dir() -> String {
        format!(
            "{}\\System32",
            env::var("WINDIR").unwrap_or_else(|_| "C:\\Windows".to_string())
        )
    }

    #[test]
    fn test_image_creation() {
        // Test with cmd.exe which should definitely exist
        let test_file = format!("{}\\cmd.exe", get_system32_dir());
        let result = Image::try_new_from_file(&test_file, 32, 32);
        assert!(result.is_ok(), "Failed to create image from cmd.exe");
        result.unwrap().save_as_png(32, 32, "test.png").unwrap();
    }

    #[test]
    fn test_invalid_file() {
        // Test with a non-existent file
        let test_file = "nonexistent_file_that_should_not_exist_12345.exe";
        let result = Image::try_new_from_file(test_file, 32, 32);
        assert!(result.is_err());
    }

    #[test]
    fn test_image_dimensions() {
        let test_file = format!("{}\\cmd.exe", get_system32_dir());
        if let Ok(image) = Image::try_new_from_file(&test_file, 32, 32) {
            assert_eq!(image.width, 32);
            assert_eq!(image.height, 32);
        }
    }

    #[test]
    fn test_base64_conversion() {
        let test_file = format!("{}\\cmd.exe", get_system32_dir());
        if let Ok(image) = Image::try_new_from_file(&test_file, 32, 32) {
            let base64 = image.as_base64_raw();
            assert!(!base64.is_empty());
        }
    }
}

use image::ImageReader;
use std::io::Cursor;
use tao::window::Icon;

pub fn load_icon(path: &[u8]) -> anyhow::Result<Icon> {
    // Decode the image from bytes
    let img = ImageReader::new(Cursor::new(path))
        .with_guessed_format()?
        .decode()?
        .into_rgba8(); // Convert to RGBA8 format

    let (width, height) = img.dimensions();
    let rgba = img.into_raw();

    // Ensure data length matches width * height * 4
    if rgba.len() != (width * height * 4) as usize {
        anyhow::bail!("Decoded image size does not match width and height");
    }

    Icon::from_rgba(rgba, width, height)
        .map_err(|e| anyhow::anyhow!("Failed to create Icon: {}", e))
}

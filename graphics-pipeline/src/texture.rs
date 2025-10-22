//! Texture management

/// Texture formats
pub enum TextureFormat {
    RGBA8,
    RGB8,
    R8,
}

/// Texture resource
pub struct Texture {
    width: u32,
    height: u32,
    format: TextureFormat,
    data: Vec<u8>,
}

impl Texture {
    pub fn new(width: u32, height: u32, format: TextureFormat) -> Self {
        let bytes_per_pixel = match format {
            TextureFormat::RGBA8 => 4,
            TextureFormat::RGB8 => 3,
            TextureFormat::R8 => 1,
        };
        
        let data = vec![0u8; (width * height * bytes_per_pixel) as usize];
        
        Self {
            width,
            height,
            format,
            data,
        }
    }
}
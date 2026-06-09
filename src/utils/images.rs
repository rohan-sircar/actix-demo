use actix_web::web;
use image::{DynamicImage, GenericImageView, ImageFormat};
use std::io::Cursor;

use crate::errors::DomainError;

/// Maximum dimensions for each image variant
const THUMBNAIL_MAX_DIMENSION: u32 = 80;
const MEDIUM_MAX_DIMENSION: u32 = 400;
const ORIGINAL_MAX_DIMENSION: u32 = 1920;

/// Resized image variants
#[derive(Debug, Clone)]
pub struct ResizedImage {
    pub thumbnail: web::Bytes,
    pub medium: web::Bytes,
    pub original: web::Bytes,
}

/// Resizes an image and encodes all three variants as WebP
pub fn resize_and_encode_webp(
    bytes: &[u8],
) -> Result<ResizedImage, DomainError> {
    let img = image::load_from_memory(bytes).map_err(|err| {
        DomainError::new_bad_input_error(format!(
            "Failed to decode image: {}",
            err
        ))
    })?;

    let thumbnail =
        encode_webp(&resize_image(img.clone(), THUMBNAIL_MAX_DIMENSION))?;
    let medium = encode_webp(&resize_image(img.clone(), MEDIUM_MAX_DIMENSION))?;
    let original = encode_webp(&resize_image(img, ORIGINAL_MAX_DIMENSION))?;

    Ok(ResizedImage {
        thumbnail,
        medium,
        original,
    })
}

fn resize_image(img: DynamicImage, max_dimension: u32) -> DynamicImage {
    let (width, height) = img.dimensions();

    if width <= max_dimension && height <= max_dimension {
        return img;
    }

    let scale = (max_dimension as f64 / width.max(height) as f64).min(1.0);
    let new_width = (width as f64 * scale) as u32;
    let new_height = (height as f64 * scale) as u32;

    img.resize_to_fill(
        new_width,
        new_height,
        image::imageops::FilterType::Lanczos3,
    )
}

fn encode_webp(img: &DynamicImage) -> Result<web::Bytes, DomainError> {
    let mut buffer = Vec::new();
    img.write_to(&mut Cursor::new(&mut buffer), ImageFormat::WebP)
        .map_err(|err| {
            DomainError::new_internal_error(format!(
                "Failed to encode WebP: {}",
                err
            ))
        })?;

    Ok(web::Bytes::from(buffer))
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageFormat, RgbImage};
    use std::io::Cursor;

    #[test]
    fn resize_large_image_to_correct_dimensions() {
        let img = RgbImage::from_pixel(1000, 800, image::Rgb([255, 0, 0]));
        let mut bytes = Vec::new();
        img.write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)
            .unwrap();

        let resized = resize_and_encode_webp(&bytes).unwrap();

        let thumbnail = image::load_from_memory(&resized.thumbnail).unwrap();
        let medium = image::load_from_memory(&resized.medium).unwrap();
        let original = image::load_from_memory(&resized.original).unwrap();

        assert!(
            thumbnail.width() <= 80,
            "thumbnail {} > 80",
            thumbnail.width()
        );
        assert!(
            thumbnail.height() <= 80,
            "thumbnail {} > 80",
            thumbnail.height()
        );

        assert!(medium.width() <= 400, "medium {} > 400", medium.width());
        assert!(medium.height() <= 400, "medium {} > 400", medium.height());

        assert!(
            original.width() <= 1920,
            "original {} > 1920",
            original.width()
        );
        assert!(
            original.height() <= 1920,
            "original {} > 1920",
            original.height()
        );

        assert_eq!(thumbnail.width() * 4, thumbnail.height() * 5);
        assert_eq!(medium.width() * 4, medium.height() * 5);
        assert_eq!(original.width() * 4, original.height() * 5);
    }

    #[test]
    fn resize_small_image_no_upscaling() {
        let img = RgbImage::from_pixel(50, 50, image::Rgb([0, 255, 0]));
        let mut bytes = Vec::new();
        img.write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)
            .unwrap();

        let resized = resize_and_encode_webp(&bytes).unwrap();

        let thumbnail = image::load_from_memory(&resized.thumbnail).unwrap();
        let medium = image::load_from_memory(&resized.medium).unwrap();
        let original = image::load_from_memory(&resized.original).unwrap();

        assert_eq!(thumbnail.width(), 50, "thumbnail was upscaled");
        assert_eq!(thumbnail.height(), 50, "thumbnail was upscaled");
        assert_eq!(medium.width(), 50, "medium was upscaled");
        assert_eq!(medium.height(), 50, "medium was upscaled");
        assert_eq!(original.width(), 50, "original was upscaled");
        assert_eq!(original.height(), 50, "original was upscaled");
    }
}

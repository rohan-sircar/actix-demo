use actix_web::web;
use image::{DynamicImage, GenericImageView, ImageFormat};
use std::io::Cursor;

use crate::errors::DomainError;

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
    max_dimension: u32,
) -> Result<ResizedImage, DomainError> {
    let img = image::load_from_memory(bytes).map_err(|err| {
        DomainError::new_bad_input_error(format!(
            "Failed to decode image: {}",
            err
        ))
    })?;

    let resized_original = resize_image(img, max_dimension);

    let thumbnail = encode_webp(&resized_original, 75)?;
    let medium = encode_webp(&resized_original, 80)?;
    let original = encode_webp(&resized_original, 85)?;

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

fn encode_webp(
    img: &DynamicImage,
    _quality: u8,
) -> Result<web::Bytes, DomainError> {
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
